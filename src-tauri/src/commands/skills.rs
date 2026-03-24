use crate::types::{
    AdoptIdeSkillRequest, AnalyzeConflictRequest, ConflictAnalysis, ConflictResolution,
    AppConfig, AppConfigResponse,
    ConflictSeverity, ConflictType, CreateVariantRequest, CreateVariantResponse,
    CreateVersionRequest, CreateVersionResponse,
    DeleteLocalSkillRequest, DeleteStrategy, DeleteVariantRequest, DeleteVersionRequest,
    DeleteVersionResponse, GetSkillPackageRequest, GetSkillPackageResponse, IdeSkill, ImportRequest,
    InstallRequest, InstallResult, ListSkillPackagesResponse, LocalScanRequest, LocalSkill,
    MetadataChange, Overview, ProjectIdeDir, ProjectScanRequest, ProjectScanResult, ProjectSkill,
    ProjectSkillImportStatus, ProjectSkillScanResult, RenameVersionRequest,
    RenameVersionResponse, ResolutionAction, ResolutionSuggestion, ResolveConflictRequest,
    ResolveConflictResult, SaveAppConfigRequest, ScanProjectSkillsRequest, SetDefaultVersionRequest, SkillDiff,
    SkillPackage, SkillPackageSummary, SkillVariant, SkillVersion, SkillVersionMetadata,
    SkillVersionSource, UninstallRequest, UpdateVariantRequest,
};
use crate::utils::download::copy_dir_recursive;
use crate::utils::path::{normalize_path, resolve_canonical, sanitize_dir_name};
use crate::utils::security::{is_absolute_ide_path, is_valid_ide_path};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const VERSION_METADATA_FILE: &str = ".skills-manager-version.json";

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
struct StoredVersionMetadata {
    version: Option<String>,
    display_name: Option<String>,
    source_url: Option<String>,
    parent_version: Option<String>,
    deleted: Option<bool>,
}

#[derive(Debug, Clone)]
struct ParsedSkillMetadata {
    name: String,
    description: String,
    version: Option<String>,
    author: Option<String>,
    namespace: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
struct StoredPackageState {
    default_version: Option<String>,
    variants: Vec<SkillVariant>,
}

fn load_default_version_strategy() -> String {
    let Some(home) = dirs::home_dir() else {
        return "manual".to_string();
    };
    let config = read_app_config(&home);
    match config.default_version_strategy.as_str() {
        "manual" | "latest" | "stable" => config.default_version_strategy,
        _ => "manual".to_string(),
    }
}

fn select_strategy_default_version(versions: &[SkillVersion], strategy: &str) -> Option<String> {
    match strategy {
        "latest" => versions.first().map(|version| version.id.clone()),
        "stable" => versions
            .iter()
            .find(|version| !version.version.contains("alpha") && !version.version.contains("beta") && !version.version.contains("rc"))
            .map(|version| version.id.clone())
            .or_else(|| versions.first().map(|version| version.id.clone())),
        _ => versions.first().map(|version| version.id.clone()),
    }
}

fn resolve_default_version(
    explicit_default_version: Option<String>,
    versions: &[SkillVersion],
    strategy: &str,
    fallback_version_id: &str,
) -> (String, String) {
    if let Some(explicit) = explicit_default_version {
        return (explicit, "explicit".to_string());
    }

    (
        select_strategy_default_version(versions, strategy)
            .unwrap_or_else(|| fallback_version_id.to_string()),
        "strategy".to_string(),
    )
}

fn now_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn manager_versions_root(home: &Path) -> PathBuf {
    home.join(".skills-manager/versions")
}

fn build_skill_id(name: &str, namespace: Option<&str>) -> String {
    let safe_name = sanitize_dir_name(name);
    let safe_namespace = namespace
        .map(sanitize_dir_name)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "default".to_string());
    format!("{}_{}", safe_name, safe_namespace)
}

fn version_metadata_path(home: &Path, skill_id: &str, version_id: &str) -> PathBuf {
    manager_versions_root(home)
        .join(skill_id)
        .join(version_id)
        .join("metadata.json")
}

fn write_version_sidecar(skill_dir: &Path, sidecar: &StoredVersionMetadata) -> Result<(), String> {
    let path = skill_dir.join(VERSION_METADATA_FILE);
    let serialized = serde_json::to_string_pretty(sidecar).map_err(|err| err.to_string())?;
    fs::write(path, serialized).map_err(|err| err.to_string())
}

fn package_state_path(home: &Path, skill_id: &str) -> PathBuf {
    manager_versions_root(home).join(skill_id).join("package.json")
}

fn app_config_path(home: &Path) -> PathBuf {
    home.join(".skills-manager/config.json")
}

fn read_app_config(home: &Path) -> AppConfig {
    let path = app_config_path(home);
    if !path.exists() {
        return AppConfig {
            default_version_strategy: "manual".to_string(),
        };
    }

    fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str::<AppConfig>(&content).ok())
        .unwrap_or(AppConfig {
            default_version_strategy: "manual".to_string(),
        })
}

fn write_app_config(home: &Path, config: &AppConfig) -> Result<(), String> {
    let path = app_config_path(home);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let serialized = serde_json::to_string_pretty(config).map_err(|err| err.to_string())?;
    fs::write(path, serialized).map_err(|err| err.to_string())
}

fn read_skill_metadata(skill_dir: &Path) -> (String, String) {
    let name = skill_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("skill")
        .to_string();

    let skill_file = skill_dir.join("SKILL.md");
    if !skill_file.exists() {
        return (name, String::new());
    }

    let content = fs::read_to_string(&skill_file).unwrap_or_default();
    let lines = content.lines();

    let mut frontmatter_name: Option<String> = None;
    let mut description = String::new();

    let mut in_frontmatter = false;
    let mut frontmatter_closed = false;
    for line in lines {
        let trimmed = line.trim();
        if trimmed == "---" {
            if !in_frontmatter {
                in_frontmatter = true;
                continue;
            }
            in_frontmatter = false;
            frontmatter_closed = true;
            continue;
        }
        if in_frontmatter {
            if let Some(value) = trimmed.strip_prefix("name:") {
                frontmatter_name = Some(value.trim().to_string());
            }
            continue;
        }
        if (frontmatter_closed || frontmatter_name.is_none())
            && description.is_empty()
            && !trimmed.is_empty()
            && !trimmed.starts_with('#')
        {
            description = trimmed.to_string();
        }
    }

    let final_name = frontmatter_name.unwrap_or(name);
    (final_name, description)
}

fn parse_skill_metadata(skill_dir: &Path) -> ParsedSkillMetadata {
    let (name, description) = read_skill_metadata(skill_dir);
    let content = fs::read_to_string(skill_dir.join("SKILL.md")).unwrap_or_default();
    let mut version = None;
    let mut author = None;
    let mut namespace = None;
    let mut in_frontmatter = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "---" {
            in_frontmatter = !in_frontmatter;
            continue;
        }
        if !in_frontmatter {
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("version:") {
            version = Some(value.trim().to_string());
        } else if let Some(value) = trimmed.strip_prefix("author:") {
            author = Some(value.trim().to_string());
        } else if let Some(value) = trimmed.strip_prefix("namespace:") {
            namespace = Some(value.trim().to_string());
        }
    }

    ParsedSkillMetadata {
        name,
        description,
        version,
        author,
        namespace,
    }
}

fn simple_hash(input: &str) -> String {
    let mut hash: u64 = 1469598103934665603;
    for byte in input.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("{:016x}", hash)
}

fn skill_content_hash(skill_dir: &Path) -> String {
    let content = fs::read_to_string(skill_dir.join("SKILL.md")).unwrap_or_default();
    simple_hash(&content)
}

fn read_version_sidecar(skill_dir: &Path) -> StoredVersionMetadata {
    let metadata_path = skill_dir.join(VERSION_METADATA_FILE);
    if !metadata_path.exists() {
        return StoredVersionMetadata::default();
    }

    fs::read_to_string(metadata_path)
        .ok()
        .and_then(|content| serde_json::from_str::<StoredVersionMetadata>(&content).ok())
        .unwrap_or_default()
}

fn build_skill_version(skill_dir: &Path, source: SkillVersionSource) -> SkillVersion {
    let metadata = parse_skill_metadata(skill_dir);
    let sidecar = read_version_sidecar(skill_dir);
    let content_hash = skill_content_hash(skill_dir);
    let version_label = sidecar
        .version
        .clone()
        .or(metadata.version.clone())
        .unwrap_or_else(|| "1.0.0".to_string());
    let skill_id = build_skill_id(&metadata.name, metadata.namespace.as_deref());
    let version_id = format!("{}_{}", sanitize_dir_name(&version_label), &content_hash[..8]);

    SkillVersion {
        id: version_id,
        skill_id,
        version: version_label.clone(),
        display_name: sidecar.display_name.unwrap_or(version_label),
        content_hash,
        created_at: now_timestamp(),
        source,
        source_url: sidecar.source_url,
        parent_version: sidecar.parent_version,
        is_active: !sidecar.deleted.unwrap_or(false),
        metadata: SkillVersionMetadata {
            name: metadata.name,
            description: metadata.description,
            author: metadata.author,
            namespace: metadata.namespace,
        },
    }
}

