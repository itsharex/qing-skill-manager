mod commands;
mod types;
mod utils;

use tauri::Manager;
use commands::market::{download_marketplace_skill, search_marketplaces, update_marketplace_skill};
use commands::skills::{
    adopt_ide_skill, analyze_skill_conflict, clone_local_skill, create_skill_variant, create_skill_version, delete_local_skills,
    delete_skill_variant, delete_skill_version, get_skill_package, import_local_skill,
    list_skill_packages, rename_skill_version, resolve_skill_conflict,
    scan_overview, scan_project_ide_dirs, scan_project_opencode_skills, scan_project_skills, compare_skill_versions,
    set_default_skill_version, uninstall_skill, update_skill_variant, get_app_config, save_app_config,
};

pub use crate::types::{
    AdoptIdeSkillRequest, AnalyzeConflictRequest, ConflictAnalysis, ConflictResolution,
    AppConfig, AppConfigResponse,
    CreateVariantRequest, CreateVariantResponse, CreateVersionRequest, CreateVersionResponse, DeleteLocalSkillRequest, DeleteVariantRequest,
    DeleteVersionRequest, DeleteVersionResponse, GetSkillPackageRequest, GetSkillPackageResponse,
    IdeDir, IdeSkill, ImportProjectSkillRequest, ImportRequest, InstallRequest, InstallResult,
    LinkTarget, ListSkillPackagesResponse, LocalScanRequest, LocalSkill, MarketStatus,
    MarketStatusType, Overview, ProjectIdeDir, ProjectScanRequest, ProjectScanResult,
    ProjectSkill, ProjectSkillImportStatus, ProjectSkillScanResult, RemoteSkill,
    RemoteSkillView, RemoteSkillsResponse, RemoteSkillsViewResponse, RenameVersionRequest,
    RenameVersionResponse, ResolveConflictRequest, ResolveConflictResult,
    SaveAppConfigRequest, ScanProjectSkillsRequest, SetDefaultVersionRequest, SkillDiff, SkillPackage,
    SkillPackageSummary, SkillVariant, SkillVersion, UninstallRequest, UpdateVariantRequest,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            search_marketplaces,
            download_marketplace_skill,
            update_marketplace_skill,
            clone_local_skill,
            scan_overview,
            uninstall_skill,
            import_local_skill,
            delete_local_skills,
            adopt_ide_skill,
            scan_project_ide_dirs,
            scan_project_opencode_skills,
            scan_project_skills,
            resolve_skill_conflict,
            analyze_skill_conflict,
            create_skill_version,
            compare_skill_versions,
            list_skill_packages,
            get_skill_package,
            get_app_config,
            save_app_config,
            rename_skill_version,
            delete_skill_version,
            set_default_skill_version,
            create_skill_variant,
            update_skill_variant,
            delete_skill_variant
        ]);

    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
        // When a second instance is started, focus the existing window
        let _ = app
            .get_webview_window("main")
            .expect("no main window")
            .set_focus();
    }));

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
