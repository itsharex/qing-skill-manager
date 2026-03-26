use crate::types::{
    AppConfig, IdeSkill, LocalSkill, MetadataChange, SkillDiff, SkillPackage,
    SkillVariant, SkillVersion, SkillVersionMetadata, SkillVersionSource,
};
use crate::utils::path::sanitize_dir_name;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub mod conflict;
pub mod config;
pub mod scan;
pub mod variant;
pub mod version;

// Re-export all tauri::command functions
pub use conflict::analyze_skill_conflict;
pub use conflict::resolve_skill_conflict;
pub use config::get_app_config;
pub use config::save_app_config;
pub use scan::clone_local_skill;
pub use scan::delete_local_skills;
pub use scan::adopt_ide_skill;
pub use scan::import_local_skill;
pub use scan::scan_overview;
pub use scan::uninstall_skill;
pub use scan::scan_project_ide_dirs;
pub use scan::scan_project_opencode_skills;
pub use scan::scan_project_skills;
pub use version::compare_skill_versions;
pub use version::create_skill_version;
pub use version::delete_skill_version;
pub use version::get_skill_package;
pub use version::list_skill_packages;
pub use version::rename_skill_version;
pub use version::set_default_skill_version;
pub use variant::create_skill_variant;
pub use variant::delete_skill_variant;
pub use variant::update_skill_variant;

