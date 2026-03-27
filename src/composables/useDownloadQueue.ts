import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { homeDir, join } from "@tauri-apps/api/path";
import type { RemoteSkill, DownloadTask } from "./types";
import type { AppContext } from "./useAppContext";

export function useDownloadQueue(
  ctx: AppContext
) {
  const downloadQueue = ref<DownloadTask[]>([]);
  const recentTaskStatus = ref<Record<string, "download" | "update">>({});
  let isProcessingQueue = false;
  const timers: number[] = [];

  async function buildInstallBaseDir(): Promise<string> {
    const home = await homeDir();
    return join(home, ".qing-skill-manager/skills");
  }

  function addToDownloadQueue(skill: RemoteSkill, action: "download" | "update" = "download"): void {
    if (downloadQueue.value.some((task) => task.id === skill.id)) {
      return;
    }
    downloadQueue.value.push({
      id: skill.id,
      name: skill.name,
      sourceUrl: skill.sourceUrl,
      action,
      status: "pending"
    });
    void processQueue();
  }

  async function processQueue(): Promise<void> {
    if (isProcessingQueue) return;
    isProcessingQueue = true;

    while (true) {
      const task = downloadQueue.value.find((item) => item.status === "pending");
      if (!task) break;

      task.status = "downloading";
      try {
        const installBaseDir = await buildInstallBaseDir();
        const command = task.action === "update"
          ? "update_marketplace_skill"
          : "download_marketplace_skill";

        await invoke(command, {
          request: {
            sourceUrl: task.sourceUrl,
            skillName: task.name,
            installBaseDir
          }
        });

        task.status = "done";
        recentTaskStatus.value = {
          ...recentTaskStatus.value,
          [task.id]: task.action
        };

        ctx.toast.success(
          task.action === "update"
            ? ctx.t("messages.updated", { path: task.name })
            : ctx.t("messages.downloaded", { path: task.name })
        );

        const timerId = window.setTimeout(() => {
          downloadQueue.value = downloadQueue.value.filter((item) => item.id !== task.id);
          const nextStatus = { ...recentTaskStatus.value };
          delete nextStatus[task.id];
          recentTaskStatus.value = nextStatus;
          void ctx.scanLocalSkills();
          const index = timers.indexOf(timerId);
          if (index > -1) timers.splice(index, 1);
        }, 2500);
        timers.push(timerId);
      } catch (err) {
        task.status = "error";
        task.error = err instanceof Error ? err.message : String(err);
      }
    }

    isProcessingQueue = false;
  }

  function removeFromQueue(taskId: string): void {
    downloadQueue.value = downloadQueue.value.filter((task) => task.id !== taskId);
  }

  function retryDownload(taskId: string): void {
    const task = downloadQueue.value.find((item) => item.id === taskId);
    if (task && task.status === "error") {
      task.status = "pending";
      task.error = undefined;
      void processQueue();
    }
  }

  async function downloadSkill(skill: RemoteSkill): Promise<void> {
    addToDownloadQueue(skill, "download");
  }

  async function updateSkill(skill: RemoteSkill): Promise<void> {
    addToDownloadQueue(skill, "update");
  }

  function cleanup(): void {
    timers.forEach((id) => clearTimeout(id));
  }

  return {
    downloadQueue,
    recentTaskStatus,
    addToDownloadQueue,
    removeFromQueue,
    retryDownload,
    downloadSkill,
    updateSkill,
    cleanup
  };
}
