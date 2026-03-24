import { ref } from "vue";
import type { SkillVersion, SkillDiff, ProjectSkill } from "./types";

export function useVersionManagementState() {
  const comparingFromVersion = ref<SkillVersion | null>(null);
  const comparingToVersion = ref<SkillVersion | null>(null);
  const currentDiff = ref<SkillDiff | null>(null);

  const currentManagedSkillPath = ref("");
  const selectedCreateVersionSourcePath = ref("");
  const versionImportProjectId = ref<string | null>(null);
  const versionImportProjectSkills = ref<ProjectSkill[]>([]);
  const versionImportProjectSkillsLoading = ref(false);

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