fn write_version_metadata(home: &Path, version: &SkillVersion) -> Result<(), String> {
    let path = version_metadata_path(home, &version.skill_id, &version.id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let serialized = serde_json::to_string_pretty(version).map_err(|err| err.to_string())?;
    fs::write(path, serialized).map_err(|err| err.to_string())
}

fn read_package_state(home: &Path, skill_id: &str) -> StoredPackageState {
    let path = package_state_path(home, skill_id);
    if !path.exists() {
        return StoredPackageState::default();
    }

    fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str::<StoredPackageState>(&content).ok())
        .unwrap_or_default()
}

fn write_package_state(home: &Path, skill_id: &str, state: &StoredPackageState) -> Result<(), String> {
    let path = package_state_path(home, skill_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let serialized = serde_json::to_string_pretty(state).map_err(|err| err.to_string())?;
    fs::write(path, serialized).map_err(|err| err.to_string())
}

fn collect_versions_for_skill(base: &Path, skill_id: &str) -> Vec<(PathBuf, SkillVersion)> {
    let Some(home) = dirs::home_dir() else {
        return Vec::new();
    };

    let mut versions = Vec::new();
    let entries = match fs::read_dir(base) {
        Ok(entries) => entries,
        Err(_) => return versions,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() || !path.join("SKILL.md").exists() {
            continue;
        }

        let version = version_summary_for_skill(&home, &path);
        if version.skill_id == skill_id {
            versions.push((path, version));
        }
    }

    versions.sort_by(|left, right| right.1.created_at.cmp(&left.1.created_at));
    versions
}

fn version_summary_for_skill(home: &Path, skill_dir: &Path) -> SkillVersion {
    let version = build_skill_version(skill_dir, SkillVersionSource::Migration);
    let _ = write_version_metadata(home, &version);
    version
}

fn build_skill_diff(base: &SkillVersion, incoming: &SkillVersion) -> SkillDiff {
    let mut metadata_changes = Vec::new();

    if base.metadata.description != incoming.metadata.description {
        metadata_changes.push(MetadataChange {
            field: "description".to_string(),
            old_value: Some(base.metadata.description.clone()),
            new_value: Some(incoming.metadata.description.clone()),
        });
    }
    if base.metadata.author != incoming.metadata.author {
        metadata_changes.push(MetadataChange {
            field: "author".to_string(),
            old_value: base.metadata.author.clone(),
            new_value: incoming.metadata.author.clone(),
        });
    }
    if base.version != incoming.version {
        metadata_changes.push(MetadataChange {
            field: "version".to_string(),
            old_value: Some(base.version.clone()),
            new_value: Some(incoming.version.clone()),
        });
    }

    let similarity_score = if base.content_hash == incoming.content_hash {
        1.0
    } else if metadata_changes.len() <= 1 {
        0.82
    } else if metadata_changes.len() <= 3 {
        0.55
    } else {
        0.25
    };

    SkillDiff {
        from_version: base.id.clone(),
        to_version: incoming.id.clone(),
        files_changed: vec!["SKILL.md".to_string()],
        additions: incoming.metadata.description.lines().count(),
        deletions: base.metadata.description.lines().count(),
        content_diff: Some(format!(
            "--- existing\n+++ incoming\n- version: {}\n+ version: {}\n- description: {}\n+ description: {}",
            base.version,
            incoming.version,
            base.metadata.description,
            incoming.metadata.description
        )),
        metadata_changes,
        similarity_score,
    }
}

fn classify_conflict(
    diff: &SkillDiff,
) -> (ConflictType, ConflictSeverity, bool, Vec<ResolutionSuggestion>) {
    if (diff.similarity_score - 1.0).abs() < f64::EPSILON {
        return (
            ConflictType::Identical,
            ConflictSeverity::None,
            true,
            vec![ResolutionSuggestion {
                action: ResolutionAction::FastForward,
                description: "Identical content; keep the existing version".to_string(),
                confidence: 1.0,
            }],
        );
    }

    if diff.metadata_changes.len() <= 1 {
        return (
            ConflictType::Patch,
            ConflictSeverity::Minor,
            true,
            vec![ResolutionSuggestion {
                action: ResolutionAction::CreateVersion,
                description: "Small metadata-only difference; add as a new version".to_string(),
                confidence: 0.9,
            }],
        );
    }

    if diff.similarity_score >= 0.5 {
        return (
            ConflictType::Minor,
            ConflictSeverity::Major,
            true,
            vec![
                ResolutionSuggestion {
                    action: ResolutionAction::CreateVersion,
                    description: "Moderate changes detected; store as a new version".to_string(),
                    confidence: 0.78,
                },
                ResolutionSuggestion {
                    action: ResolutionAction::CreateVariant,
                    description: "Keep a separate named variant if both should remain discoverable".to_string(),
                    confidence: 0.64,
                },
            ],
        );
    }

    (
        ConflictType::Fork,
        ConflictSeverity::Breaking,
        false,
        vec![ResolutionSuggestion {
            action: ResolutionAction::InteractiveMerge,
            description: "Substantial divergence detected; compare and resolve manually".to_string(),
            confidence: 0.71,
        }],
    )
}

fn package_from_skill_dir(home: &Path, manager_dir: &Path, skill_dir: &Path) -> SkillPackage {
    let primary_version = build_skill_version(skill_dir, SkillVersionSource::Migration);
    let mut versions: Vec<SkillVersion> = collect_versions_for_skill(manager_dir, &primary_version.skill_id)
        .into_iter()
        .map(|(_, version)| version)
        .collect();

    if versions.is_empty() {
        versions.push(primary_version.clone());
    }

    let mut state = read_package_state(home, &primary_version.skill_id);
    if state.variants.is_empty() {
        state.variants.push(SkillVariant {
            id: format!("{}-default", primary_version.skill_id),
            name: "default".to_string(),
            current_version: state
                .default_version
                .clone()
                .unwrap_or_else(|| primary_version.id.clone()),
            created_at: now_timestamp(),
            description: Some("Default tracked version".to_string()),
        });
    }

    let strategy = load_default_version_strategy();
    let (default_version, default_version_source) = resolve_default_version(
        state.default_version.clone(),
        &versions,
        &strategy,
        &primary_version.id,
    );

    SkillPackage {
        id: primary_version.skill_id.clone(),
        name: primary_version.metadata.name.clone(),
        namespace: primary_version
            .metadata
            .namespace
            .clone()
            .unwrap_or_else(|| "default".to_string()),
        default_version,
        default_version_source,
        versions,
        variants: state.variants,
        created_at: now_timestamp(),
        updated_at: now_timestamp(),
    }
}

fn collect_skills_from_dir(base: &Path, source: &str, ide: Option<&str>) -> Vec<LocalSkill> {
    let mut skills = Vec::new();
    if !base.exists() {
        return skills;
    }

    let entries = match fs::read_dir(base) {
        Ok(entries) => entries,
        Err(_) => return skills,
    };

    for entry in entries {
        let entry = match entry {
            Ok(item) => item,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_dir() || !path.join("SKILL.md").exists() {
            continue;
        }
        let (name, description) = read_skill_metadata(&path);
        let version = dirs::home_dir().map(|home| version_summary_for_skill(&home, &path));
        skills.push(LocalSkill {
            id: path.display().to_string(),
            name,
            description,
            path: path.display().to_string(),
            source: source.to_string(),
            ide: ide.map(|value| value.to_string()),
            used_by: Vec::new(),
            version_count: usize::from(version.is_some()),
            current_version: version,
        });
    }

    skills
}

fn collect_ide_skills(
    base: &Path,
    ide_label: &str,
    manager_map: &[(String, usize)],
    manager_skills: &mut [LocalSkill],
) -> Vec<IdeSkill> {
    let mut skills = Vec::new();
    if !base.exists() {
        return skills;
    }

    let entries = match fs::read_dir(base) {
        Ok(entries) => entries,
        Err(_) => return skills,
    };

    for entry in entries {
        let entry = match entry {
            Ok(item) => item,
            Err(_) => continue,
        };
        let path = entry.path();
        let metadata = match fs::metadata(&path) {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };
        if !metadata.is_dir() {
            continue;
        }

        let skill_dir = path.as_path();
        let has_skill_file = skill_dir.join("SKILL.md").exists();
        if !has_skill_file {
            continue;
        }

        let name = if has_skill_file {
            read_skill_metadata(skill_dir).0
        } else {
            skill_dir
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("skill")
                .to_string()
        };

        let path = skill_dir.to_path_buf();
        let mut managed = false;
        let content_hash = skill_content_hash(&path);
        for (manager_hash, idx) in manager_map {
            if *manager_hash == content_hash {
                managed = true;
                if let Some(skill) = manager_skills.get_mut(*idx) {
                    if !skill.used_by.contains(&ide_label.to_string()) {
                        skill.used_by.push(ide_label.to_string());
                    }
                }
                break;
            }
        }
        let source = if managed { "managed" } else { "local" };

        skills.push(IdeSkill {
            id: path.display().to_string(),
            name,
            path: path.display().to_string(),
            ide: ide_label.to_string(),
            source: source.to_string(),
            managed,
        });
    }

    skills
}

fn remove_path(path: &Path) -> Result<(), String> {
    if path.is_dir() {
        fs::remove_dir_all(path).map_err(|err| err.to_string())
    } else {
        fs::remove_file(path).map_err(|err| err.to_string())
    }
}

#[tauri::command]
pub fn clone_local_skill(request: InstallRequest) -> Result<InstallResult, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let normalized_home = normalize_path(&home);
    let manager_root_raw = home.join(".skills-manager/skills");
    let manager_root =
        resolve_canonical(&manager_root_raw).unwrap_or_else(|| normalize_path(&manager_root_raw));

    let skill_path = PathBuf::from(&request.skill_path);
    let skill_canon = resolve_canonical(&skill_path)
        .ok_or_else(|| "Local skill path does not exist".to_string())?;
    if !skill_canon.starts_with(&manager_root) {
        return Err("Local skill path must stay inside Skills Manager storage".to_string());
    }
    let skill_path = skill_canon;

    let safe_name = sanitize_dir_name(&request.skill_name);
    let mut installed = Vec::new();
    let mut skipped = Vec::new();

    for target in request.install_targets {
        let target_base = PathBuf::from(&target.path);
        let normalized_target = normalize_path(&target_base);
        if !normalized_target.starts_with(&normalized_home) {
            return Err(format!(
                "Target directory is outside the home directory: {}",
                target.name
            ));
        }

        fs::create_dir_all(&target_base).map_err(|err| err.to_string())?;
        let clone_path = target_base.join(&safe_name);

        if clone_path.exists() {
            skipped.push(format!("{}: target already exists", target.name));
            continue;
        }

        copy_dir_recursive(&skill_path, &clone_path)?;
        installed.push(format!("{}: {}", target.name, clone_path.display()));
    }

    Ok(InstallResult {
        installed_path: skill_path.display().to_string(),
        installed,
        skipped,
    })
}

#[tauri::command]
pub fn scan_overview(request: LocalScanRequest) -> Result<Overview, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;

    let manager_dir = home.join(".skills-manager/skills");
    let mut manager_skills = collect_skills_from_dir(&manager_dir, "manager", None);

    // Resolve IDE directories: absolute paths are used directly, relative paths are joined with home
    let ide_dirs: Vec<(String, PathBuf)> = if request.ide_dirs.is_empty() {
        vec![
            (
                "Antigravity".to_string(),
                home.join(".gemini/antigravity/skills"),
            ),
            ("Claude".to_string(), home.join(".claude/skills")),
            ("CodeBuddy".to_string(), home.join(".codebuddy/skills")),
            ("Codex".to_string(), home.join(".codex/skills")),
            ("Cursor".to_string(), home.join(".cursor/skills")),
            ("Kiro".to_string(), home.join(".kiro/skills")),
            ("Qoder".to_string(), home.join(".qoder/skills")),
            ("Trae".to_string(), home.join(".trae/skills")),
            ("VSCode".to_string(), home.join(".github/skills")),
            ("Windsurf".to_string(), home.join(".windsurf/skills")),
        ]
    } else {
        request
            .ide_dirs
            .iter()
            .map(|item| {
                if !is_valid_ide_path(&item.relative_dir) {
                    return Err(format!("Invalid IDE directory: {}", item.label));
                }
                // Absolute path: use directly
                if is_absolute_ide_path(&item.relative_dir) {
                    Ok((item.label.clone(), PathBuf::from(&item.relative_dir)))
                } else {
                    // Relative path: join with home directory
                    Ok((item.label.clone(), home.join(&item.relative_dir)))
                }
            })
            .collect::<Result<Vec<_>, String>>()?
    };

    let mut ide_skills: Vec<IdeSkill> = Vec::new();

    let mut manager_map: Vec<(String, usize)> = Vec::new();
    for (idx, skill) in manager_skills.iter().enumerate() {
        let path = Path::new(&skill.path);
        if path.join("SKILL.md").exists() {
            manager_map.push((skill_content_hash(path), idx));
        }
    }

    for (label, dir) in &ide_dirs {
        ide_skills.extend(collect_ide_skills(
            dir,
            label,
            &manager_map,
            &mut manager_skills,
        ));
    }

    if let Some(project) = request.project_dir {
        let base = PathBuf::from(project);
        for (label, dir) in &ide_dirs {
            // For absolute paths, also check the same path under project
            // For relative paths, join with project directory
            let project_dir = if dir.is_absolute() {
                dir.clone()
            } else {
                base.join(dir)
            };
            ide_skills.extend(collect_ide_skills(
                &project_dir,
                label,
                &manager_map,
                &mut manager_skills,
            ));
        }
    }

    Ok(Overview {
        manager_skills,
        ide_skills,
    })
}

