import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type {
  SkillPackage,
  SkillDiff,
  SkillVersion,
  ProjectSkill,
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

export type ToastFunction = (message: string) => void;
export type ErrorToastFunction = (message: string) => void;
export type ScanLocalSkillsFunction = () => Promise<boolean>;
export type TranslateFunction = (key: string, values?: Record<string, string | number>) => string;

export function useVersionManagement(
  onSuccess: ToastFunction,
  onError: ErrorToastFunction,
  scanLocalSkills: ScanLocalSkillsFunction,
  t: TranslateFunction
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
      onError(getErrorMessage(err, t("errors.loadPackageFailed")));
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
      onError(getErrorMessage(err, t("errors.compareVersionsFailed")));
      return null;
    }
  }

  async function createVersion(request: CreateVersionRequest): Promise<CreateVersionResponse | null> {
    busy.value = true;
    busyText.value = t("messages.creatingVersion");
    try {
      const response = await invoke("create_skill_version", { request }) as CreateVersionResponse;
      if (currentSkillPackage.value?.id === request.skillId) {
        await loadSkillPackage(request.skillId);
      }
      await scanLocalSkills();
      onSuccess(t("messages.versionCreated", { version: response.version.displayName }));
      return response;
    } catch (err) {
      onError(getErrorMessage(err, t("errors.createVersionFailed")));
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
      onError(getErrorMessage(err, t("errors.analyzeConflictFailed")));
      return null;
    }
  }

  async function renameVersion(skillId: string, versionId: string, newDisplayName: string): Promise<boolean> {
    busy.value = true;
    busyText.value = t("messages.renamingVersion");
    try {
      const response = await invoke("rename_skill_version", {
        request: { skillId, versionId, newDisplayName }
      }) as RenameVersionResponse;
      onSuccess(t("messages.versionRenamed", { name: response.version.displayName }));
      if (currentSkillPackage.value?.id === skillId) {
        await loadSkillPackage(skillId);
      }
      await scanLocalSkills();
      return true;
    } catch (err) {
      onError(getErrorMessage(err, t("errors.renameVersionFailed")));
      return false;
    } finally {
      busy.value = false;
      busyText.value = "";
    }
  }

  async function deleteVersion(skillId: string, versionId: string, strategy: DeleteStrategy, force = false): Promise<boolean> {
    busy.value = true;
    busyText.value = t("messages.deletingVersion");
    try {
      const response = await invoke("delete_skill_version", {
        request: { skillId, versionId, strategy, force }
      } as { request: DeleteVersionRequest }) as DeleteVersionResponse;
      if (response.success) {
        onSuccess(t("messages.versionDeleted"));
        if (currentSkillPackage.value?.id === skillId) {
          await loadSkillPackage(skillId);
        }
        await scanLocalSkills();
        return true;
      }
      onError(response.message);
      return false;
    } catch (err) {
      onError(getErrorMessage(err, t("errors.deleteVersionFailed")));
      return false;
    } finally {
      busy.value = false;
      busyText.value = "";
    }
  }

  async function setDefaultVersion(skillId: string, versionId: string): Promise<boolean> {
    busy.value = true;
    busyText.value = t("messages.settingDefaultVersion");
    try {
      await invoke("set_default_skill_version", {
        request: { skillId, versionId }
      });
      onSuccess(t("messages.defaultVersionSet"));
      if (currentSkillPackage.value?.id === skillId) {
        await loadSkillPackage(skillId);
      }
      await scanLocalSkills();
      return true;
    } catch (err) {
      onError(getErrorMessage(err, t("errors.setDefaultVersionFailed")));
      return false;
    } finally {
      busy.value = false;
      busyText.value = "";
    }
  }

  async function createVariant(request: CreateVariantRequest): Promise<SkillVariant | null> {
    busy.value = true;
    busyText.value = t("messages.creatingVariant");
    try {
      const response = await invoke("create_skill_variant", { request }) as CreateVariantResponse;
      if (currentSkillPackage.value?.id === request.skillId) {
        await loadSkillPackage(request.skillId);
      }
      onSuccess(t("messages.variantCreated", { name: response.variant.name }));
      return response.variant;
    } catch (err) {
      onError(getErrorMessage(err, t("errors.createVariantFailed")));
      return null;
    } finally {
      busy.value = false;
      busyText.value = "";
    }
  }

  async function updateVariant(request: UpdateVariantRequest): Promise<SkillVariant | null> {
    busy.value = true;
    busyText.value = t("messages.updatingVariant");
    try {
      const variant = await invoke("update_skill_variant", { request }) as SkillVariant;
      if (currentSkillPackage.value?.id === request.skillId) {
        await loadSkillPackage(request.skillId);
      }
      onSuccess(t("messages.variantUpdated", { name: variant.name }));
      return variant;
    } catch (err) {
      onError(getErrorMessage(err, t("errors.updateVariantFailed")));
      return null;
    } finally {
      busy.value = false;
      busyText.value = "";
    }
  }

  async function deleteVariant(request: DeleteVariantRequest): Promise<boolean> {
    busy.value = true;
    busyText.value = t("messages.deletingVariant");
    try {
      await invoke("delete_skill_variant", { request });
      if (currentSkillPackage.value?.id === request.skillId) {
        await loadSkillPackage(request.skillId);
      }
      onSuccess(t("messages.variantDeleted"));
      return true;
    } catch (err) {
      onError(getErrorMessage(err, t("errors.deleteVariantFailed")));
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
    setVersionImportProject
  };
}
