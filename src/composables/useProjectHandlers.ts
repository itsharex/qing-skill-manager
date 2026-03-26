import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import { buildProjectCloneTargetPath } from "./constants";
import { useToast } from "./useToast";
import { getErrorMessage } from "./utils";
import type { ProjectConfig, ProjectSkill, LocalSkill, ConflictAnalysis, ProjectIdeDir } from "./types";

export interface UseProjectHandlersOptions {
  projects: { value: ProjectConfig[] };
  selectedProjectId: { value: string | null };
  localSkills: { value: LocalSkill[] };
  configuringProject: { value: ProjectConfig | null };
  addProject: (path: string, name: string, ideTargets: string[]) => ProjectConfig | undefined;
  removeProject: (id: string) => void;
  updateProjectIdeTargets: (id: string, ideTargets: string[]) => void;
  updateDetectedIdeDirs: (
    id: string,
    dirs: ProjectIdeDir[]
  ) => void;
  scanProjectSkills: (
    path: string,
    options?: { silent?: boolean }
  ) => Promise<{ skills: ProjectSkill[] } | null>;
  scanLocalSkills: () => Promise<boolean>;
  refreshProjectSkillSnapshots: () => Promise<void>;
  analyzeConflict: (request: {
    skillId: string;
    baseVersionId: string;
    incomingPath: string;
  }) => Promise<ConflictAnalysis | null>;
  openConflictModal: (skill: ProjectSkill) => void;
  closeConflictModal: () => void;
  resolveConflict: (
    skill: ProjectSkill,
    resolution: "keep" | "overwrite" | "coexist",
    coexistName?: string
  ) => Promise<boolean>;
  openProjectAddModal: () => void;
  closeProjectAddModal: () => void;
  openProjectConfigModal: (project: ProjectConfig) => void;
  closeProjectConfigModal: () => void;
  openProjectExportModal: () => void;
  closeProjectExportModal: () => void;
  openProjectImportModal: (project: ProjectConfig) => void;
  closeProjectImportModal: () => void;
}