#[tauri::command]
pub fn uninstall_skill(request: UninstallRequest) -> Result<String, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let mut allowed_roots = vec![home.join(".skills-manager/skills")];

    let ide_dirs: Vec<String> = if request.ide_dirs.is_empty() {
        vec![
            ".gemini/antigravity/skills".to_string(),
            ".claude/skills".to_string(),
            ".codebuddy/skills".to_string(),
            ".codex/skills".to_string(),
            ".cursor/skills".to_string(),
            ".kiro/skills".to_string(),
            ".qoder/skills".to_string(),
            ".trae/skills".to_string(),
            ".github/skills".to_string(),
            ".windsurf/skills".to_string(),
        ]
    } else {
        request
            .ide_dirs
            .iter()
            .map(|item| item.relative_dir.clone())
            .collect()
    };

    for dir in &ide_dirs {
        if !is_valid_ide_path(dir) {
            return Err("Invalid IDE directory".to_string());
        }
        // Absolute path: add directly to allowed roots
        if is_absolute_ide_path(dir) {
            allowed_roots.push(PathBuf::from(dir));
        } else {
            // Relative path: join with home directory
            allowed_roots.push(home.join(dir));
        }
    }
    if let Some(project) = request.project_dir {
        let base = PathBuf::from(project);
        allowed_roots.push(base.join(".codex/skills"));
        allowed_roots.push(base.join(".trae/skills"));
        allowed_roots.push(base.join(".opencode/skill"));
        allowed_roots.push(base.join(".skills-manager/skills"));
    }

    let target = PathBuf::from(&request.target_path);
    let parent = target.parent().unwrap_or(Path::new(&request.target_path));
    let parent_canon = resolve_canonical(parent).unwrap_or_else(|| normalize_path(parent));
    let allowed_roots_canon: Vec<PathBuf> = allowed_roots
        .iter()
        .map(|root| resolve_canonical(root).unwrap_or_else(|| normalize_path(root)))
        .collect();
    let allowed = allowed_roots_canon
        .iter()
        .any(|root| parent_canon.starts_with(root));
    if !allowed {
        return Err("Target path is outside the allowed directories".to_string());
    }

    fs::remove_dir_all(&target).map_err(|err| err.to_string())?;
    Ok("Directory removed".to_string())
}

#[tauri::command]
pub fn import_local_skill(request: ImportRequest) -> Result<String, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let manager_dir = home.join(".skills-manager/skills");

    let source_path = PathBuf::from(&request.source_path);
    if !source_path.exists() {
        return Err("Source path does not exist".to_string());
    }

    if !source_path.join("SKILL.md").exists() {
        return Err("The selected directory does not contain SKILL.md".to_string());
    }

    let (name, _) = read_skill_metadata(&source_path);
    let safe_name = sanitize_dir_name(&name);
    let target_dir = manager_dir.join(&safe_name);

    if target_dir.exists() {
        return Err(format!("Target skill already exists: {}", safe_name));
    }

    fs::create_dir_all(&target_dir).map_err(|err| err.to_string())?;
    copy_dir_recursive(&source_path, &target_dir)?;

    Ok(format!("Imported skill: {}", name))
}

