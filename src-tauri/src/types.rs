use serde::{Deserialize, Serialize};
use std::fmt;

/// Market connection status enum
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MarketStatusType {
    Online,
    Error,
    NeedsKey,
}

impl fmt::Display for MarketStatusType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MarketStatusType::Online => write!(f, "online"),
            MarketStatusType::Error => write!(f, "error"),
            MarketStatusType::NeedsKey => write!(f, "needs_key"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSkill {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub source_url: String,
    pub description: String,
    pub author: String,
    pub installs: u64,
    pub stars: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSkillsResponse {
    pub skills: Vec<RemoteSkill>,
    pub total: u64,
    pub limit: u64,
    pub offset: u64,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSkillView {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub source_url: String,
    pub description: String,
    pub author: String,
    pub installs: u64,
    pub stars: u64,
    pub market_id: String,
    pub market_label: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MarketStatus {
    pub id: String,
    pub name: String,
    pub status: MarketStatusType,
    pub error: Option<String>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSkillsViewResponse {
    pub skills: Vec<RemoteSkillView>,
    pub total: u64,
    pub limit: u64,
    pub offset: u64,
    pub market_statuses: Vec<MarketStatus>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LinkTarget {
    pub name: String,
    pub path: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstallResult {
    pub installed_path: String,
    pub installed: Vec<String>,
    pub skipped: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DownloadRequest {
    pub source_url: String,
    pub skill_name: String,
    pub install_base_dir: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DownloadResult {
    pub installed_path: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstallRequest {
    pub skill_path: String,
    pub skill_name: String,
    pub install_targets: Vec<LinkTarget>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LocalSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub path: String,
    pub source: String,
    pub ide: Option<String>,
    pub used_by: Vec<String>,
    pub version_count: usize,
    pub current_version: Option<SkillVersion>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LocalScanRequest {
    pub project_dirs: Vec<String>,
    pub ide_dirs: Vec<IdeDir>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IdeSkill {
    pub id: String,
    pub name: String,
    pub path: String,
    pub ide: String,
    pub source: String,
    pub managed: bool,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Overview {
    pub manager_skills: Vec<LocalSkill>,
    pub ide_skills: Vec<IdeSkill>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UninstallRequest {
    pub target_path: String,
    pub project_dir: Option<String>,
    pub ide_dirs: Vec<IdeDir>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IdeDir {
    pub label: String,
    pub relative_dir: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImportRequest {
    pub source_path: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteLocalSkillRequest {
    pub target_paths: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AdoptIdeSkillRequest {
    pub target_path: String,
    pub ide_label: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectScanRequest {
    pub project_dir: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectIdeDir {
    pub label: String,
    pub relative_dir: String,
    pub absolute_path: String,
    /// true if detected by IDE characteristic files, false if detected by existing skills directory
    pub inferred: bool,
}

/// Status classification for project-level skill import
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectSkillImportStatus {
    New,
    Duplicate,
    ManagedVersion,
    Conflict,
}

/// A skill discovered in a project directory
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub path: String,
    pub status: ProjectSkillImportStatus,
    pub existing_registry_skill: Option<LocalSkill>,
    pub matched_registry_skill: Option<LocalSkill>,
    pub current_version: Option<SkillVersion>,
    pub matched_version_id: Option<String>,
    pub matched_version_name: Option<String>,
    pub matches_default_version: Option<bool>,
}

/// Result of scanning a project for OpenCode skills
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSkillScanResult {
    pub project_path: String,
    pub skills: Vec<ProjectSkill>,
    pub new_count: u64,
    pub duplicate_count: u64,
    pub managed_version_count: u64,
    pub conflict_count: u64,
}

/// Conflict resolution action
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConflictResolution {
    Keep,
    Overwrite,
    Coexist,
}

/// Request to resolve a skill import conflict
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResolveConflictRequest {
    pub project_skill_path: String,
    pub resolution: ConflictResolution,
    pub coexist_name: Option<String>,
}

/// Result of resolving a skill import conflict
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResolveConflictResult {
    pub success: bool,
    pub skill_id: Option<String>,
    pub action: String,
}

/// Request to scan project for OpenCode skills
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ScanProjectSkillsRequest {
    pub project_dir: String,
    pub manager_root: String,
}

/// Request to import a project skill
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImportProjectSkillRequest {
    pub project_skill_path: String,
    pub manager_root: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectScanResult {
    pub project_dir: String,
    pub detected_ide_dirs: Vec<ProjectIdeDir>,
}

// ============================================================================
// Skill Version Management - New Types for Git-style Version Control
// ============================================================================

/// Source type for a skill version
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SkillVersionSource {
    Market,
    Project,
    Import,
    Clone,
    Migration,
}

/// A specific version of a skill
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillVersion {
    pub id: String,                    // Version unique ID (content_hash)
    pub skill_id: String,              // Skill package ID (name_namespace)
    pub version: String,               // Semantic version x.y.z
    pub display_name: String,          // Custom display name
    pub content_hash: String,          // SHA256 hash of content
    pub created_at: i64,               // Creation timestamp
    pub source: SkillVersionSource,    // Source of this version
    pub source_url: Option<String>,    // Source URL if from market
    pub parent_version: Option<String>, // Parent version ID for tracing
    pub is_active: bool,               // Whether this version is active
    pub metadata: SkillVersionMetadata,
}

/// Metadata for a skill version
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillVersionMetadata {
    pub name: String,
    pub description: String,
    pub author: Option<String>,
    pub namespace: Option<String>,
}

/// A variant (branch) of a skill
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillVariant {
    pub id: String,
    pub name: String,                  // Variant name (e.g., "stable", "dev")
    pub current_version: String,       // Version ID this variant points to
    pub created_at: i64,
    pub description: Option<String>,
}

/// A skill package containing all versions and variants
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillPackage {
    pub id: String,                    // name_namespace
    pub name: String,
    pub namespace: String,
    pub default_version: String,       // Default version ID
    pub default_version_source: String,
    pub versions: Vec<SkillVersion>,
    pub variants: Vec<SkillVariant>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Summary of a skill package (for list views)
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillPackageSummary {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub version_count: usize,
    pub variant_count: usize,
    pub latest_version: String,
    pub default_version: String,
    pub updated_at: i64,
}

/// Content difference between two versions
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillDiff {
    pub from_version: String,
    pub to_version: String,
    pub files_changed: Vec<String>,
    pub additions: usize,
    pub deletions: usize,
    pub content_diff: Option<String>, // Unified diff format
    pub metadata_changes: Vec<MetadataChange>,
    pub similarity_score: f64,        // 0.0 - 1.0
}

/// Metadata field change
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MetadataChange {
    pub field: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

/// Conflict analysis result
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConflictAnalysis {
    pub conflict_type: ConflictType,
    pub severity: ConflictSeverity,
    pub base_version: SkillVersion,
    pub incoming_version: SkillVersion,
    pub diff: SkillDiff,
    pub auto_resolvable: bool,
    pub suggestions: Vec<ResolutionSuggestion>,
}

/// Type of conflict
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConflictType {
    Identical,    // Completely identical versions
    Patch,        // Small patch (metadata changes)
    Minor,        // Minor changes (new features)
    Major,        // Major changes (breaking changes)
    Fork,         // Substantial divergence
}

/// Severity of conflict
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConflictSeverity {
    None,
    Minor,
    Major,
    Breaking,
}

/// Resolution suggestion
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResolutionSuggestion {
    pub action: ResolutionAction,
    pub description: String,
    pub confidence: f64,               // 0.0 - 1.0
}

/// Possible resolution actions
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionAction {
    FastForward,      // Simple fast-forward
    CreateVersion,    // Create as new version
    CreateVariant,    // Create as new variant
    Overwrite,        // Overwrite existing
    InteractiveMerge, // Manual merge required
}

/// Strategy for deleting a version
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeleteStrategy {
    Soft,     // Mark as deleted, keep files
    Hard,     // Completely delete
    Archive,  // Move to archive directory
}

// ============================================================================
// Request/Response Types for Version Management Commands
// ============================================================================

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateVersionRequest {
    pub skill_id: String,
    pub version: String,
    pub display_name: String,
    pub source_path: String,
    pub source: SkillVersionSource,
    pub source_url: Option<String>,
    pub parent_version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub default_version_strategy: String,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppConfigResponse {
    pub config: AppConfig,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveAppConfigRequest {
    pub default_version_strategy: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateVersionResponse {
    pub version: SkillVersion,
    pub installed_path: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CompareVersionsRequest {
    pub skill_id: String,
    pub from_version: String,
    pub to_version: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteVersionRequest {
    pub skill_id: String,
    pub version_id: String,
    pub strategy: DeleteStrategy,
    pub force: Option<bool>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteVersionResponse {
    pub success: bool,
    pub message: String,
    pub archived_path: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetDefaultVersionRequest {
    pub skill_id: String,
    pub version_id: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateVariantRequest {
    pub skill_id: String,
    pub name: String,
    pub version_id: String,
    pub description: Option<String>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateVariantResponse {
    pub variant: SkillVariant,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVariantRequest {
    pub skill_id: String,
    pub variant_id: String,
    pub new_name: Option<String>,
    pub new_version_id: Option<String>,
    pub new_description: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteVariantRequest {
    pub skill_id: String,
    pub variant_id: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeConflictRequest {
    pub skill_id: String,
    pub base_version_id: String,
    pub incoming_path: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListSkillPackagesResponse {
    pub packages: Vec<SkillPackageSummary>,
    pub total: usize,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetSkillPackageRequest {
    pub skill_id: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetSkillPackageResponse {
    pub package: SkillPackage,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RenameVersionRequest {
    pub skill_id: String,
    pub version_id: String,
    pub new_display_name: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RenameVersionResponse {
    pub success: bool,
    pub version: SkillVersion,
}
