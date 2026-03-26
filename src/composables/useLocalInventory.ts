import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { dirname } from "@tauri-apps/api/path";
import type { LocalSkill, IdeSkill, IdeOption } from "./types";
import { getErrorMessage, validateOverviewResponse } from "./utils";

export type ToastFunction = (message: string) => void;
export type ProgressFunction = (busy: boolean, text: string) => void;
export type TranslateFunction = (key: string, values?: Record<string, string | number>) => string;

export function useLocalInventory(
  ideOptions: { value: IdeOption[] },
  projectPaths: { value: string[] },
  onSuccess: ToastFunction,
  onError: ToastFunction,
  t: TranslateFunction
) {
  const localSkills = ref<LocalSkill[]>([]);
  const ideSkills = ref<IdeSkill[]>([]);
  const localLoading = ref(false);

  const localSkillNameSet = computed(() => {
    const set = new Set<string>();
    for (const skill of localSkills.value) {
      const key = skill.name.trim().toLowerCase();
      if (key) set.add(key);
    }
    return set;
  });

  async function scanLocalSkills(): Promise<boolean> {
    if (localLoading.value) return false;
    localLoading.value = true;

    try {
      const response = await invoke("scan_overview", {
        request: {
          projectDirs: projectPaths.value,
          ideDirs: ideOptions.value.map((item) => ({
            label: item.label,
            relativeDir: item.globalDir
          }))
        }
      });

      const validation = validateOverviewResponse(response);
      if (!validation.success) {
        console.error("[scanLocalSkills] Invalid response:", validation.error, response);
        onError(`${t("errors.scanFailed")}: ${validation.error}`);
        return false;
      }

      localSkills.value = validation.data.managerSkills as LocalSkill[];
      ideSkills.value = validation.data.ideSkills as IdeSkill[];
      return true;
    } catch (err) {
      console.error("[scanLocalSkills] Error:", err);
      onError(getErrorMessage(err, t("errors.scanFailed")));
      return false;
    } finally {
      localLoading.value = false;
    }
  }

  async function importLocalSkill(onProgress: ProgressFunction): Promise<void> {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        directory: true,
        multiple: true,
        title: t("local.selectSkillDir")
      });

      if (!selected) return;
      const paths = Array.isArray(selected) ? selected : [selected];
      if (paths.length === 0) return;

      onProgress(true, t("messages.importing"));

      let successCount = 0;
      let failCount = 0;
      let lastError = "";

      for (const path of paths) {
        try {
          await invoke("import_local_skill", {
            request: {
              sourcePath: path
            }
          });
          successCount++;
        } catch (err) {
          failCount++;
          lastError = err instanceof Error ? err.message : String(err);
        }
      }

      if (successCount > 0) {
        if (failCount > 0) {
          onError(t("messages.imported", { success: successCount, failed: failCount }));
        } else {
          onSuccess(t("messages.imported", { success: successCount, failed: 0 }));
        }
      } else {
        onError(
          t("messages.imported", { success: 0, failed: failCount }) +
          (paths.length === 1 ? `: ${lastError}` : "")
        );
      }

      const scanResult = await scanLocalSkills();
      if (!scanResult) {
        console.warn("[importLocalSkill] scanLocalSkills failed after import");
      }
    } catch (err) {
      onError(getErrorMessage(err, t("errors.importFailed")));
    } finally {
      onProgress(false, "");
    }
  }

  async function openSkillDirectory(path: string): Promise<void> {
    try {
      await revealItemInDir(path);
    } catch (err) {
      const message = getErrorMessage(err, t("errors.openDirFailed"));
      if (message.includes("os error 2") || message.toLowerCase().includes("cannot find the file")) {
        try {
          await revealItemInDir(await dirname(path));
          onError(t("errors.openDirFailed") + ": " + path);
          return;
        } catch {
        }
      }
      onError(message);
    }
  }

  return {
    localSkills,
    ideSkills,
    localLoading,
    localSkillNameSet,
    scanLocalSkills,
    importLocalSkill,
    openSkillDirectory
  };
}