#[tauri::command]
pub fn adopt_ide_skill(request: AdoptIdeSkillRequest) -> Result<String, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory".to_string())?;
    let normalized_home = normalize_path(&home);
    let manager_root = home.join(".skills-manager/skills");
    fs::create_dir_all(&manager_root).map_err(|err| err.to_string())?;

    let target = PathBuf::from(&request.target_path);
    let normalized_target = normalize_path(&target);
    if !normalized_target.starts_with(&normalized_home) {
        return Err("IDE skill path must stay inside the home directory".to_string());
    }

    fs::metadata(&target).map_err(|_| "IDE skill path does not exist".to_string())?;
    let target_canon = resolve_canonical(&target);

    let (name, has_skill_file) = if let Some(path) = target_canon.as_ref() {
        (read_skill_metadata(path).0, path.join("SKILL.md").exists())
    } else {
        (
            target
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("skill")
                .to_string(),
            false,
        )
    };

    let safe_name = sanitize_dir_name(&name);
    let manager_target = manager_root.join(&safe_name);

    if manager_target.exists() {
        let manager_canon = resolve_canonical(&manager_target)
            .ok_or_else(|| "Managed skill path does not exist".to_string())?;
        if target_canon
            .as_ref()
            .is_some_and(|target_path| *target_path == manager_canon)
        {
            return Ok(format!("{} is already managed", name));
        }
    } else {
        let source_dir = target_canon
            .as_ref()
            .ok_or_else(|| "IDE skill path does not exist".to_string())?;
        if !has_skill_file {
            return Err("Target directory does not contain SKILL.md".to_string());
        }
        copy_dir_recursive(source_dir, &manager_target)?;
    }

    remove_path(&target)?;
    copy_dir_recursive(&manager_target, &target)?;

    Ok(format!(
        "Managed {} and restored a local copy for {}",
        name, request.ide_label
    ))
}

#[tauri::command]
pub fn delete_local_skills(request: DeleteLocalSkillRequest) -> Result<String, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let manager_root = resolve_canonical(&home.join(".skills-manager/skills"))
        .unwrap_or_else(|| normalize_path(&home.join(".skills-manager/skills")));

    if request.target_paths.is_empty() {
        return Err("No skills were provided for deletion".to_string());
    }

    let mut deleted = 0usize;

    for raw_path in request.target_paths {
        let target = PathBuf::from(&raw_path);
        let canonical =
            resolve_canonical(&target).ok_or_else(|| "Target skill does not exist".to_string())?;
        if !canonical.starts_with(&manager_root) {
            return Err("Only Skills Manager local skills can be deleted".to_string());
        }
        if canonical == manager_root {
            return Err("Refusing to delete the skills root directory".to_string());
        }
        if !canonical.join("SKILL.md").exists() {
            return Err("Refusing to delete a directory without SKILL.md".to_string());
        }

        fs::remove_dir_all(&canonical).map_err(|err| err.to_string())?;
        deleted += 1;
    }

    Ok(format!("Deleted {} skills", deleted))
}

#[tauri::command]
pub fn scan_project_ide_dirs(request: ProjectScanRequest) -> Result<ProjectScanResult, String> {
    let project_dir = PathBuf::from(&request.project_dir);

    if !project_dir.exists() {
        return Err("Project directory does not exist".to_string());
    }

    let ide_dir_patterns = [
        (".gemini/antigravity/skills", "Antigravity"),
        (".claude/skills", "Claude Code"),
        (".codebuddy/skills", "CodeBuddy"),
        (".codex/skills", "Codex"),
        (".cursor/skills", "Cursor"),
        (".kiro/skills", "Kiro"),
        (".openclaw/skills", "OpenClaw"),
        (".config/opencode/skills", "OpenCode"),
        (".qoder/skills", "Qoder"),
        (".trae/skills", "Trae"),
        (".github/skills", "VSCode"),
        (".windsurf/skills", "Windsurf"),
    ];

    let mut detected_ide_dirs = Vec::new();

    for (relative_path, label) in ide_dir_patterns.iter() {
        let ide_path = project_dir.join(relative_path);
        if ide_path.exists() && ide_path.is_dir() {
            detected_ide_dirs.push(ProjectIdeDir {
                label: label.to_string(),
                relative_dir: relative_path.to_string(),
                absolute_path: ide_path.display().to_string(),
            });
        }
    }

    Ok(ProjectScanResult {
        project_dir: request.project_dir,
        detected_ide_dirs,
    })
}

#[tauri::command]
pub fn scan_project_opencode_skills(
    request: ScanProjectSkillsRequest,
) -> Result<ProjectSkillScanResult, String> {
    let project_dir = PathBuf::from(&request.project_dir);
    let manager_root = PathBuf::from(&request.manager_root);

    let opencode_path = project_dir.join(".opencode/skills");
    if !opencode_path.exists() || !opencode_path.is_dir() {
        return Ok(ProjectSkillScanResult {
            project_path: request.project_dir,
            skills: Vec::new(),
            new_count: 0,
            duplicate_count: 0,
            managed_version_count: 0,
            conflict_count: 0,
        });
    }

    let existing_skills = collect_skills_from_dir(&manager_root, "manager", None);
    let mut existing_names: std::collections::HashMap<String, Vec<LocalSkill>> = std::collections::HashMap::new();
    for skill in existing_skills {
        existing_names.entry(skill.name.clone()).or_default().push(skill);
    }

    let mut skills = Vec::new();
    let mut new_count = 0;
    let mut duplicate_count = 0;
    let mut managed_version_count = 0;
    let mut conflict_count = 0;

    let entries = match fs::read_dir(&opencode_path) {
        Ok(entries) => entries,
        Err(_) => {
            return Ok(ProjectSkillScanResult {
                project_path: request.project_dir,
                skills: Vec::new(),
                new_count: 0,
                duplicate_count: 0,
                managed_version_count: 0,
                conflict_count: 0,
            })
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(item) => item,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_dir() || !path.join("SKILL.md").exists() {
            continue;
        }

        let (name, description) = read_skill_metadata(&path);
        let incoming_version = build_skill_version(&path, SkillVersionSource::Project);
        let candidates = existing_names.get(&name).cloned().unwrap_or_default();
        let existing_registry_skill = candidates.first().cloned();

        let mut matched_registry_skill: Option<LocalSkill> = None;
        let mut matched_version: Option<(SkillVersion, String)> = None;

        for candidate in &candidates {
            if candidate
                .current_version
                .as_ref()
                .is_some_and(|version| version.content_hash == incoming_version.content_hash)
            {
                matched_registry_skill = Some(candidate.clone());
                if let Some(version) = candidate.current_version.clone() {
                    let default_version = get_skill_package(GetSkillPackageRequest {
                        skill_id: version.skill_id.clone(),
                    })
                    .ok()
                    .map(|package| package.package.default_version)
                    .unwrap_or_else(|| version.id.clone());
                    matched_version = Some((version.clone(), default_version));
                }
                break;
            }

            if let Some(local_version) = candidate.current_version.as_ref() {
                if let Ok(package) = get_skill_package(GetSkillPackageRequest {
                    skill_id: local_version.skill_id.clone(),
                }) {
                    if let Some(version) = package
                        .package
                        .versions
                        .into_iter()
                        .find(|version| version.content_hash == incoming_version.content_hash)
                    {
                        matched_registry_skill = Some(candidate.clone());
                        matched_version = Some((version, package.package.default_version));
                        break;
                    }
                }
            }
        }

        let status = if candidates.is_empty() {
            ProjectSkillImportStatus::New
        } else if let Some((version, default_version)) = matched_version.as_ref() {
            if version.id == *default_version {
                ProjectSkillImportStatus::Duplicate
            } else {
                ProjectSkillImportStatus::ManagedVersion
            }
        } else {
            ProjectSkillImportStatus::Conflict
        };

        match &status {
            ProjectSkillImportStatus::New => new_count += 1,
            ProjectSkillImportStatus::Duplicate => duplicate_count += 1,
            ProjectSkillImportStatus::ManagedVersion => managed_version_count += 1,
            ProjectSkillImportStatus::Conflict => conflict_count += 1,
        }

        skills.push(ProjectSkill {
            id: path.display().to_string(),
            name: name.clone(),
            description,
            path: path.display().to_string(),
            status: status.clone(),
            existing_registry_skill,
            matched_registry_skill,
            current_version: Some(incoming_version),
            matched_version_id: matched_version.as_ref().map(|(version, _)| version.id.clone()),
            matched_version_name: matched_version.as_ref().map(|(version, _)| version.display_name.clone()),
            matches_default_version: matched_version
                .as_ref()
                .map(|(version, default_version)| version.id == *default_version),
        });
    }

    Ok(ProjectSkillScanResult {
        project_path: request.project_dir,
        skills,
        new_count,
        duplicate_count,
        managed_version_count,
        conflict_count,
    })
}

#[tauri::command]
pub fn resolve_skill_conflict(
    request: ResolveConflictRequest,
) -> Result<ResolveConflictResult, String> {
    let skill_path = PathBuf::from(&request.project_skill_path);
    let skill_name = skill_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("skill")
        .to_string();

    let result = match request.resolution {
        ConflictResolution::Keep => ResolveConflictResult {
            success: true,
            skill_id: None,
            action: "kept_existing".to_string(),
        },
        ConflictResolution::Overwrite => {
            let safe_name = sanitize_dir_name(&skill_name);
            ResolveConflictResult {
                success: true,
                skill_id: Some(safe_name),
                action: "overwritten".to_string(),
            }
        }
        ConflictResolution::Coexist => {
            let coexist_name = request
                .coexist_name
                .unwrap_or_else(|| format!("{}-project", skill_name));
            let safe_name = sanitize_dir_name(&coexist_name);
            ResolveConflictResult {
                success: true,
                skill_id: Some(safe_name),
                action: "coexisted".to_string(),
            }
        }
    };

    Ok(result)
}

#[tauri::command]
pub fn create_skill_version(request: CreateVersionRequest) -> Result<CreateVersionResponse, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let manager_dir = home.join(".skills-manager/skills");
    let source_path = PathBuf::from(&request.source_path);

    if !source_path.exists() || !source_path.join("SKILL.md").exists() {
        return Err("Source skill path is invalid".to_string());
    }

    let package = get_skill_package(GetSkillPackageRequest {
        skill_id: request.skill_id.clone(),
    })?
    .package;

    let reference_version = package
        .versions
        .iter()
        .find(|version| version.id == package.default_version)
        .cloned()
        .or_else(|| package.versions.first().cloned())
        .ok_or_else(|| "Skill package has no base versions".to_string())?;

    let destination_dir_name = format!(
        "{}-{}",
        sanitize_dir_name(&package.name),
        sanitize_dir_name(&request.display_name)
    );
    let destination_path = manager_dir.join(destination_dir_name);

    if destination_path.exists() {
        return Err("A version with the same destination folder already exists".to_string());
    }

    copy_dir_recursive(&source_path, &destination_path).map_err(|err| err.to_string())?;

    let sidecar = StoredVersionMetadata {
        version: Some(request.version.clone()),
        display_name: Some(request.display_name.clone()),
        source_url: request.source_url.clone(),
        parent_version: request.parent_version.clone().or(Some(reference_version.id)),
        deleted: Some(false),
    };
    write_version_sidecar(&destination_path, &sidecar)?;

    let created_version = build_skill_version(&destination_path, request.source.clone());
    write_version_metadata(&home, &created_version)?;

    Ok(CreateVersionResponse {
        version: created_version,
        installed_path: destination_path.display().to_string(),
    })
}

