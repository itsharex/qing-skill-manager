import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { LocalSkill, IdeOption } from "./types";
import { getErrorMessage } from "./utils";
import type { AppContext } from "./useAppContext";

export function useUninstallActions(
  ideOptions: { value: IdeOption[] },
  ctx: AppContext,
  projectPaths?: { value: string[] }
) {
  const showUninstallModal = ref(false);
  const uninstallTargetPath = ref("");
  const uninstallTargetName = ref("");
  const uninstallTargetPaths = ref<string[]>([]);
  const uninstallMode = ref<"ide" | "local">("ide");
  const busy = ref(false);
  const busyText = ref("");

  function openUninstallModal(targetPath: string): void {
    uninstallMode.value = "ide";
    uninstallTargetPath.value = targetPath;
    uninstallTargetPaths.value = [targetPath];
    uninstallTargetName.value = targetPath.split(/[\\/]/).pop() || targetPath;
    showUninstallModal.value = true;
  }

  function openUninstallManyModal(paths: string[]): void {
    if (paths.length === 0) return;
    uninstallMode.value = "ide";
    uninstallTargetPath.value = "";
    uninstallTargetPaths.value = paths;
    uninstallTargetName.value = ctx.t("ide.uninstallSelectedCount", { count: paths.length });
    showUninstallModal.value = true;
  }

  function openDeleteLocalModal(targets: LocalSkill[]): void {
    uninstallMode.value = "local";
    uninstallTargetPath.value = "";
    uninstallTargetPaths.value = targets.map((skill) => skill.path);
    uninstallTargetName.value =
      targets.length === 1 ? targets[0].name : ctx.t("local.deleteSelectedCount", { count: targets.length });
    showUninstallModal.value = true;
  }

  async function confirmUninstall(): Promise<void> {
    busy.value = true;
    busyText.value = uninstallMode.value === "local" ? ctx.t("messages.deleting") : ctx.t("messages.uninstalling");
    try {
      if (uninstallMode.value === "local") {
        const message = (await invoke("delete_local_skills", {
          request: {
            targetPaths: uninstallTargetPaths.value
          }
        })) as string;
        ctx.toast.success(message);
      } else {
        let successCount = 0;
        let failCount = 0;
        const failedPaths: string[] = [];

        for (const targetPath of uninstallTargetPaths.value) {
          try {
            // Determine project dir for this path
            let projectDir: string | null = null;
            if (projectPaths?.value) {
              projectDir = projectPaths.value.find(p => targetPath.startsWith(p)) ?? null;
            }
            await invoke("uninstall_skill", {
              request: {
                targetPath,
                projectDir,
                ideDirs: ideOptions.value.map((item) => ({
                  label: item.label,
                  relativeDir: item.globalDir
                }))
              }
            });
            successCount++;
          } catch {
            failCount++;
            failedPaths.push(targetPath);
          }
        }

        if (successCount > 0 && failCount === 0) {
          ctx.toast.success(ctx.t("messages.uninstalledCount", { count: successCount }));
        } else if (successCount > 0 && failCount > 0) {
          ctx.toast.error(ctx.t("messages.uninstalledPartial", { success: successCount, failed: failCount }));
          if (failedPaths.length > 0) {
            console.warn("[confirmUninstall] Failed paths:", failedPaths);
          }
        } else {
          ctx.toast.error(ctx.t("errors.uninstallFailed"));
        }
      }

      const scanResult = await ctx.scanLocalSkills();
      if (!scanResult) {
        console.warn("[confirmUninstall] scanLocalSkills failed after uninstall");
      }
    } catch (err) {
      ctx.toast.error(
        getErrorMessage(
          err,
          uninstallMode.value === "local" ? ctx.t("errors.deleteFailed") : ctx.t("errors.uninstallFailed")
        )
      );
    } finally {
      showUninstallModal.value = false;
      uninstallTargetPath.value = "";
      uninstallTargetName.value = "";
      uninstallTargetPaths.value = [];
      busy.value = false;
      busyText.value = "";
    }
  }

  function cancelUninstall(): void {
    showUninstallModal.value = false;
    uninstallTargetPath.value = "";
    uninstallTargetName.value = "";
    uninstallTargetPaths.value = [];
  }

  async function uninstallFromLibrary(path: string): Promise<void> {
    try {
      await invoke("uninstall_skill", {
        request: { targetPath: path, ideLabel: "", ideDirs: [], projectDir: null }
      });
      await ctx.scanLocalSkills();
    } catch (err) {
      console.error("Failed to uninstall skill:", err);
    }
  }

  return {
    showUninstallModal,
    uninstallTargetPath,
    uninstallTargetName,
    uninstallTargetPaths,
    uninstallMode,
    busy,
    busyText,
    openUninstallModal,
    openUninstallManyModal,
    openDeleteLocalModal,
    confirmUninstall,
    cancelUninstall,
    uninstallFromLibrary
  };
}