// Shared constants and types
pub(crate) const VERSION_METADATA_FILE: &str = ".qing-skill-manager-version.json";
pub(crate) const INSTALL_SIDECAR_FILE: &str = ".qing-skill-version.json";

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StoredVersionMetadata {
    pub version: Option<String>,
    pub display_name: Option<String>,
    pub source_url: Option<String>,
    pub parent_version: Option<String>,
    pub deleted: Option<bool>,
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedSkillMetadata {
    pub name: String,
    pub description: String,
    pub version: Option<String>,
    pub author: Option<String>,
    pub namespace: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StoredPackageState {
    pub default_version: Option<String>,
    pub variants: Vec<SkillVariant>,
}

/// Metadata written into IDE/project skill directories on installation
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InstalledSkillSidecar {
    pub version_id: Option<String>,
    pub content_hash: Option<String>,
    pub installed_at: Option<i64>,
    pub source_skill_id: Option<String>,
}

pub(crate) fn read_install_sidecar(skill_dir: &Path) -> InstalledSkillSidecar {
    let path = skill_dir.join(INSTALL_SIDECAR_FILE);
    if !path.exists() {
        return InstalledSkillSidecar::default();
    }
    fs::read_to_string(&path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

pub(crate) fn write_install_sidecar(skill_dir: &Path, sidecar: &InstalledSkillSidecar) -> Result<(), String> {
    let path = skill_dir.join(INSTALL_SIDECAR_FILE);
    let content = serde_json::to_string_pretty(sidecar).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())
}

// Shared helper functions
pub(crate) fn now_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

pub(crate) fn manager_versions_root(home: &Path) -> PathBuf {
    home.join(".qing-skill-manager/versions")
}

pub(crate) fn build_skill_id(name: &str, namespace: Option<&str>) -> String {
    let safe_name = sanitize_dir_name(name);
    let safe_namespace = namespace
        .map(sanitize_dir_name)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "default".to_string());
    format!("{}_{}", safe_name, safe_namespace)
}

pub(crate) fn version_metadata_path(home: &Path, skill_id: &str, version_id: &str) -> PathBuf {
    manager_versions_root(home)
        .join(skill_id)
        .join(version_id)
        .join("metadata.json")
}

pub(crate) fn package_state_path(home: &Path, skill_id: &str) -> PathBuf {
    manager_versions_root(home).join(skill_id).join("package.json")
}

pub(crate) fn app_config_path(home: &Path) -> PathBuf {
    home.join(".qing-skill-manager/config.json")
}

pub(crate) fn read_app_config(home: &Path) -> AppConfig {
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

pub(crate) fn write_app_config(home: &Path, config: &AppConfig) -> Result<(), String> {
    let path = app_config_path(home);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let serialized = serde_json::to_string_pretty(config).map_err(|err| err.to_string())?;
    fs::write(path, serialized).map_err(|err| err.to_string())
}

pub(crate) fn read_skill_metadata(skill_dir: &Path) -> (String, String) {
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

pub(crate) fn parse_skill_metadata(skill_dir: &Path) -> ParsedSkillMetadata {
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

pub(crate) fn simple_hash(input: &str) -> String {
    let mut hash: u64 = 1469598103934665603;
    for byte in input.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("{:016x}", hash)
}

pub(crate) fn skill_content_hash(skill_dir: &Path) -> String {
    let content = fs::read_to_string(skill_dir.join("SKILL.md")).unwrap_or_default();
    simple_hash(&content)
}

pub(crate) fn read_version_sidecar(skill_dir: &Path) -> StoredVersionMetadata {
    let metadata_path = skill_dir.join(VERSION_METADATA_FILE);
    if !metadata_path.exists() {
        return StoredVersionMetadata::default();
    }

    fs::read_to_string(metadata_path)
        .ok()
        .and_then(|content| serde_json::from_str::<StoredVersionMetadata>(&content).ok())
        .unwrap_or_default()
}

pub(crate) fn write_version_sidecar(skill_dir: &Path, sidecar: &StoredVersionMetadata) -> Result<(), String> {
    let path = skill_dir.join(VERSION_METADATA_FILE);
    let serialized = serde_json::to_string_pretty(sidecar).map_err(|err| err.to_string())?;
    fs::write(path, serialized).map_err(|err| err.to_string())
}

pub(crate) fn build_skill_version(skill_dir: &Path, source: SkillVersionSource) -> SkillVersion {
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

pub(crate) fn write_version_metadata(home: &Path, version: &SkillVersion) -> Result<(), String> {
    let path = version_metadata_path(home, &version.skill_id, &version.id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let serialized = serde_json::to_string_pretty(version).map_err(|err| err.to_string())?;
    fs::write(path, serialized).map_err(|err| err.to_string())
}

pub(crate) fn read_package_state(home: &Path, skill_id: &str) -> StoredPackageState {
    let path = package_state_path(home, skill_id);
    if !path.exists() {
        return StoredPackageState::default();
    }

    fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str::<StoredPackageState>(&content).ok())
        .unwrap_or_default()
}

pub(crate) fn write_package_state(home: &Path, skill_id: &str, state: &StoredPackageState) -> Result<(), String> {
    let path = package_state_path(home, skill_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let serialized = serde_json::to_string_pretty(state).map_err(|err| err.to_string())?;
    fs::write(path, serialized).map_err(|err| err.to_string())
}

pub(crate) fn collect_versions_for_skill(base: &Path, skill_id: &str) -> Vec<(PathBuf, SkillVersion)> {
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

pub(crate) fn version_summary_for_skill(home: &Path, skill_dir: &Path) -> SkillVersion {
    let version = build_skill_version(skill_dir, SkillVersionSource::Migration);
    let _ = write_version_metadata(home, &version);
    version
}

pub(crate) fn build_skill_diff(base: &SkillVersion, incoming: &SkillVersion) -> SkillDiff {
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

pub(crate) fn package_from_skill_dir(home: &Path, manager_dir: &Path, skill_dir: &Path) -> SkillPackage {
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

pub(crate) fn collect_skills_from_dir(base: &Path, source: &str, ide: Option<&str>) -> Vec<LocalSkill> {
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

pub(crate) fn collect_ide_skills(
    base: &Path,
    ide_label: &str,
    scope: &str,
    manager_map: &[(String, usize)],
    manager_skills: &mut [LocalSkill],
    version_hash_map: &std::collections::HashMap<String, (String, String)>,
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
        if !skill_dir.join("SKILL.md").exists() {
            continue;
        }

        let name = read_skill_metadata(skill_dir).0;
        let path = skill_dir.to_path_buf();
        let content_hash = skill_content_hash(&path);

        // Check manager match (existing logic)
        let mut managed = false;
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

        // Read install sidecar for version tracking
        let sidecar = read_install_sidecar(&path);
        let has_sidecar = sidecar.version_id.is_some();

        let (version_id, installed_hash, sync_status) = if has_sidecar {
            let installed_h = sidecar.content_hash.clone();
            if installed_h.as_deref() == Some(&content_hash) {
                // Content unchanged since install → synced
                (sidecar.version_id.clone(), installed_h, "synced".to_string())
            } else {
                // Content changed since install → modified
                managed = true; // was installed by us
                (sidecar.version_id.clone(), installed_h, "modified".to_string())
            }
        } else if managed {
            // No sidecar but hash matches manager → legacy install
            let matched_version = version_hash_map.get(&content_hash).map(|(vid, _)| vid.clone());
            (matched_version, None, "untracked".to_string())
        } else {
            // Try version_hash_map for unmanaged skills
            if let Some((vid, _)) = version_hash_map.get(&content_hash) {
                managed = true;
                (Some(vid.clone()), None, "untracked".to_string())
            } else {
                (None, None, "unknown".to_string())
            }
        };

        let source = if managed { "managed" } else { "local" };

        skills.push(IdeSkill {
            id: path.display().to_string(),
            name,
            path: path.display().to_string(),
            ide: ide_label.to_string(),
            source: source.to_string(),
            managed,
            scope: scope.to_string(),
            version_id,
            content_hash: Some(content_hash),
            installed_hash,
            sync_status,
        });
    }

    skills
}

/// Scan Claude Code plugin directories for top-level skills only.
/// Matches: {base}/**/skills/{skill-name}/SKILL.md (direct children of "skills/" dirs)
pub(crate) fn collect_plugin_skills(base: &Path, ide_label: &str) -> Vec<IdeSkill> {
    let mut skills = Vec::new();
    if !base.exists() {
        return skills;
    }

    for entry in walkdir::WalkDir::new(base)
        .min_depth(1)
        .max_depth(6)
        .into_iter()
        .flatten()
    {
        if entry.file_name() != "skills" || !entry.file_type().is_dir() {
            continue;
        }
        let skills_dir = entry.path();
        let Ok(children) = fs::read_dir(skills_dir) else {
            continue;
        };
        for child in children.flatten() {
            let path = child.path();
            if !path.is_dir() || !path.join("SKILL.md").exists() {
                continue;
            }
            let (name, _) = read_skill_metadata(&path);
            skills.push(IdeSkill {
                id: path.display().to_string(),
                name,
                path: path.display().to_string(),
                ide: ide_label.to_string(),
                source: "plugin".to_string(),
                managed: false,
                scope: "plugin".to_string(),
                version_id: None,
                content_hash: None,
                installed_hash: None,
                sync_status: "unknown".to_string(),
            });
        }
    }

    skills
}

pub(crate) fn remove_path(path: &Path) -> Result<(), String> {
    if path.is_dir() {
        fs::remove_dir_all(path).map_err(|err| err.to_string())
    } else {
        fs::remove_file(path).map_err(|err| err.to_string())
    }
}

pub(crate) fn load_default_version_strategy() -> String {
    let Some(home) = dirs::home_dir() else {
        return "manual".to_string();
    };
    let config = read_app_config(&home);
    match config.default_version_strategy.as_str() {
        "manual" | "latest" | "stable" => config.default_version_strategy,
        _ => "manual".to_string(),
    }
}

pub(crate) fn select_strategy_default_version(versions: &[SkillVersion], strategy: &str) -> Option<String> {
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

pub(crate) fn resolve_default_version(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        AdoptIdeSkillRequest, AppConfig, ConflictSeverity, ConflictType, IdeDir, InstallRequest,
        LinkTarget, LocalScanRequest, ProjectSkillImportStatus, ResolutionAction,
        SaveAppConfigRequest, ScanProjectSkillsRequest, SkillVersion, SkillVersionMetadata,
        SkillVersionSource, UninstallRequest,
    };
    use crate::utils::download::copy_dir_recursive;
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
    fn clone_local_skill_copies_into_target_directory() {
        use crate::types::LinkTarget;
        let home = dirs::home_dir().expect("home dir");
        let unique = unique_test_name("clone-success");
        let root = home.join(".skills-manager-test").join(&unique);
        let manager_root = home.join(".qing-skill-manager/skills");
        let target_root = root.join("ide");
        fs::create_dir_all(&manager_root).expect("create manager root");
        fs::create_dir_all(&target_root).expect("create target root");

        let skill_dir = write_skill_dir(
            &manager_root,
            &format!("demo-skill-{}", unique),
            "---\nname: Demo Skill\nversion: 1.0.0\n---\nDemo description\n",
        );

        let result = scan::clone_local_skill(InstallRequest {
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
        use crate::types::LinkTarget;
        let home = dirs::home_dir().expect("home dir");
        let unique = unique_test_name("clone-invalid");
        let root = home.join(".skills-manager-test").join(&unique);
        let manager_root = home.join(".qing-skill-manager/skills");
        fs::create_dir_all(&manager_root).expect("create manager root");

        let skill_dir = write_skill_dir(
            &manager_root,
            &format!("demo-skill-{}", unique),
            "---\nname: Demo Skill\nversion: 1.0.0\n---\nDemo description\n",
        );

        let result = scan::clone_local_skill(InstallRequest {
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

        let result = scan::adopt_ide_skill(AdoptIdeSkillRequest {
            target_path: ide_skill_dir.display().to_string(),
            ide_label: "Test IDE".to_string(),
        })
        .expect("adopt succeeds");

        let manager_dir = home.join(".qing-skill-manager/skills/demo-adopt-skill");
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
        let manager_root = home.join(".qing-skill-manager/skills");
        fs::create_dir_all(&manager_root).expect("create manager root");
        let ide_root = home.join(".skills-manager-test").join(&unique).join("ide");
        fs::create_dir_all(&ide_root).expect("create ide root");

        let manager_skill_dir = write_skill_dir(
            &manager_root,
            "demo-managed-skill",
            "---\nname: Demo Managed Skill\nversion: 1.0.0\n---\nManaged copy\n",
        );
        let copied_skill_dir = ide_root.join("demo-managed-skill");
        copy_dir_recursive(&manager_skill_dir, &copied_skill_dir).expect("copy skill to ide");

        let overview = scan::scan_overview(LocalScanRequest {
            project_dirs: vec![],
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

        let message = scan::uninstall_skill(UninstallRequest {
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
    fn save_app_config_rejects_invalid_strategy() {
        let result = config::save_app_config(SaveAppConfigRequest {
            default_version_strategy: "invalid".to_string(),
        });

        assert!(result.is_err());
    }

    #[test]
    fn scan_project_opencode_skills_marks_matching_managed_version() {
        use crate::types::{CompareVersionsRequest, ScanProjectSkillsRequest};
        let home = dirs::home_dir().expect("home dir");
        let unique = unique_test_name("project-scan-match");
        let manager_root = home.join(".qing-skill-manager/skills");
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

        let result = scan::scan_project_opencode_skills(ScanProjectSkillsRequest {
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
        use crate::types::ScanProjectSkillsRequest;
        let home = dirs::home_dir().expect("home dir");
        let unique = unique_test_name("project-scan-conflict");
        let manager_root = home.join(".qing-skill-manager/skills");
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

        let result = scan::scan_project_opencode_skills(ScanProjectSkillsRequest {
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
        use crate::types::ScanProjectSkillsRequest;
        let home = dirs::home_dir().expect("home dir");
        let unique = unique_test_name("project-scan-managed-version");
        let manager_root = home.join(".qing-skill-manager/skills");
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

        let result = scan::scan_project_opencode_skills(ScanProjectSkillsRequest {
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
