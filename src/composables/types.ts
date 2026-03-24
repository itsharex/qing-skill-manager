/**
 * Remote skill from marketplace
 */
export type RemoteSkill = {
  id: string;
  name: string;
  namespace: string;
  sourceUrl: string;
  description: string;
  author: string;
  installs: number;
  stars: number;
  marketId: string;
  marketLabel: string;
};

/**
 * Market connection status
 */
export type MarketStatus = {
  id: string;
  name: string;
  status: "online" | "error" | "needs_key";
  error?: string;
};

/**
 * Result of skill installation
 */
export type InstallResult = {
  installedPath: string;
  installed: string[];
  skipped: string[];
};

/**
 * Local skill managed by skills-manager
 */
export type LocalSkill = {
  id: string;
  name: string;
  description: string;
  path: string;
  source: string;
  ide?: string;
  usedBy: string[];
  versionCount: number;
  currentVersion?: SkillVersion;
};

/**
 * Skill in IDE directory
 */
export type IdeSkill = {
  id: string;
  name: string;
  path: string;
  ide: string;
  source: string;
  managed: boolean;
};

/**
 * Overview of all skills
 */
export type Overview = {
  managerSkills: LocalSkill[];
  ideSkills: IdeSkill[];
};

/**
 * IDE configuration option
 */
export type IdeOption = {
  id: string;
  label: string;
  globalDir: string;
  projectDir?: string;
};

export type LinkTarget = {
  name: string;
  path: string;
};

/**
 * Download task in queue
 */
export type DownloadTask = {
  id: string;
  name: string;
  sourceUrl: string;
  action: "download" | "update";
  status: "pending" | "downloading" | "done" | "error";
  error?: string;
};

/**
 * IDE directory in a project
 */
export type ProjectIdeDir = {
  label: string;
  relativeDir: string;
  absolutePath: string;
};

/**
 * Project configuration
 */
export type ProjectConfig = {
  id: string;
  name: string;
  path: string;
  ideTargets: string[];
  detectedIdeDirs: ProjectIdeDir[];
};

/**
 * Status of a project-level skill import classification
 */
export type ProjectSkillImportStatus = "new" | "duplicate" | "managed_version" | "conflict";

/**
 * Project-level skill discovered during import scan
 */
export type ProjectSkill = {
  id: string;
  name: string;
  description: string;
  path: string;
  status: ProjectSkillImportStatus;
  existingRegistrySkill?: LocalSkill;
  matchedRegistrySkill?: LocalSkill;
  currentVersion?: SkillVersion;
  matchedVersionId?: string;
  matchedVersionName?: string;
  matchesDefaultVersion?: boolean;
};

/**
 * Result of scanning project for OpenCode skills
 */
export type ProjectSkillScanResult = {
  projectPath: string;
  skills: ProjectSkill[];
  newCount: number;
  duplicateCount: number;
  managedVersionCount: number;
  conflictCount: number;
};

/**
 * Conflict resolution action
 */
export type ConflictResolution = "keep" | "overwrite" | "coexist";

/**
 * Request to resolve a skill import conflict
 */
export type ResolveConflictRequest = {
  projectSkillPath: string;
  resolution: ConflictResolution;
  coexistName?: string;
};

/**
 * Result of resolving a skill import conflict
 */
export type ResolveConflictResult = {
  success: boolean;
  skillId?: string;
  action: string;
};

// ============================================================================
// Skill Version Management - Git-style Version Control Types
// ============================================================================

/**
 * Source type for a skill version
 */
export type SkillVersionSource = "market" | "project" | "import" | "clone" | "migration";

/**
 * A specific version of a skill
 */
export type SkillVersion = {
  id: string;
  skillId: string;
  version: string;
  displayName: string;
  contentHash: string;
  createdAt: number;
  source: SkillVersionSource;
  sourceUrl?: string;
  parentVersion?: string;
  isActive: boolean;
  metadata: SkillVersionMetadata;
};

/**
 * Metadata for a skill version
 */
export type SkillVersionMetadata = {
  name: string;
  description: string;
  author?: string;
  namespace?: string;
};