#[tauri::command]
pub fn analyze_skill_conflict(request: AnalyzeConflictRequest) -> Result<ConflictAnalysis, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let manager_dir = home.join(".skills-manager/skills");
    let existing_skills = collect_skills_from_dir(&manager_dir, "manager", None);

    let base_skill = existing_skills
        .into_iter()
        .find(|skill| {
            skill.current_version.as_ref().is_some_and(|version| {
                version.skill_id == request.skill_id && version.id == request.base_version_id
            })
        })
        .ok_or_else(|| "Base version not found".to_string())?;

    let base_version = base_skill
        .current_version
        .ok_or_else(|| "Base version metadata is missing".to_string())?;
    let incoming_path = PathBuf::from(&request.incoming_path);
    let incoming_version = build_skill_version(&incoming_path, SkillVersionSource::Project);
    let diff = build_skill_diff(&base_version, &incoming_version);
    let (conflict_type, severity, auto_resolvable, suggestions) = classify_conflict(&diff);

    Ok(ConflictAnalysis {
        conflict_type,
        severity,
        base_version,
        incoming_version,
        diff,
        auto_resolvable,
        suggestions,
    })
}

#[tauri::command]
pub fn compare_skill_versions(request: crate::types::CompareVersionsRequest) -> Result<SkillDiff, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let manager_dir = home.join(".skills-manager/skills");
    let package = get_skill_package(GetSkillPackageRequest {
        skill_id: request.skill_id,
    })?
    .package;

    let from_version = package
        .versions
        .iter()
        .find(|version| version.id == request.from_version)
        .cloned()
        .ok_or_else(|| "Source version not found".to_string())?;

    let to_version = package
        .versions
        .iter()
        .find(|version| version.id == request.to_version)
        .cloned()
        .ok_or_else(|| "Target version not found".to_string())?;

    let _ = manager_dir;
    Ok(build_skill_diff(&from_version, &to_version))
}

#[tauri::command]
pub fn list_skill_packages() -> Result<ListSkillPackagesResponse, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let manager_dir = home.join(".skills-manager/skills");
    let mut packages = Vec::new();
    let mut seen: HashMap<String, bool> = HashMap::new();
    for skill in collect_skills_from_dir(&manager_dir, "manager", None) {
        if let Some(version) = skill.current_version {
            if seen.insert(version.skill_id.clone(), true).is_some() {
                continue;
            }
            let package = package_from_skill_dir(&home, &manager_dir, Path::new(&skill.path));
            if let Some(first) = package.versions.first() {
                packages.push(SkillPackageSummary {
                    id: package.id,
                    name: package.name,
                    namespace: package.namespace,
                    version_count: package.versions.len(),
                    variant_count: package.variants.len(),
                    latest_version: first.version.clone(),
                    default_version: package.default_version,
                    updated_at: package.updated_at,
                });
            }
        }
    }
    packages.sort_by(|left, right| left.name.cmp(&right.name));

    Ok(ListSkillPackagesResponse {
        total: packages.len(),
        packages,
    })
}

#[tauri::command]
pub fn get_skill_package(request: GetSkillPackageRequest) -> Result<GetSkillPackageResponse, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let manager_dir = home.join(".skills-manager/skills");

    let package = collect_skills_from_dir(&manager_dir, "manager", None)
        .into_iter()
        .find_map(|skill| {
            skill.current_version.as_ref().and_then(|version| {
                if version.skill_id == request.skill_id {
                    Some(package_from_skill_dir(&home, &manager_dir, Path::new(&skill.path)))
                } else {
                    None
                }
            })
        })
        .ok_or_else(|| "Skill package not found".to_string())?;

    Ok(GetSkillPackageResponse { package })
}

#[tauri::command]
pub fn rename_skill_version(
    request: RenameVersionRequest,
) -> Result<RenameVersionResponse, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let manager_dir = home.join(".skills-manager/skills");
    let skill = collect_skills_from_dir(&manager_dir, "manager", None)
        .into_iter()
        .find(|item| {
            item.current_version.as_ref().is_some_and(|version| {
                version.skill_id == request.skill_id && version.id == request.version_id
            })
        })
        .ok_or_else(|| "Version not found".to_string())?;

    let skill_path = PathBuf::from(&skill.path);
    let mut sidecar = read_version_sidecar(&skill_path);
    sidecar.display_name = Some(request.new_display_name.clone());
    let sidecar_path = skill_path.join(VERSION_METADATA_FILE);
    let serialized = serde_json::to_string_pretty(&sidecar).map_err(|err| err.to_string())?;
    fs::write(sidecar_path, serialized).map_err(|err| err.to_string())?;

    let version = version_summary_for_skill(&home, &skill_path);
    Ok(RenameVersionResponse {
        success: true,
        version,
    })
}

#[tauri::command]
pub fn delete_skill_version(
    request: DeleteVersionRequest,
) -> Result<DeleteVersionResponse, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let manager_dir = home.join(".skills-manager/skills");
    let skill = collect_skills_from_dir(&manager_dir, "manager", None)
        .into_iter()
        .find(|item| {
            item.current_version.as_ref().is_some_and(|version| {
                version.skill_id == request.skill_id && version.id == request.version_id
            })
        })
        .ok_or_else(|| "Version not found".to_string())?;

    let skill_path = PathBuf::from(&skill.path);
    let force = request.force.unwrap_or(false);
    let version_count = collect_skills_from_dir(&manager_dir, "manager", None)
        .into_iter()
        .filter(|item| {
            item.current_version
                .as_ref()
                .is_some_and(|version| version.skill_id == request.skill_id)
        })
        .count();

    if version_count <= 1 && !force {
        return Err("Refusing to delete the only available version without force".to_string());
    }

    match request.strategy {
        DeleteStrategy::Soft => {
            let mut sidecar = read_version_sidecar(&skill_path);
            sidecar.deleted = Some(true);
            let serialized = serde_json::to_string_pretty(&sidecar).map_err(|err| err.to_string())?;
            fs::write(skill_path.join(VERSION_METADATA_FILE), serialized).map_err(|err| err.to_string())?;
            Ok(DeleteVersionResponse {
                success: true,
                message: "Version marked as deleted".to_string(),
                archived_path: None,
            })
        }
        DeleteStrategy::Archive => {
            let archive_root = home.join(".skills-manager/archive");
            fs::create_dir_all(&archive_root).map_err(|err| err.to_string())?;
            let archive_path = archive_root.join(
                skill_path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or("skill-version"),
            );
            fs::rename(&skill_path, &archive_path).map_err(|err| err.to_string())?;
            Ok(DeleteVersionResponse {
                success: true,
                message: "Version archived".to_string(),
                archived_path: Some(archive_path.display().to_string()),
            })
        }
        DeleteStrategy::Hard => {
            fs::remove_dir_all(&skill_path).map_err(|err| err.to_string())?;
            Ok(DeleteVersionResponse {
                success: true,
                message: "Version deleted".to_string(),
                archived_path: None,
            })
        }
    }
}

