import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { homeDir, join } from "@tauri-apps/api/path";
import type { ProjectSkill, ProjectSkillScanResult, ConflictResolution, ResolveConflictResult } from "./types";
import { getErrorMessage } from "./utils";

export type ToastFunction = (message: string) => void;
export type ErrorToastFunction = (message: string) => void;
export type TranslateFunction = (key: string, values?: Record<string, string | number>) => string;

export function useProjectScan(
  onSuccess: ToastFunction,
  onError: ErrorToastFunction,
  t: TranslateFunction
) {
  const projectSkillScanResult = ref<ProjectSkillScanResult | null>(null);
  const showConflictModal = ref(false);
  const currentConflictSkill = ref<ProjectSkill | null>(null);
  const busy = ref(false);
  const busyText = ref("");

  async function buildInstallBaseDir(): Promise<string> {
    const home = await homeDir();
    return join(home, ".qing-skill-manager/skills");
  }

  async function scanProjectSkills(projectPath: string, options?: { silent?: boolean }): Promise<ProjectSkillScanResult | null> {
    const silent = options?.silent === true;
    if (!silent) {
      busy.value = true;
      busyText.value = t("messages.scanningProject");
    }

    try {
      const installBaseDir = await buildInstallBaseDir();
      const result = await invoke("scan_project_skills", {
        request: {
          projectDir: projectPath,
          managerRoot: installBaseDir
        }
      }) as ProjectSkillScanResult;
      projectSkillScanResult.value = result;
      return result;
    } catch (err) {
      if (!silent) {
        onError(getErrorMessage(err, t("errors.scanProjectFailed")));
      }
      return null;
    } finally {
      if (!silent) {
        busy.value = false;
        busyText.value = "";
      }
    }
  }

  async function resolveConflict(skill: ProjectSkill, resolution: ConflictResolution, coexistName?: string): Promise<boolean> {
    busy.value = true;
    busyText.value = t("messages.resolvingConflict");
    try {
      const result = await invoke("resolve_skill_conflict", {
        request: {
          projectSkillPath: skill.path,
          resolution,
          coexistName
        }
      }) as ResolveConflictResult;

      if (result.success) {
        onSuccess(t("messages.conflictResolved", { action: result.action }));
        if (projectSkillScanResult.value) {
          const skillIndex = projectSkillScanResult.value.skills.findIndex((item) => item.path === skill.path);
          if (skillIndex !== -1) {
            projectSkillScanResult.value.skills[skillIndex].status = "duplicate";
          }
        }
        return true;
      }

      onError(t("errors.resolveConflictFailed"));
      return false;
    } catch (err) {
      onError(getErrorMessage(err, t("errors.resolveConflictFailed")));
      return false;
    } finally {
      busy.value = false;
      busyText.value = "";
    }
  }

  function openConflictModal(skill: ProjectSkill): void {
    currentConflictSkill.value = skill;
    showConflictModal.value = true;
  }

  function closeConflictModal(): void {
    showConflictModal.value = false;
    currentConflictSkill.value = null;
  }

  return {
    projectSkillScanResult,
    showConflictModal,
    currentConflictSkill,
    busy,
    busyText,
    scanProjectSkills,
    resolveConflict,
    openConflictModal,
    closeConflictModal
  };
}
