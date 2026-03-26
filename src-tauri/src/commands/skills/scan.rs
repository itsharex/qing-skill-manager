use super::{
    build_skill_version, collect_ide_skills, collect_skills_from_dir,
    read_skill_metadata, remove_path, skill_content_hash,
};
use crate::types::{
    AdoptIdeSkillRequest, DeleteLocalSkillRequest, ImportRequest, InstallRequest,
    InstallResult, LocalScanRequest, Overview, ProjectScanRequest, ProjectScanResult,
    ProjectIdeDir, ProjectSkill, ProjectSkillImportStatus, ProjectSkillScanResult,
    SkillVersionSource, ScanProjectSkillsRequest, UninstallRequest,
};
use crate::utils::download::copy_dir_recursive;
use crate::utils::path::{normalize_path, resolve_canonical, sanitize_dir_name};
use crate::utils::security::{is_absolute_ide_path, is_valid_ide_path};
use std::path::{Path, PathBuf};
use std::fs;

#[tauri::command]
pub fn clone_local_skill(request: InstallRequest) -> Result<InstallResult, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let normalized_home = normalize_path(&home);
    let manager_root_raw = home.join(".qing-skill-manager/skills");
    let manager_root =
        resolve_canonical(&manager_root_raw).unwrap_or_else(|| normalize_path(&manager_root_raw));

    let skill_path = PathBuf::from(&request.skill_path);
    let skill_canon = resolve_canonical(&skill_path)
        .ok_or_else(|| "Local skill path does not exist".to_string())?;
    if !skill_canon.starts_with(&manager_root) {
        return Err("Local skill path must stay inside Qing Skill Manager storage".to_string());
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

    let manager_dir = home.join(".qing-skill-manager/skills");
    let mut manager_skills = collect_skills_from_dir(&manager_dir, "manager", None);

    // Resolve IDE directories: absolute paths are used directly, relative paths are joined with home
    let ide_dirs: Vec<(String, PathBuf)> = if request.ide_dirs.is_empty() {
        vec![
            ("Claude Code".to_string(), home.join(".claude/skills")),
            ("Codex".to_string(), home.join(".codex/skills")),
            ("Cursor".to_string(), home.join(".cursor/skills")),
            ("OpenClaw".to_string(), home.join(".openclaw/skills")),
            ("OpenCode".to_string(), home.join(".config/opencode/skills")),
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

    let mut ide_skills = Vec::new();

    let mut manager_map = Vec::new();
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

    // Project-level IDE skill directories (e.g., project/.claude/skills)
    let project_ide_patterns = [
        ("Claude Code", ".claude/skills"),
        ("Codex", ".codex/skills"),
        ("Cursor", ".cursor/skills"),
        ("OpenClaw", ".openclaw/skills"),
        ("OpenCode", ".opencode/skills"),
    ];

    for project_path in &request.project_dirs {
        let base = PathBuf::from(project_path);
        if !base.exists() {
            continue;
        }
        for (label, relative_dir) in &project_ide_patterns {
            let project_dir = base.join(relative_dir);
            if project_dir.exists() && project_dir.is_dir() {
                ide_skills.extend(collect_ide_skills(
                    &project_dir,
                    label,
                    &manager_map,
                    &mut manager_skills,
                ));
            }
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
    let mut allowed_roots = vec![home.join(".qing-skill-manager/skills")];

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
        allowed_roots.push(base.join(".qing-skill-manager/skills"));
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
    let manager_dir = home.join(".qing-skill-manager/skills");

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
    let manager_root = home.join(".qing-skill-manager/skills");
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
    let manager_root = resolve_canonical(&home.join(".qing-skill-manager/skills"))
        .unwrap_or_else(|| normalize_path(&home.join(".qing-skill-manager/skills")));

    if request.target_paths.is_empty() {
        return Err("No skills were provided for deletion".to_string());
    }

    let mut deleted = 0usize;

    for raw_path in request.target_paths {
        let target = PathBuf::from(&raw_path);
        let canonical =
            resolve_canonical(&target).ok_or_else(|| "Target skill does not exist".to_string())?;
        if !canonical.starts_with(&manager_root) {
            return Err("Only Qing Skill Manager local skills can be deleted".to_string());
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

    // IDE characteristic files/directories for detection
    let ide_signatures: Vec<(&str, &str, Vec<&str>)> = vec![
        ("Claude Code", ".claude/skills", vec![".claude", "CLAUDE.md"]),
        ("Codex", ".codex/skills", vec![".codex"]),
        ("Cursor", ".cursor/skills", vec![".cursor", ".cursorrules", ".cursorignore"]),
        ("OpenClaw", ".openclaw/skills", vec![".openclaw"]),
        ("OpenCode", ".opencode/skills", vec![".opencode", ".config/opencode"]),
    ];

    let mut detected_ide_dirs = Vec::new();
    let mut detected_labels = std::collections::HashSet::new();

    for (label, skills_dir, signatures) in ide_signatures.iter() {
        let skills_path = project_dir.join(skills_dir);
        let has_skills_dir = skills_path.exists() && skills_path.is_dir();
        let has_signature = signatures.iter().any(|sig| project_dir.join(sig).exists());

        if has_skills_dir || has_signature {
            detected_ide_dirs.push(ProjectIdeDir {
                label: label.to_string(),
                relative_dir: skills_dir.to_string(),
                absolute_path: skills_path.display().to_string(),
                inferred: !has_skills_dir && has_signature,
            });
            detected_labels.insert(label.to_string());
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
    use super::version::get_skill_package;
    use crate::types::GetSkillPackageRequest;

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
    let mut existing_names: std::collections::HashMap<String, Vec<_>> = std::collections::HashMap::new();
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

        let mut matched_registry_skill: Option<_> = None;
        let mut matched_version: Option<(_, String)> = None;

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