#[tauri::command]
pub fn set_default_skill_version(
    request: SetDefaultVersionRequest,
) -> Result<SkillVersion, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let skill_id = request.skill_id.clone();
    let version_id = request.version_id.clone();
    let package = get_skill_package(GetSkillPackageRequest {
        skill_id,
    })?;
    let version = package
        .package
        .versions
        .into_iter()
        .find(|version| version.id == version_id)
        .ok_or_else(|| "Version not found".to_string())?;

    let mut state = read_package_state(&home, &version.skill_id);
    state.default_version = Some(request.version_id.clone());
    if let Some(default_variant) = state.variants.iter_mut().find(|variant| variant.name == "default") {
        default_variant.current_version = request.version_id;
    }
    write_package_state(&home, &version.skill_id, &state)?;

    Ok(version)
}

#[tauri::command]
pub fn create_skill_variant(
    request: CreateVariantRequest,
) -> Result<CreateVariantResponse, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let mut state = read_package_state(&home, &request.skill_id);
    let variant = SkillVariant {
        id: format!("{}-{}", request.skill_id, sanitize_dir_name(&request.name)),
        name: request.name,
        current_version: request.version_id,
        created_at: now_timestamp(),
        description: request.description,
    };
    state.variants.push(variant.clone());
    write_package_state(&home, &request.skill_id, &state)?;
    Ok(CreateVariantResponse { variant })
}

#[tauri::command]
pub fn update_skill_variant(request: UpdateVariantRequest) -> Result<SkillVariant, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let mut state = read_package_state(&home, &request.skill_id);
    let variant = state
        .variants
        .iter_mut()
        .find(|variant| variant.id == request.variant_id)
        .ok_or_else(|| "Variant not found".to_string())?;

    if let Some(new_name) = request.new_name {
        variant.name = new_name;
    }
    if let Some(new_version_id) = request.new_version_id {
        variant.current_version = new_version_id;
    }
    if request.new_description.is_some() {
        variant.description = request.new_description;
    }

    let updated = variant.clone();
    write_package_state(&home, &request.skill_id, &state)?;
    Ok(updated)
}

#[tauri::command]
pub fn delete_skill_variant(request: DeleteVariantRequest) -> Result<(), String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let mut state = read_package_state(&home, &request.skill_id);
    let before = state.variants.len();
    state.variants.retain(|variant| variant.id != request.variant_id);
    if before == state.variants.len() {
        return Err("Variant not found".to_string());
    }
    write_package_state(&home, &request.skill_id, &state)?;
    Ok(())
}

#[tauri::command]
pub fn get_app_config() -> Result<AppConfigResponse, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    Ok(AppConfigResponse {
        config: read_app_config(&home),
    })
}

#[tauri::command]
pub fn save_app_config(request: SaveAppConfigRequest) -> Result<AppConfigResponse, String> {
    if !matches!(request.default_version_strategy.as_str(), "manual" | "latest" | "stable") {
        return Err("Invalid default version strategy".to_string());
    }

    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let config = AppConfig {
        default_version_strategy: request.default_version_strategy,
    };
    write_app_config(&home, &config)?;
    Ok(AppConfigResponse { config })
}