export function useProjectHandlers(options: UseProjectHandlersOptions) {
  const { t } = useI18n();
  const toast = useToast();

  const localBusy = ref(false);
  const localBusyText = ref("");

  const {
    projects,
    selectedProjectId,
    localSkills,
    configuringProject,
    addProject,
    removeProject,
    updateProjectIdeTargets,
    updateDetectedIdeDirs,
    scanProjectSkills,
    scanLocalSkills,
    refreshProjectSkillSnapshots,
    analyzeConflict,
    openConflictModal,
    closeConflictModal,
    resolveConflict,
    closeProjectAddModal,
    closeProjectConfigModal,
    openProjectExportModal,
    closeProjectExportModal,
    openProjectImportModal,
    closeProjectImportModal
  } = options;

  async function handleRemoveProject(projectId: string) {
    removeProject(projectId);
  }

  async function handleSelectProject(projectId: string | null) {
    selectedProjectId.value = projectId;
  }

  async function handleProjectAddConfirm(path: string, name: string) {
    try {
      const scanResult = (await invoke("scan_project_ide_dirs", {
        request: { projectDir: path }
      })) as {
        detectedIdeDirs: Array<{
          label: string;
          relativeDir: string;
          absolutePath: string;
          inferred: boolean;
        }>;
      };

      const autoIdeTargets = scanResult.detectedIdeDirs.map((d) => d.label);
      const project = addProject(path, name, autoIdeTargets);
      if (project) {
        updateDetectedIdeDirs(project.id, scanResult.detectedIdeDirs);
      }
      closeProjectAddModal();
    } catch (err) {
      console.error("Failed to scan project:", err);
    }
  }

  async function rescanAllProjectIdes() {
    for (const project of projects.value) {
      try {
        const scanResult = (await invoke("scan_project_ide_dirs", {
          request: { projectDir: project.path }
        })) as {
          detectedIdeDirs: Array<{
            label: string;
            relativeDir: string;
            absolutePath: string;
            inferred: boolean;
          }>;
        };

        updateDetectedIdeDirs(project.id, scanResult.detectedIdeDirs);

        // Add newly detected IDEs to targets (don't remove existing user choices)
        const newLabels = scanResult.detectedIdeDirs
          .map((d) => d.label)
          .filter((label) => !project.ideTargets.includes(label));
        if (newLabels.length > 0) {
          updateProjectIdeTargets(project.id, [...project.ideTargets, ...newLabels]);
        }
      } catch (err) {
        console.error(`Failed to rescan project ${project.name}:`, err);
      }
    }
  }

  async function handleProjectConfigSave(
    projectId: string,
    ideTargets: string[]
  ) {
    updateProjectIdeTargets(projectId, ideTargets);
    closeProjectConfigModal();
  }

  async function handleExportSkills(projectId: string) {
    const project = projects.value.find((item) => item.id === projectId);
    if (!project) {
      toast.error(t("errors.projectNotFound"));
      return;
    }

    const result = await scanProjectSkills(project.path);
    if (!result) return;

    if (result.skills.length === 0) {
      toast.info(t("projects.noSkillsFound"));
      return;
    }

    openProjectExportModal();
  }

  function handleImportSkills(projectId: string) {
    const project = projects.value.find((item) => item.id === projectId);
    if (!project) {
      toast.error(t("errors.projectNotFound"));
      return;
    }

    openProjectImportModal(project);
  }

  function handleLibraryCloneToProject(projectId: string) {
    const project = projects.value.find((item) => item.id === projectId);
    if (!project) {
      toast.error(t("errors.projectNotFound"));
      return;
    }

    openProjectImportModal(project);
  }

  async function handleConflictResolution(
    currentConflictSkill: ProjectSkill | null,
    resolution: "keep" | "overwrite" | "coexist",
    coexistName?: string
  ) {
    if (!currentConflictSkill) return;

    await resolveConflict(currentConflictSkill, resolution, coexistName);
    closeConflictModal();
    await scanLocalSkills();

    if (selectedProjectId.value) {
      const project = projects.value.find(
        (item) => item.id === selectedProjectId.value
      );
      if (project) {
        await scanProjectSkills(project.path);
      }
    }

    await refreshProjectSkillSnapshots();
  }

  async function handleImportSelected(skillPaths: string[]) {
    if (skillPaths.length === 0) return;

    localBusy.value = true;
    localBusyText.value = t("messages.importing");

    try {
      let successCount = 0;
      let failCount = 0;

      for (const path of skillPaths) {
        try {
          await invoke("import_local_skill", {
            request: { sourcePath: path }
          });
          successCount++;
        } catch {
          failCount++;
        }
      }

      if (successCount > 0) {
        toast.success(
          t("messages.imported", { success: successCount, failed: failCount })
        );
      } else {
        toast.error(t("errors.importFailed"));
      }

      closeProjectExportModal();
      await scanLocalSkills();
      await refreshProjectSkillSnapshots();
    } catch (err) {
      toast.error(getErrorMessage(err, t("errors.importFailed")));
    } finally {
      localBusy.value = false;
      localBusyText.value = "";
    }
  }

  async function handleResolveConflictFromImport(skill: ProjectSkill) {
    if (skill.existingRegistrySkill?.currentVersion && skill.currentVersion) {
      await analyzeConflict({
        skillId: skill.existingRegistrySkill.currentVersion.skillId,
        baseVersionId: skill.existingRegistrySkill.currentVersion.id,
        incomingPath: skill.path
      });
    }
    openConflictModal(skill);
  }

  async function handleCloneSkillsToProject(
    skillIds: string[],
    ideLabels: string[]
  ) {
    if (!configuringProject.value) return;

    localBusy.value = true;
    localBusyText.value = t("messages.cloningSkillsToProject");

    try {
      let successCount = 0;
      let failCount = 0;

      for (const skillId of skillIds) {
        const skill = localSkills.value.find((item) => item.id === skillId);
        if (!skill) continue;

        try {
          const installTargets = [];
          for (const ideLabel of ideLabels) {
            const projectPath = configuringProject.value!.path;
            const ideDir = buildProjectCloneTargetPath(projectPath, ideLabel);
            if (!ideDir) continue;
            installTargets.push({
              name: ideLabel,
              path: ideDir
            });
          }

          if (installTargets.length === 0) {
            failCount++;
            continue;
          }

          await invoke("clone_local_skill", {
            request: {
              skillPath: skill.path,
              skillName: skill.name,
              installTargets
            }
          });
          successCount++;
        } catch {
          failCount++;
        }
      }

      if (successCount > 0) {
        toast.success(
          t("messages.skillsClonedToProject", {
            success: successCount,
            failed: failCount
          })
        );
        closeProjectImportModal();
        await scanLocalSkills();
        await refreshProjectSkillSnapshots();
      } else {
        toast.error(t("errors.cloneToProjectFailed"));
      }
    } catch (err) {
      toast.error(getErrorMessage(err, t("errors.cloneToProjectFailed")));
    } finally {
      localBusy.value = false;
      localBusyText.value = "";
    }
  }

  return {
    localBusy,
    localBusyText,
    handleRemoveProject,
    handleSelectProject,
    handleProjectAddConfirm,
    handleProjectConfigSave,
    handleExportSkills,
    handleImportSkills,
    handleLibraryCloneToProject,
    handleConflictResolution,
    handleImportSelected,
    handleResolveConflictFromImport,
    handleCloneSkillsToProject,
    rescanAllProjectIdes
  };
}
