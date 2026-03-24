import { ref, onUnmounted } from "vue";
import type { ProjectConfig, ProjectSkill } from "./types";

const REFRESH_INTERVAL_MS = 30000;

export function useProjectSnapshots(
  options: {
    projects: { value: ProjectConfig[] };
    scanProjectSkills: (projectPath: string, opts?: { silent?: boolean }) => Promise<{ skills: ProjectSkill[] } | null>;
    onRefresh?: (snapshots: Record<string, ProjectSkill[]>) => void;
  }
) {
  const projectSkillSnapshots = ref<Record<string, ProjectSkill[]>>({});
  let projectSnapshotRefreshTimer: number | null = null;

  async function refreshProjectSkillSnapshots(): Promise<void> {
    const nextSnapshots: Record<string, ProjectSkill[]> = {};
    for (const project of options.projects.value) {
      const result = await options.scanProjectSkills(project.path, { silent: true });
      nextSnapshots[project.id] = result?.skills ?? [];
    }
    projectSkillSnapshots.value = nextSnapshots;
    options.onRefresh?.(nextSnapshots);
  }

  function restartProjectSnapshotRefreshLoop(intervalMs = REFRESH_INTERVAL_MS): void {
    if (projectSnapshotRefreshTimer !== null) {
      window.clearInterval(projectSnapshotRefreshTimer);
    }
    projectSnapshotRefreshTimer = window.setInterval(() => {
      void refreshProjectSkillSnapshots();
    }, intervalMs);
  }

  function stopProjectSnapshotRefreshLoop(): void {
    if (projectSnapshotRefreshTimer !== null) {
      window.clearInterval(projectSnapshotRefreshTimer);
      projectSnapshotRefreshTimer = null;
    }
  }

  onUnmounted(() => {
    stopProjectSnapshotRefreshLoop();
  });

  return {
    projectSkillSnapshots,
    refreshProjectSkillSnapshots,
    restartProjectSnapshotRefreshLoop,
    stopProjectSnapshotRefreshLoop
  };
}