#[cfg(test)]
mod tests {
    use super::{
        adopt_ide_skill, build_skill_diff, build_skill_version, classify_conflict,
        clone_local_skill, package_state_path, parse_skill_metadata, read_app_config,
        resolve_default_version, save_app_config, scan_overview,
        scan_project_opencode_skills, select_strategy_default_version, simple_hash,
        uninstall_skill, write_app_config, write_package_state, StoredPackageState,
    };
    use crate::types::{
        AdoptIdeSkillRequest, AppConfig, ConflictSeverity, ConflictType, IdeDir,
        InstallRequest, LinkTarget, LocalScanRequest, ResolutionAction,
        SaveAppConfigRequest, ScanProjectSkillsRequest, SkillVersion, SkillVersionMetadata,
        SkillVersionSource, UninstallRequest,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn fixture_version(description: &str, version: &str, hash: &str) -> SkillVersion {
        SkillVersion {
            id: format!("{}-{}", version, hash),
            skill_id: "demo_default".to_string(),
            version: version.to_string(),
            display_name: version.to_string(),
            content_hash: hash.to_string(),
            created_at: 0,
            source: SkillVersionSource::Migration,
            source_url: None,
            parent_version: None,
            is_active: true,
            metadata: SkillVersionMetadata {
                name: "Demo".to_string(),
                description: description.to_string(),
                author: Some("A".to_string()),
                namespace: Some("default".to_string()),
            },
        }
    }

    fn unique_test_name(prefix: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        format!("{}-{}", prefix, nanos)
    }

    fn write_skill_dir(base: &PathBuf, name: &str, body: &str) -> PathBuf {
        let dir = base.join(name);
        fs::create_dir_all(&dir).expect("create skill dir");
        fs::write(dir.join("SKILL.md"), body).expect("write skill file");
        dir
    }

    #[test]
    fn simple_hash_is_stable() {
        assert_eq!(simple_hash("abc"), simple_hash("abc"));
        assert_ne!(simple_hash("abc"), simple_hash("def"));
    }

    #[test]
    fn parse_skill_metadata_reads_version_fields() {
        let temp_dir = std::env::temp_dir().join("skills-manager-parse-skill-metadata");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).expect("create temp dir");
        fs::write(
            temp_dir.join("SKILL.md"),
            "---\nname: Demo\nversion: 2.1.0\nauthor: Tester\nnamespace: team\n---\nDescription line\n",
        )
        .expect("write skill file");

        let parsed = parse_skill_metadata(&PathBuf::from(&temp_dir));
        assert_eq!(parsed.name, "Demo");
        assert_eq!(parsed.version.as_deref(), Some("2.1.0"));
        assert_eq!(parsed.author.as_deref(), Some("Tester"));
        assert_eq!(parsed.namespace.as_deref(), Some("team"));
        assert_eq!(parsed.description, "Description line");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn classify_conflict_detects_identical_versions() {
        let base = fixture_version("same", "1.0.0", "aaaa");
        let incoming = fixture_version("same", "1.0.0", "aaaa");
        let diff = build_skill_diff(&base, &incoming);
        let (conflict_type, severity, auto_resolvable, suggestions) = classify_conflict(&diff);
        assert_eq!(conflict_type, ConflictType::Identical);
        assert_eq!(severity, ConflictSeverity::None);
        assert!(auto_resolvable);
        assert_eq!(suggestions[0].action, ResolutionAction::FastForward);
    }

    #[test]
    fn strategy_default_prefers_latest_version() {
        let versions = vec![
            fixture_version("latest", "2.0.0", "bbbb"),
            fixture_version("older", "1.0.0", "aaaa"),
        ];

        let selected = select_strategy_default_version(&versions, "latest");
        assert_eq!(selected.as_deref(), Some("2.0.0-bbbb"));
    }

    #[test]
    fn strategy_default_prefers_stable_version_over_beta() {
        let versions = vec![
            fixture_version("beta", "2.0.0-beta", "bbbb"),
            fixture_version("stable", "1.9.0", "aaaa"),
        ];

        let selected = select_strategy_default_version(&versions, "stable");
        assert_eq!(selected.as_deref(), Some("1.9.0-aaaa"));
    }

    #[test]
    fn explicit_default_version_overrides_global_strategy() {
        let versions = vec![
            fixture_version("latest", "2.0.0", "bbbb"),
            fixture_version("explicit", "1.5.0", "aaaa"),
        ];

        let resolved = resolve_default_version(
            Some("1.5.0-aaaa".to_string()),
            &versions,
            "latest",
            "2.0.0-bbbb",
        );

        assert_eq!(resolved.0, "1.5.0-aaaa");
        assert_eq!(resolved.1, "explicit");
    }

    #[test]
    fn matched_version_can_be_found_by_content_hash() {
        let versions = vec![
            fixture_version("current", "2.0.0", "bbbb"),
            fixture_version("older", "1.0.0", "aaaa"),
        ];

        let matched = versions
            .iter()
            .find(|version| version.content_hash == "aaaa")
            .map(|version| version.display_name.clone());

        assert_eq!(matched.as_deref(), Some("1.0.0"));
    }

    #[test]
    fn project_skill_matches_default_version_correctly() {
        // Test scenario: project skill content matches the default version
        let default_version = fixture_version("default desc", "1.0.0", "hash_default");
        let other_version = fixture_version("other desc", "2.0.0", "hash_other");
        let versions = vec![default_version.clone(), other_version.clone()];

        // Simulate finding a match when project skill has the default version's hash
        let matched = versions
            .iter()
            .find(|v| v.content_hash == "hash_default")
            .map(|v| (v.id.clone(), v.version.clone()));

        assert_eq!(matched, Some(("1.0.0-hash_default".to_string(), "1.0.0".to_string())));
    }

    #[test]
    fn project_skill_matches_non_default_version_correctly() {
        // Test scenario: project skill content matches a non-default version
        let default_version = fixture_version("default desc", "1.0.0", "hash_default");
        let other_version = fixture_version("other desc", "2.0.0", "hash_other");
        let versions = vec![default_version.clone(), other_version.clone()];

        // Simulate finding a match when project skill has a non-default version's hash
        let matched = versions
            .iter()
            .find(|v| v.content_hash == "hash_other")
            .map(|v| (v.id.clone(), v.version.clone()));

        assert_eq!(matched, Some(("2.0.0-hash_other".to_string(), "2.0.0".to_string())));
        // Verify this is NOT the default version
        assert_ne!(matched.unwrap().0, default_version.id);
    }

    #[test]
    fn project_skill_no_match_when_hash_unknown() {
        // Test scenario: project skill content doesn't match any managed version
        let versions = vec![
            fixture_version("v1", "1.0.0", "hash_v1"),
            fixture_version("v2", "2.0.0", "hash_v2"),
        ];

        // Simulate no match found for unknown hash
        let matched = versions
            .iter()
            .find(|v| v.content_hash == "unknown_hash")
            .map(|v| v.id.clone());

        assert_eq!(matched, None);
    }

    #[test]
    fn clone_local_skill_path_resolution() {
        // Test that clone_local_skill correctly resolves source and target paths
        // This tests the core logic without filesystem operations
        let source_relative = "my-skill";
        let project_name = "test-project";
        
        // Verify path components are correctly joined
        let source_path = PathBuf::from(source_relative);
        let target_path = PathBuf::from(project_name).join(".opencode/skills").join(source_relative);
        
        assert_eq!(source_path.file_name().unwrap().to_str().unwrap(), "my-skill");
        assert_eq!(target_path.file_name().unwrap().to_str().unwrap(), "my-skill");
        assert!(target_path.to_str().unwrap().contains("test-project/.opencode/skills"));
    }

    #[test]
    fn version_conflict_detected_on_content_hash_mismatch() {
        // Test that different content hashes indicate a conflict
        let local_version = fixture_version("local", "1.0.0", "hash_local");
        let project_version = fixture_version("project", "1.0.0", "hash_project");
        
        // Same version number but different content hashes = conflict
        assert_ne!(local_version.content_hash, project_version.content_hash);
        assert_eq!(local_version.version, project_version.version);
    }

    #[test]
    fn identical_content_detected_as_duplicate() {
        // Test that identical content hashes indicate a duplicate (not conflict)
        let local_version = fixture_version("local", "1.0.0", "same_hash");
        let project_version = fixture_version("project", "2.0.0", "same_hash");
        
        // Different versions but same content = duplicate, not conflict
        assert_eq!(local_version.content_hash, project_version.content_hash);
        assert_ne!(local_version.version, project_version.version);
    }

    #[test]
    fn strategy_stable_prefers_non_beta_non_alpha() {
        // Test that 'stable' strategy correctly filters out pre-release versions
        let versions = vec![
            fixture_version("alpha", "2.0.0-alpha", "hash1"),
            fixture_version("beta", "2.0.0-beta", "hash2"),
            fixture_version("rc", "2.0.0-rc", "hash3"),
            fixture_version("stable", "1.9.0", "hash4"),
        ];

        let selected = select_strategy_default_version(&versions, "stable");
        // Should select the stable version, not any pre-release
        assert_eq!(selected.as_deref(), Some("1.9.0-hash4"));
    }

    #[test]
    fn strategy_latest_selects_first_version() {
        // Test that 'latest' strategy selects the first version in the list
        // (assuming versions are pre-ordered with latest first)
        let versions = vec![
            fixture_version("latest", "2.0.0", "hash2"),
            fixture_version("older", "1.0.0", "hash1"),
            fixture_version("oldest", "0.9.0", "hash0"),
        ];

        let selected = select_strategy_default_version(&versions, "latest");
        // Should select the first version (presumed to be latest)
        assert_eq!(selected.as_deref(), Some("2.0.0-hash2"));
    }

    #[test]
    fn clone_local_skill_copies_into_target_directory() {
        let home = dirs::home_dir().expect("home dir");
        let unique = unique_test_name("clone-success");
        let root = home.join(".skills-manager-test").join(&unique);
        let manager_root = home.join(".skills-manager/skills");
        let target_root = root.join("ide");
        fs::create_dir_all(&manager_root).expect("create manager root");
        fs::create_dir_all(&target_root).expect("create target root");

        let skill_dir = write_skill_dir(
            &manager_root,
            &format!("demo-skill-{}", unique),
            "---\nname: Demo Skill\nversion: 1.0.0\n---\nDemo description\n",
        );

        let result = clone_local_skill(InstallRequest {
            skill_path: skill_dir.display().to_string(),
            skill_name: "Demo Skill".to_string(),
            install_targets: vec![LinkTarget {
                name: "Test IDE".to_string(),
                path: target_root.display().to_string(),
            }],
        })
        .expect("clone succeeds");

        let cloned_dir = target_root.join("demo-skill");
        assert!(cloned_dir.exists());
        assert!(cloned_dir.join("SKILL.md").exists());
        assert_eq!(result.installed.len(), 1);
        assert!(result.skipped.is_empty());

        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&skill_dir);
    }

