import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type {
  SkillPackage,
  SkillDiff,
  SkillVersion,
  ProjectSkill,
  ProjectConfig,
  ProjectSkillScanResult,
  ConflictAnalysis,
  CreateVersionRequest,
  CreateVersionResponse,
  CompareVersionsRequest,
  DeleteVersionRequest,
  DeleteVersionResponse,
  DeleteStrategy,
  CreateVariantRequest,
  CreateVariantResponse,
  UpdateVariantRequest,
  DeleteVariantRequest,
  SkillVariant,
  GetSkillPackageResponse,
  RenameVersionResponse,
  AnalyzeConflictRequest
} from "./types";
import { getErrorMessage } from "./utils";
import type { AppContext } from "./useAppContext";

export function useVersionManagement(
  ctx: AppContext
) {
  const currentSkillPackage = ref<SkillPackage | null>(null);
  const showVersionManagerModal = ref(false);
  const versionLoading = ref(false);
  const currentConflictAnalysis = ref<ConflictAnalysis | null>(null);
  const showVersionDiffModal = ref(false);
  const currentVersionDiff = ref<SkillDiff | null>(null);
  const busy = ref(false);
  const busyText = ref("");

  // Version comparison & import state (formerly useVersionManagementState)
  const comparingFromVersion = ref<SkillVersion | null>(null);
  const comparingToVersion = ref<SkillVersion | null>(null);
  const currentDiff = ref<SkillDiff | null>(null);
  const currentManagedSkillPath = ref("");
  const selectedCreateVersionSourcePath = ref("");
  const versionImportProjectId = ref<string | null>(null);
  const versionImportProjectSkills = ref<ProjectSkill[]>([]);
  const versionImportProjectSkillsLoading = ref(false);

  async function loadSkillPackage(skillId: string): Promise<SkillPackage | null> {
    versionLoading.value = true;
    try {
      const response = await invoke("get_skill_package", {
        request: { skillId }
      }) as GetSkillPackageResponse;
      currentSkillPackage.value = response.package;
      return response.package;
    } catch (err) {
      ctx.toast.error(getErrorMessage(err, ctx.t("errors.loadPackageFailed")));
      return null;
    } finally {
      versionLoading.value = false;
    }
  }

  async function compareVersions(skillId: string, fromVersion: string, toVersion: string): Promise<SkillDiff | null> {
    try {
      const response = await invoke("compare_skill_versions", {
        request: { skillId, fromVersion, toVersion } as CompareVersionsRequest
      }) as SkillDiff;
      currentVersionDiff.value = response;
      return response;
    } catch (err) {
      ctx.toast.error(getErrorMessage(err, ctx.t("errors.compareVersionsFailed")));
      return null;
    }
  }

  async function createVersion(request: CreateVersionRequest): Promise<CreateVersionResponse | null> {
    busy.value = true;
    busyText.value = ctx.t("messages.creatingVersion");
    try {
      const response = await invoke("create_skill_version", { request }) as CreateVersionResponse;
      if (currentSkillPackage.value?.id === request.skillId) {
        await loadSkillPackage(request.skillId);
      }
      await ctx.scanLocalSkills();
      ctx.toast.success(ctx.t("messages.versionCreated", { version: response.version.displayName }));
      return response;
    } catch (err) {
      ctx.toast.error(getErrorMessage(err, ctx.t("errors.createVersionFailed")));
      return null;
    } finally {
      busy.value = false;
      busyText.value = "";
    }
  }

  async function analyzeConflict(request: AnalyzeConflictRequest): Promise<ConflictAnalysis | null> {
    try {
      const response = await invoke("analyze_skill_conflict", { request }) as ConflictAnalysis;
      currentConflictAnalysis.value = response;
      return response;
    } catch (err) {
      ctx.toast.error(getErrorMessage(err, ctx.t("errors.analyzeConflictFailed")));
      return null;
    }
  }

  async function renameVersion(skillId: string, versionId: string, newDisplayName: string): Promise<boolean> {
    busy.value = true;
    busyText.value = ctx.t("messages.renamingVersion");
    try {
      const response = await invoke("rename_skill_version", {
        request: { skillId, versionId, newDisplayName }
      }) as RenameVersionResponse;
      ctx.toast.success(ctx.t("messages.versionRenamed", { name: response.version.displayName }));
      if (currentSkillPackage.value?.id === skillId) {
        await loadSkillPackage(skillId);
      }
      await ctx.scanLocalSkills();
      return true;
    } catch (err) {
      ctx.toast.error(getErrorMessage(err, ctx.t("errors.renameVersionFailed")));
      return false;
    } finally {
      busy.value = false;
      busyText.value = "";
    }
  }

  async function deleteVersion(skillId: string, versionId: string, strategy: DeleteStrategy, force = false): Promise<boolean> {
    busy.value = true;
    busyText.value = ctx.t("messages.deletingVersion");
    try {
      const response = await invoke("delete_skill_version", {
        request: { skillId, versionId, strategy, force }
      } as { request: DeleteVersionRequest }) as DeleteVersionResponse;
      if (response.success) {
        ctx.toast.success(ctx.t("messages.versionDeleted"));
        if (currentSkillPackage.value?.id === skillId) {
          await loadSkillPackage(skillId);
        }
        await ctx.scanLocalSkills();
        return true;
      }
      ctx.toast.error(response.message);
      return false;
    } catch (err) {
      ctx.toast.error(getErrorMessage(err, ctx.t("errors.deleteVersionFailed")));
      return false;
    } finally {
      busy.value = false;
      busyText.value = "";
    }
  }

  async function setDefaultVersion(skillId: string, versionId: string): Promise<boolean> {
    busy.value = true;
    busyText.value = ctx.t("messages.settingDefaultVersion");
    try {
      await invoke("set_default_skill_version", {
        request: { skillId, versionId }
      });
      ctx.toast.success(ctx.t("messages.defaultVersionSet"));
      if (currentSkillPackage.value?.id === skillId) {
        await loadSkillPackage(skillId);
      }
      await ctx.scanLocalSkills();
      return true;
    } catch (err) {
      ctx.toast.error(getErrorMessage(err, ctx.t("errors.setDefaultVersionFailed")));
      return false;
    } finally {
      busy.value = false;
      busyText.value = "";
    }
  }

  async function createVariant(request: CreateVariantRequest): Promise<SkillVariant | null> {
    busy.value = true;
    busyText.value = ctx.t("messages.creatingVariant");
    try {
      const response = await invoke("create_skill_variant", { request }) as CreateVariantResponse;
      if (currentSkillPackage.value?.id === request.skillId) {
        await loadSkillPackage(request.skillId);
      }
      ctx.toast.success(ctx.t("messages.variantCreated", { name: response.variant.name }));
      return response.variant;
    } catch (err) {
      ctx.toast.error(getErrorMessage(err, ctx.t("errors.createVariantFailed")));
      return null;
    } finally {
      busy.value = false;
      busyText.value = "";
    }
  }

  async function updateVariant(request: UpdateVariantRequest): Promise<SkillVariant | null> {
    busy.value = true;
    busyText.value = ctx.t("messages.updatingVariant");
    try {
      const variant = await invoke("update_skill_variant", { request }) as SkillVariant;
      if (currentSkillPackage.value?.id === request.skillId) {
        await loadSkillPackage(request.skillId);
      }
      ctx.toast.success(ctx.t("messages.variantUpdated", { name: variant.name }));
      return variant;
    } catch (err) {
      ctx.toast.error(getErrorMessage(err, ctx.t("errors.updateVariantFailed")));
      return null;
    } finally {
      busy.value = false;
      busyText.value = "";
    }
  }

  async function deleteVariant(request: DeleteVariantRequest): Promise<boolean> {
    busy.value = true;
    busyText.value = ctx.t("messages.deletingVariant");
    try {
      await invoke("delete_skill_variant", { request });
      if (currentSkillPackage.value?.id === request.skillId) {
        await loadSkillPackage(request.skillId);
      }
      ctx.toast.success(ctx.t("messages.variantDeleted"));
      return true;
    } catch (err) {
      ctx.toast.error(getErrorMessage(err, ctx.t("errors.deleteVariantFailed")));
      return false;
    } finally {
      busy.value = false;
      busyText.value = "";
    }
  }

  function openVersionManagerModal(skillId: string): void {
    showVersionManagerModal.value = true;
    void loadSkillPackage(skillId);
  }

  function closeVersionManagerModal(): void {
    showVersionManagerModal.value = false;
    currentSkillPackage.value = null;
  }

  function openVersionDiffModal(): void {
    showVersionDiffModal.value = true;
  }

  function closeVersionDiffModal(): void {
    showVersionDiffModal.value = false;
    currentVersionDiff.value = null;
  }

  function setComparisonVersions(from: SkillVersion | null, to: SkillVersion | null, diff: SkillDiff | null): void {
    comparingFromVersion.value = from;
    comparingToVersion.value = to;
    currentDiff.value = diff;
  }

  function setVersionImportProject(projectId: string | null, skills: ProjectSkill[] = [], loading = false): void {
    versionImportProjectId.value = projectId;
    versionImportProjectSkills.value = skills;
    versionImportProjectSkillsLoading.value = loading;
  }

  async function pickSourcePath(): Promise<void> {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const selected = await open({
      directory: true,
      multiple: false,
      title: ctx.t("version.pickSourcePathTitle")
    });

    if (!selected || Array.isArray(selected)) {
      return;
    }

    selectedCreateVersionSourcePath.value = selected;
  }

  async function pickVersionImportProject(
    projectId: string,
    projects: { value: ProjectConfig[] },
    scanProjectSkills: (projectPath: string, options?: { silent?: boolean }) => Promise<ProjectSkillScanResult | null>,
    projectSkillSnapshots: { value: Record<string, ProjectSkill[]> }
  ): Promise<void> {
    const project = projects.value.find((item) => item.id === projectId);
    setVersionImportProject(projectId, [], false);

    if (!project) {
      return;
    }

    versionImportProjectSkillsLoading.value = true;
    try {
      const result = await scanProjectSkills(project.path, { silent: true });
      setVersionImportProject(projectId, result?.skills ?? [], false);
      projectSkillSnapshots.value = {
        ...projectSkillSnapshots.value,
        [project.id]: result?.skills ?? []
      };
    } finally {
      versionImportProjectSkillsLoading.value = false;
    }
  }

  return {
    currentSkillPackage,
    showVersionManagerModal,
    versionLoading,
    currentConflictAnalysis,
    showVersionDiffModal,
    currentVersionDiff,
    busy,
    busyText,
    loadSkillPackage,
    compareVersions,
    createVersion,
    analyzeConflict,
    renameVersion,
    deleteVersion,
    setDefaultVersion,
    createVariant,
    updateVariant,
    deleteVariant,
    openVersionManagerModal,
    closeVersionManagerModal,
    openVersionDiffModal,
    closeVersionDiffModal,
    comparingFromVersion,
    comparingToVersion,
    currentDiff,
    currentManagedSkillPath,
    selectedCreateVersionSourcePath,
    versionImportProjectId,
    versionImportProjectSkills,
    versionImportProjectSkillsLoading,
    setComparisonVersions,
    setVersionImportProject,
    pickSourcePath,
    pickVersionImportProject
  };
}
