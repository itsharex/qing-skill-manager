use super::{
    build_skill_diff, build_skill_version, collect_skills_from_dir,
};
use crate::types::{
    AnalyzeConflictRequest, ConflictAnalysis, ConflictResolution, ConflictSeverity,
    ConflictType, ResolveConflictRequest, ResolveConflictResult, ResolutionAction,
    ResolutionSuggestion, SkillDiff, SkillVersionSource,
};
use crate::utils::download::copy_dir_recursive;
use crate::utils::path::sanitize_dir_name;
use std::path::PathBuf;

#[tauri::command]
pub fn analyze_skill_conflict(request: AnalyzeConflictRequest) -> Result<ConflictAnalysis, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let manager_dir = home.join(".qing-skill-manager/skills");
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
            let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
            let target_dir = home.join(".qing-skill-manager/skills").join(&safe_name);

            // Remove existing directory content before overwriting
            if target_dir.exists() {
                std::fs::remove_dir_all(&target_dir)
                    .map_err(|e| format!("Failed to remove existing skill directory: {}", e))?;
            }
            std::fs::create_dir_all(&target_dir)
                .map_err(|e| format!("Failed to create skill directory: {}", e))?;

            copy_dir_recursive(&skill_path, &target_dir)
                .map_err(|e| format!("Failed to copy skill files (overwrite): {}", e))?;

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
            let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
            let target_dir = home.join(".qing-skill-manager/skills").join(&safe_name);

            std::fs::create_dir_all(&target_dir)
                .map_err(|e| format!("Failed to create coexist skill directory: {}", e))?;

            copy_dir_recursive(&skill_path, &target_dir)
                .map_err(|e| format!("Failed to copy skill files (coexist): {}", e))?;

            ResolveConflictResult {
                success: true,
                skill_id: Some(safe_name),
                action: "coexisted".to_string(),
            }
        }
    };

    Ok(result)
}

pub(crate) fn classify_conflict(
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
