use super::{
    build_skill_diff, build_skill_version, collect_skills_from_dir, now_timestamp,
    package_from_skill_dir, read_package_state, read_version_sidecar,
    write_package_state, write_version_metadata, write_version_sidecar,
    StoredVersionMetadata, VERSION_METADATA_FILE,
};
use crate::types::{
    CompareVersionsRequest, CreateVersionRequest, CreateVersionResponse,
    DeleteStrategy, DeleteVersionRequest, DeleteVersionResponse, GetSkillPackageRequest,
    GetSkillPackageResponse, ListSkillPackagesResponse, RenameVersionRequest,
    RenameVersionResponse, SetDefaultVersionRequest, SkillDiff, SkillPackageSummary,
    SkillVersion,
};
use crate::utils::download::copy_dir_recursive;
use crate::utils::path::sanitize_dir_name;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[tauri::command]
pub fn create_skill_version(request: CreateVersionRequest) -> Result<CreateVersionResponse, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let manager_dir = home.join(".qing-skill-manager/skills");
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
    let destination_path = manager_dir.join(&destination_dir_name);

    if destination_path.exists() {
        // If the existing directory is a soft-deleted version, remove it to allow reuse
        let existing_sidecar = read_version_sidecar(&destination_path);
        if existing_sidecar.deleted == Some(true) {
            fs::remove_dir_all(&destination_path).map_err(|err| {
                format!("Failed to remove soft-deleted version directory: {err}")
            })?;
        } else {
            return Err("A version with the same destination folder already exists".to_string());
        }
    }

    copy_dir_recursive(&source_path, &destination_path).map_err(|err| err.to_string())?;

    let sidecar = StoredVersionMetadata {
        version: Some(request.version.clone()),
        display_name: Some(request.display_name.clone()),
        source_url: request.source_url.clone(),
        parent_version: request.parent_version.clone().or(Some(reference_version.id)),
        deleted: Some(false),
        created_at: Some(now_timestamp()),
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
pub fn compare_skill_versions(request: CompareVersionsRequest) -> Result<SkillDiff, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let manager_dir = home.join(".qing-skill-manager/skills");
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
    let manager_dir = home.join(".qing-skill-manager/skills");
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
    let manager_dir = home.join(".qing-skill-manager/skills");

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
    use super::{read_version_sidecar, version_summary_for_skill};

    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let manager_dir = home.join(".qing-skill-manager/skills");
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
    use super::read_version_sidecar;

    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let manager_dir = home.join(".qing-skill-manager/skills");
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
            let archive_root = home.join(".qing-skill-manager/archive");
            fs::create_dir_all(&archive_root).map_err(|err| err.to_string())?;
            let dir_name = skill_path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("skill-version");
            let mut archive_path = archive_root.join(dir_name);
            // Avoid collision with existing archives
            if archive_path.exists() {
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                archive_path = archive_root.join(format!("{}-{}", dir_name, timestamp));
            }
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