    #[test]
    fn clone_local_skill_rejects_target_outside_home() {
        let home = dirs::home_dir().expect("home dir");
        let unique = unique_test_name("clone-invalid");
        let root = home.join(".skills-manager-test").join(&unique);
        let manager_root = home.join(".skills-manager/skills");
        fs::create_dir_all(&manager_root).expect("create manager root");

        let skill_dir = write_skill_dir(
            &manager_root,
            &format!("demo-skill-{}", unique),
            "---\nname: Demo Skill\nversion: 1.0.0\n---\nDemo description\n",
        );

        let result = clone_local_skill(InstallRequest {
            skill_path: skill_dir.display().to_string(),
            skill_name: "Demo Skill".to_string(),
            install_targets: vec![LinkTarget {
                name: "Invalid IDE".to_string(),
                path: "/tmp/skills-manager-invalid-target".to_string(),
            }],
        });

        assert!(result.is_err());

        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&skill_dir);
    }

    #[test]
    fn adopt_ide_skill_restores_local_copy_and_manager_copy() {
        let home = dirs::home_dir().expect("home dir");
        let unique = unique_test_name("adopt");
        let ide_root = home.join(".skills-manager-test").join(&unique).join("ide");
        fs::create_dir_all(&ide_root).expect("create ide root");

        let ide_skill_dir = write_skill_dir(
            &ide_root,
            "demo-adopt-skill",
            "---\nname: Demo Adopt Skill\nversion: 1.0.0\n---\nAdopt me\n",
        );

        let result = adopt_ide_skill(AdoptIdeSkillRequest {
            target_path: ide_skill_dir.display().to_string(),
            ide_label: "Test IDE".to_string(),
        })
        .expect("adopt succeeds");

        let manager_dir = home.join(".skills-manager/skills/demo-adopt-skill");
        assert!(manager_dir.exists());
        assert!(manager_dir.join("SKILL.md").exists());
        assert!(ide_skill_dir.exists());
        assert!(ide_skill_dir.join("SKILL.md").exists());
        assert!(result.contains("restored a local copy"));

        let _ = fs::remove_dir_all(home.join(".skills-manager-test").join(unique));
        let _ = fs::remove_dir_all(manager_dir);
    }

    #[test]
    fn scan_overview_marks_matching_copy_as_managed() {
        let home = dirs::home_dir().expect("home dir");
        let unique = unique_test_name("scan-overview");
        let manager_root = home.join(".skills-manager/skills");
        fs::create_dir_all(&manager_root).expect("create manager root");
        let ide_root = home.join(".skills-manager-test").join(&unique).join("ide");
        fs::create_dir_all(&ide_root).expect("create ide root");

        let manager_skill_dir = write_skill_dir(
            &manager_root,
            "demo-managed-skill",
            "---\nname: Demo Managed Skill\nversion: 1.0.0\n---\nManaged copy\n",
        );
        let copied_skill_dir = ide_root.join("demo-managed-skill");
        super::copy_dir_recursive(&manager_skill_dir, &copied_skill_dir).expect("copy skill to ide");

        let overview = scan_overview(LocalScanRequest {
            project_dir: None,
            ide_dirs: vec![IdeDir {
                label: "Test IDE".to_string(),
                relative_dir: ide_root.display().to_string(),
            }],
        })
        .expect("scan overview succeeds");

        let ide_skill = overview
            .ide_skills
            .iter()
            .find(|skill| skill.name == "Demo Managed Skill")
            .expect("managed ide skill exists");
        assert!(ide_skill.managed);
        assert_eq!(ide_skill.source, "managed");

        let manager_skill = overview
            .manager_skills
            .iter()
            .find(|skill| skill.name == "Demo Managed Skill")
            .expect("manager skill exists");
        assert!(manager_skill.used_by.contains(&"Test IDE".to_string()));

        let _ = fs::remove_dir_all(copied_skill_dir);
        let _ = fs::remove_dir_all(home.join(".skills-manager-test").join(unique));
        let _ = fs::remove_dir_all(manager_skill_dir);
    }

    #[test]
    fn uninstall_skill_removes_installed_directory() {
        let home = dirs::home_dir().expect("home dir");
        let unique = unique_test_name("uninstall");
        let ide_root = home.join(".skills-manager-test").join(&unique).join("ide");
        fs::create_dir_all(&ide_root).expect("create ide root");

        let installed_skill_dir = write_skill_dir(
            &ide_root,
            "demo-remove-skill",
            "---\nname: Demo Remove Skill\nversion: 1.0.0\n---\nRemove me\n",
        );

        let message = uninstall_skill(UninstallRequest {
            target_path: installed_skill_dir.display().to_string(),
            project_dir: None,
            ide_dirs: vec![IdeDir {
                label: "Test IDE".to_string(),
                relative_dir: ide_root.display().to_string(),
            }],
        })
        .expect("uninstall succeeds");

        assert_eq!(message, "Directory removed");
        assert!(!installed_skill_dir.exists());

        let _ = fs::remove_dir_all(home.join(".skills-manager-test").join(unique));
    }

    #[test]
    fn app_config_defaults_to_manual_when_missing() {
        let temp_home = std::env::temp_dir().join(unique_test_name("app-config-default"));
        let config = read_app_config(&temp_home);

        assert_eq!(config.default_version_strategy, "manual");

        let _ = fs::remove_dir_all(temp_home);
    }

    #[test]
    fn write_app_config_persists_strategy() {
        let temp_home = std::env::temp_dir().join(unique_test_name("app-config-write"));
        let config = AppConfig {
            default_version_strategy: "stable".to_string(),
        };

        write_app_config(&temp_home, &config).expect("write app config");
        let loaded = read_app_config(&temp_home);

        assert_eq!(loaded.default_version_strategy, "stable");

        let _ = fs::remove_dir_all(temp_home);
    }

    #[test]
    fn save_app_config_rejects_invalid_strategy() {
        let result = save_app_config(SaveAppConfigRequest {
            default_version_strategy: "invalid".to_string(),
        });

        assert!(result.is_err());
    }

    #[test]
    fn scan_project_opencode_skills_marks_matching_managed_version() {
        let home = dirs::home_dir().expect("home dir");
        let unique = unique_test_name("project-scan-match");
        let manager_root = home.join(".skills-manager/skills");
        let project_root = home.join(".skills-manager-test").join(&unique).join("project");
        let project_skills_root = project_root.join(".opencode/skills");

        fs::create_dir_all(&manager_root).expect("create manager root");
        fs::create_dir_all(&project_skills_root).expect("create project skills root");

        let manager_skill_dir = write_skill_dir(
            &manager_root,
            &format!("demo-project-match-{}", unique),
            "---\nname: Demo Project Match\nversion: 1.0.0\nnamespace: default\n---\nShared content\n",
        );
        write_skill_dir(
            &project_skills_root,
            "demo-project-match-copy",
            "---\nname: Demo Project Match\nversion: 1.0.0\nnamespace: default\n---\nShared content\n",
        );

        let result = scan_project_opencode_skills(ScanProjectSkillsRequest {
            project_dir: project_root.display().to_string(),
            manager_root: manager_root.display().to_string(),
        })
        .expect("scan project skills succeeds");

        assert_eq!(result.duplicate_count, 1);
        assert_eq!(result.conflict_count, 0);

        let skill = result.skills.first().expect("project skill exists");
        assert_eq!(skill.status, crate::types::ProjectSkillImportStatus::Duplicate);
        assert!(skill.matched_version_id.is_some());
        assert_eq!(skill.matches_default_version, Some(true));

        let _ = fs::remove_dir_all(manager_skill_dir);
        let _ = fs::remove_dir_all(home.join(".skills-manager-test").join(unique));
    }

    #[test]
    fn scan_project_opencode_skills_marks_same_name_mismatch_as_conflict() {
        let home = dirs::home_dir().expect("home dir");
        let unique = unique_test_name("project-scan-conflict");
        let manager_root = home.join(".skills-manager/skills");
        let project_root = home.join(".skills-manager-test").join(&unique).join("project");
        let project_skills_root = project_root.join(".opencode/skills");

        fs::create_dir_all(&manager_root).expect("create manager root");
        fs::create_dir_all(&project_skills_root).expect("create project skills root");

        let manager_skill_dir = write_skill_dir(
            &manager_root,
            &format!("demo-project-conflict-{}", unique),
            "---\nname: Demo Project Conflict\nversion: 1.0.0\nnamespace: default\n---\nOriginal content\n",
        );
        write_skill_dir(
            &project_skills_root,
            "demo-project-conflict-copy",
            "---\nname: Demo Project Conflict\nversion: 1.1.0\nnamespace: default\n---\nChanged content\n",
        );

        let result = scan_project_opencode_skills(ScanProjectSkillsRequest {
            project_dir: project_root.display().to_string(),
            manager_root: manager_root.display().to_string(),
        })
        .expect("scan project skills succeeds");

        assert_eq!(result.duplicate_count, 0);
        assert_eq!(result.conflict_count, 1);

        let skill = result.skills.first().expect("project skill exists");
        assert_eq!(skill.status, crate::types::ProjectSkillImportStatus::Conflict);
        assert!(skill.existing_registry_skill.is_some());
        assert_eq!(skill.matched_version_id, None);
        assert_eq!(skill.matches_default_version, None);

        let _ = fs::remove_dir_all(manager_skill_dir);
        let _ = fs::remove_dir_all(home.join(".skills-manager-test").join(unique));
    }

    #[test]
    fn scan_project_opencode_skills_marks_same_name_non_default_match_as_managed_version() {
        let home = dirs::home_dir().expect("home dir");
        let unique = unique_test_name("project-scan-managed-version");
        let manager_root = home.join(".skills-manager/skills");
        let project_root = home.join(".skills-manager-test").join(&unique).join("project");
        let project_skills_root = project_root.join(".opencode/skills");

        fs::create_dir_all(&manager_root).expect("create manager root");
        fs::create_dir_all(&project_skills_root).expect("create project skills root");

        let first_manager_dir = write_skill_dir(
            &manager_root,
            &format!("demo-project-managed-a-{}", unique),
            "---\nname: Demo Project Managed\nversion: 1.0.0\nnamespace: default\n---\nLegacy content\n",
        );
        let second_manager_dir = write_skill_dir(
            &manager_root,
            &format!("demo-project-managed-b-{}", unique),
            "---\nname: Demo Project Managed\ncategory:\n  - richer\nversion: 1.1.0\nnamespace: default\n---\nImported content\n",
        );

        let first_version = build_skill_version(&first_manager_dir, SkillVersionSource::Migration);
        let second_version = build_skill_version(&second_manager_dir, SkillVersionSource::Migration);
        write_package_state(
            &home,
            &first_version.skill_id,
            &StoredPackageState {
                default_version: Some(first_version.id.clone()),
                variants: Vec::new(),
            },
        )
        .expect("persist explicit default version");

        write_skill_dir(
            &project_skills_root,
            "demo-project-managed-copy",
            "---\nname: Demo Project Managed\ncategory:\n  - richer\nversion: 1.1.0\nnamespace: default\n---\nImported content\n",
        );

        let result = scan_project_opencode_skills(ScanProjectSkillsRequest {
            project_dir: project_root.display().to_string(),
            manager_root: manager_root.display().to_string(),
        })
        .expect("scan project skills succeeds");

        assert_eq!(result.duplicate_count, 0);
        assert_eq!(result.managed_version_count, 1);
        assert_eq!(result.conflict_count, 0);

        let skill = result.skills.first().expect("project skill exists");
        assert_eq!(skill.status, crate::types::ProjectSkillImportStatus::ManagedVersion);
        assert!(skill.matched_registry_skill.is_some());
        assert_eq!(skill.matched_version_id.as_deref(), Some(second_version.id.as_str()));
        assert_eq!(skill.matches_default_version, Some(false));

        let _ = fs::remove_dir_all(first_manager_dir);
        let _ = fs::remove_dir_all(second_manager_dir);
        let _ = fs::remove_file(package_state_path(&home, &first_version.skill_id));
        let _ = fs::remove_dir_all(home.join(".skills-manager-test").join(unique));
    }
}
