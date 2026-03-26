import { computed, ref } from "vue";
import type {
  LocalSkill,
  IdeSkill,
  ProjectConfig,
  ProjectSkill,
  SkillPackage,
  PlatformFilterOption,
  LibrarySkill,
  LibrarySkillStatus,
  LibraryVersionSummary,
  LibraryProjectMapping,
  LibraryIdeInstallation
} from "./types";

export type TranslateFunction = (key: string, values?: Record<string, string | number>) => string;

export interface UseLibraryWorkspaceOptions {
  localSkills: { value: LocalSkill[] };
  ideSkills: { value: IdeSkill[] };
  projects: { value: ProjectConfig[] };
  ideOptions: { value: Array<{ id: string; label: string }> };
  projectSkillSnapshots: { value: Record<string, ProjectSkill[]> };
  currentSkillPackage: { value: SkillPackage | null };
  t: TranslateFunction;
}

export function useLibraryWorkspace(options: UseLibraryWorkspaceOptions) {
  const { localSkills, ideSkills, projects, ideOptions, projectSkillSnapshots, currentSkillPackage, t } = options;

  // ============================================================================
  // State
  // ============================================================================

  const platformFilter = ref<string | null>(null);
  const searchQuery = ref("");
  const statusFilter = ref<LibrarySkillStatus | null>(null);
  const selectedSkillId = ref<string | null>(null);
  const loading = ref(false);

  // ============================================================================
  // Computed: Platform Filter Options
  // ============================================================================

  const platformOptions = computed<PlatformFilterOption[]>(() => {
    const options: PlatformFilterOption[] = [];

    // Add "All" option
    const totalSkillCount = localSkills.value.length;
    options.push({
      id: "all",
      label: t("library.platformAll"),
      count: totalSkillCount,
      active: platformFilter.value === null || platformFilter.value === "all"
    });

    // Add IDE-specific options
    for (const ide of ideOptions.value) {
      const count = ideSkills.value.filter((skill) => skill.ide === ide.id).length;
      options.push({
        id: ide.id,
        label: ide.label,
        count,
        active: platformFilter.value === ide.id
      });
    }

    return options;
  });

  // ============================================================================
  // Computed: Library Skills with Rich Metadata
  // ============================================================================

  const librarySkills = computed<LibrarySkill[]>(() => {
    const repoSkills = localSkills.value.map((localSkill): LibrarySkill => {
      // Build version summaries from currentSkillPackage if available
      const versions: LibraryVersionSummary[] = buildVersionSummaries(localSkill);

      // Determine status
      const status = determineSkillStatus(localSkill, ideSkills.value);

      // Build IDE installations
      const installations = buildIdeInstallations(localSkill, ideSkills.value, ideOptions.value);

      // Build project mappings
      const projectMappings = buildProjectMappings(localSkill, projects.value, projectSkillSnapshots.value);

      // Populate version usage counts from installations and project mappings
      for (const vs of versions) {
        vs.ideCount = installations.filter((inst) => inst.versionId === vs.id && inst.scope === "global").length;
        vs.projectCount = projectMappings.filter((pm) => pm.versionId === vs.id).length;
      }

      // Get projects using this skill
      const usedByProjectIds = projectMappings
        .filter((pm) => pm.versionId !== null)
        .map((pm) => pm.projectId);

      // Display path: prefer project/IDE path over repo path
      const displayPath = installations[0]?.skillPath || localSkill.path;

      return {
        id: localSkill.id,
        name: localSkill.name,
        namespace: localSkill.currentVersion?.metadata?.namespace,
        description: localSkill.description,
        source: localSkill.source,
        path: localSkill.path,
        status,
        versionCount: versions.filter((v) => v.isActive).length || localSkill.versionCount,
        defaultVersion: versions.find((v) => v.isDefault) || versions[0] || null,
        versions,
        installations,
        projectMappings,
        usedByProjectIds,
        inRepo: true,
        skillScope: "repo" as const,
        displayPath,
        unmanagedSources: []
      };
    });

    // Add unmanaged IDE skills not in central repo
    // Match by name (case-insensitive) and also by directory name from path
    const managedNames = new Set(repoSkills.map((s) => s.name.toLowerCase()));
    for (const s of repoSkills) {
      const dirName = s.path.split("/").pop();
      if (dirName) managedNames.add(dirName.toLowerCase());
    }
    const unmanagedMap = new Map<string, LibrarySkill>();

    function getSourceLabel(ideSkill: IdeSkill): string {
      if (ideSkill.scope === "global") return `${ideSkill.ide} · ${t("ide.scopeGlobal")}`;
      // Find project name from path
      const project = projects.value.find((p) => ideSkill.path.startsWith(p.path));
      return project ? project.name : ideSkill.ide;
    }

    for (const ideSkill of ideSkills.value) {
      const ideSkillDirName = ideSkill.path.split("/").pop()?.toLowerCase() || "";
      if (ideSkill.managed || managedNames.has(ideSkill.name.toLowerCase()) || managedNames.has(ideSkillDirName) || ideSkill.scope === "plugin") {
        continue;
      }
      const scope = ideSkill.scope as "global" | "project";
      const sourceEntry = {
        ide: ideSkill.ide,
        scope,
        path: ideSkill.path,
        label: getSourceLabel(ideSkill),
        contentHash: ideSkill.contentHash
      };

      const existing = unmanagedMap.get(ideSkill.name);
      if (existing) {
        existing.unmanagedSources.push(sourceEntry);
      } else {
        unmanagedMap.set(ideSkill.name, {
          id: `unmanaged_${ideSkill.id}`,
          name: ideSkill.name,
          description: "",
          source: ideSkill.source,
          path: ideSkill.path,
          status: "unmanaged",
          versionCount: 0,
          defaultVersion: null,
          versions: [],
          installations: [],
          projectMappings: [],
          usedByProjectIds: [],
          inRepo: false,
          skillScope: scope,
          displayPath: ideSkill.path,
          unmanagedSources: [sourceEntry]
        });
      }
    }

    return [...repoSkills, ...unmanagedMap.values()];
  });

  // ============================================================================
  // Computed: Filtered Skills
  // ============================================================================

  const filteredSkills = computed<LibrarySkill[]>(() => {
    let result = librarySkills.value;

    // Apply platform filter
    if (platformFilter.value && platformFilter.value !== "all") {
      result = result.filter((skill) =>
        skill.installations.some((inst) => inst.ideId === platformFilter.value)
      );
    }

    // Apply search filter
    if (searchQuery.value.trim()) {
      const query = searchQuery.value.toLowerCase().trim();
      result = result.filter((skill) =>
        skill.name.toLowerCase().includes(query) ||
        skill.description.toLowerCase().includes(query) ||
        (skill.namespace?.toLowerCase().includes(query) ?? false)
      );
    }

    // Apply status filter
    if (statusFilter.value) {
      result = result.filter((skill) => skill.status === statusFilter.value);
    }

    return result;
  });

  // ============================================================================
  // Computed: Selected Skill Detail
  // ============================================================================

  const selectedSkill = computed<LibrarySkill | null>(() => {
    if (!selectedSkillId.value) return null;
    return librarySkills.value.find((skill) => skill.id === selectedSkillId.value) || null;
  });

  // ============================================================================
  // Actions
  // ============================================================================

  function selectSkill(skillId: string | null): void {
    selectedSkillId.value = skillId;
  }

  function setPlatformFilter(filterId: string | null): void {
    platformFilter.value = filterId === "all" ? null : filterId;
  }

  function setSearchQuery(query: string): void {
    searchQuery.value = query;
  }

  function setStatusFilter(status: LibrarySkillStatus | null): void {
    statusFilter.value = status;
  }

  function clearFilters(): void {
    platformFilter.value = null;
    searchQuery.value = "";
    statusFilter.value = null;
  }

  // ============================================================================
  // Helpers
  // ============================================================================

  function buildVersionSummaries(localSkill: LocalSkill): LibraryVersionSummary[] {
    // If we have currentSkillPackage loaded and it matches this skill, use it
    if (currentSkillPackage.value && currentSkillPackage.value.id === localSkill.id) {
      return currentSkillPackage.value.versions.map((version): LibraryVersionSummary => ({
        id: version.id,
        version: version.version,
        displayName: version.displayName,
        createdAt: version.createdAt,
        isDefault: version.id === currentSkillPackage.value!.defaultVersion,
        isActive: version.isActive,
        source: version.source,
        projectCount: 0,
        ideCount: 0,
        inRepo: true
      }));
    }

    // Fallback: create a single version summary from currentVersion
    if (localSkill.currentVersion) {
      return [{
        id: localSkill.currentVersion.id,
        version: localSkill.currentVersion.version,
        displayName: localSkill.currentVersion.displayName,
        createdAt: localSkill.currentVersion.createdAt,
        isDefault: true,
        isActive: localSkill.currentVersion.isActive,
        source: localSkill.currentVersion.source,
        projectCount: 0,
        ideCount: 0,
        inRepo: true
      }];
    }

    return [];
  }

  function determineSkillStatus(
    localSkill: LocalSkill,
    allIdeSkills: IdeSkill[]
  ): LibrarySkillStatus {
    const relatedIdeSkills = allIdeSkills.filter(
      (ideSkill) => ideSkill.name === localSkill.name
    );
    const managedIdeSkills = relatedIdeSkills.filter((s) => s.managed);

    if (managedIdeSkills.length === 0) {
      return "not-installed";
    }

    // Check for modifications (any managed installation locally modified)
    const hasModified = managedIdeSkills.some((s) => s.syncStatus === "modified");
    if (hasModified) {
      return "modified";
    }

    // Check for conflicts: unmanaged copies with same name
    const unmanagedCopies = relatedIdeSkills.filter((s) => !s.managed);
    if (unmanagedCopies.length > 0) {
      return "conflict";
    }

    // Check for outdated: compare version info from skill package
    if (currentSkillPackage.value && currentSkillPackage.value.id === localSkill.id) {
      const pkg = currentSkillPackage.value;
      const defaultVersion = pkg.versions.find((v) => v.id === pkg.defaultVersion);
      if (defaultVersion && localSkill.currentVersion) {
        if (localSkill.currentVersion.id !== defaultVersion.id && !localSkill.currentVersion.isActive) {
          return "outdated";
        }
      }
      if (pkg.versions.length > 1 && defaultVersion) {
        const latestVersion = pkg.versions[0];
        if (latestVersion.id !== defaultVersion.id && latestVersion.source === "market") {
          return "outdated";
        }
      }
    }

    return "installed";
  }

  function buildIdeInstallations(
    localSkill: LocalSkill,
    allIdeSkills: IdeSkill[],
    allIdeOptions: Array<{ id: string; label: string }>
  ): LibraryIdeInstallation[] {
    const installations: LibraryIdeInstallation[] = [];
    const seen = new Set<string>();

    for (const ideSkill of allIdeSkills) {
      if (ideSkill.name !== localSkill.name) continue;
      // Dedup by skillPath
      if (seen.has(ideSkill.path)) continue;
      seen.add(ideSkill.path);

      const ideOption = allIdeOptions.find((opt) => opt.id === ideSkill.ide);
      installations.push({
        ideId: ideSkill.ide,
        ideLabel: ideOption?.label || ideSkill.ide,
        skillPath: ideSkill.path,
        versionId: ideSkill.versionId,
        isManaged: ideSkill.managed,
        scope: ideSkill.scope,
        syncStatus: ideSkill.syncStatus
      });
    }

    return installations;
  }

  function buildProjectMappings(
    localSkill: LocalSkill,
    allProjects: ProjectConfig[],
    snapshots: Record<string, ProjectSkill[]>
  ): LibraryProjectMapping[] {
    return allProjects.map((project): LibraryProjectMapping => {
      const projectSkills = snapshots[project.id] || [];
      const matchingSkill = projectSkills.find((ps) => ps.name === localSkill.name);

      if (!matchingSkill) {
        return {
          projectId: project.id,
          projectName: project.name,
          projectPath: project.path,
          versionId: null,
          versionName: null,
          isDefaultVersion: false,
          ideTargets: project.ideTargets,
          status: "missing"
        };
      }

      const isDefaultVersion = matchingSkill.matchesDefaultVersion ?? false;
      const status: LibraryProjectMapping["status"] =
        matchingSkill.status === "conflict" ? "conflict" :
        matchingSkill.status === "duplicate" || matchingSkill.status === "managed_version" ? "synced" :
        "modified";

      return {
        projectId: project.id,
        projectName: project.name,
        projectPath: project.path,
        versionId: matchingSkill.matchedVersionId || matchingSkill.currentVersion?.id || null,
        versionName: matchingSkill.matchedVersionName || matchingSkill.currentVersion?.displayName || null,
        isDefaultVersion,
        ideTargets: project.ideTargets,
        status
      };
    });
  }

  // ============================================================================
  // Return
  // ============================================================================

  return {
    // State
    platformFilter,
    searchQuery,
    statusFilter,
    selectedSkillId,
    loading,

    // Computed
    platformOptions,
    librarySkills,
    filteredSkills,
    selectedSkill,

    // Actions
    selectSkill,
    setPlatformFilter,
    setSearchQuery,
    setStatusFilter,
    clearFilters
  };
}