/**
 * A variant (branch) of a skill
 */
export type SkillVariant = {
  id: string;
  name: string;
  currentVersion: string;
  createdAt: number;
  description?: string;
};

/**
 * A skill package containing all versions and variants
 */
export type SkillPackage = {
  id: string;
  name: string;
  namespace: string;
  defaultVersion: string;
  defaultVersionSource?: "explicit" | "strategy";
  versions: SkillVersion[];
  variants: SkillVariant[];
  createdAt: number;
  updatedAt: number;
};

/**
 * Summary of a skill package (for list views)
 */
export type SkillPackageSummary = {
  id: string;
  name: string;
  namespace: string;
  versionCount: number;
  variantCount: number;
  latestVersion: string;
  defaultVersion: string;
  updatedAt: number;
};

/**
 * Content difference between two versions
 */
export type SkillDiff = {
  fromVersion: string;
  toVersion: string;
  filesChanged: string[];
  additions: number;
  deletions: number;
  contentDiff?: string;
  metadataChanges: MetadataChange[];
  similarityScore: number;
};

/**
 * Metadata field change
 */
export type MetadataChange = {
  field: string;
  oldValue?: string;
  newValue?: string;
};

/**
 * Conflict analysis result
 */
export type ConflictAnalysis = {
  conflictType: ConflictType;
  severity: ConflictSeverity;
  baseVersion: SkillVersion;
  incomingVersion: SkillVersion;
  diff: SkillDiff;
  autoResolvable: boolean;
  suggestions: ResolutionSuggestion[];
};

/**
 * Type of conflict
 */
export type ConflictType = "identical" | "patch" | "minor" | "major" | "fork";

/**
 * Severity of conflict
 */
export type ConflictSeverity = "none" | "minor" | "major" | "breaking";

/**
 * Resolution suggestion
 */
export type ResolutionSuggestion = {
  action: ResolutionAction;
  description: string;
  confidence: number;
};

/**
 * Possible resolution actions
 */
export type ResolutionAction = "fast_forward" | "create_version" | "create_variant" | "overwrite" | "interactive_merge";

/**
 * Strategy for deleting a version
 */
export type DeleteStrategy = "soft" | "hard" | "archive";

// ============================================================================
// Request/Response Types for Version Management Commands
// ============================================================================

export type CreateVersionRequest = {
  skillId: string;
  version: string;
  displayName: string;
  sourcePath: string;
  source: SkillVersionSource;
  sourceUrl?: string;
  parentVersion?: string;
};

export type CreateVersionResponse = {
  version: SkillVersion;
  installedPath: string;
};

export type CompareVersionsRequest = {
  skillId: string;
  fromVersion: string;
  toVersion: string;
};

export type DeleteVersionRequest = {
  skillId: string;
  versionId: string;
  strategy: DeleteStrategy;
  force?: boolean;
};

export type DeleteVersionResponse = {
  success: boolean;
  message: string;
  archivedPath?: string;
};

export type SetDefaultVersionRequest = {
  skillId: string;
  versionId: string;
};

export type CreateVariantRequest = {
  skillId: string;
  name: string;
  versionId: string;
  description?: string;
};

export type CreateVariantResponse = {
  variant: SkillVariant;
};

export type UpdateVariantRequest = {
  skillId: string;
  variantId: string;
  newName?: string;
  newVersionId?: string;
  newDescription?: string;
};

export type DeleteVariantRequest = {
  skillId: string;
  variantId: string;
};

export type AnalyzeConflictRequest = {
  skillId: string;
  baseVersionId: string;
  incomingPath: string;
};

export type ListSkillPackagesResponse = {
  packages: SkillPackageSummary[];
  total: number;
};

export type GetSkillPackageRequest = {
  skillId: string;
};

export type GetSkillPackageResponse = {
  package: SkillPackage;
};

export type RenameVersionRequest = {
  skillId: string;
  versionId: string;
  newDisplayName: string;
};

export type RenameVersionResponse = {
  success: boolean;
  version: SkillVersion;
};
