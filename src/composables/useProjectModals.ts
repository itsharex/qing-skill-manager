import { ref } from "vue";
import type { ProjectConfig } from "./types";

export function useProjectModals() {
  const showProjectAddModal = ref(false);
  const showProjectConfigModal = ref(false);
  const showProjectExportModal = ref(false);
  const showProjectImportModal = ref(false);

  const configuringProject = ref<ProjectConfig | null>(null);

  function openProjectAddModal(): void {
    showProjectAddModal.value = true;
  }

  function closeProjectAddModal(): void {
    showProjectAddModal.value = false;
  }

  function openProjectConfigModal(project: ProjectConfig): void {
    configuringProject.value = project;
    showProjectConfigModal.value = true;
  }

  function closeProjectConfigModal(): void {
    showProjectConfigModal.value = false;
    configuringProject.value = null;
  }

  function openProjectExportModal(): void {
    showProjectExportModal.value = true;
  }

  function closeProjectExportModal(): void {
    showProjectExportModal.value = false;
  }

  function openProjectImportModal(project: ProjectConfig): void {
    configuringProject.value = project;
    showProjectImportModal.value = true;
  }

  function closeProjectImportModal(): void {
    showProjectImportModal.value = false;
    configuringProject.value = null;
  }

  return {
    showProjectAddModal,
    showProjectConfigModal,
    showProjectExportModal,
    showProjectImportModal,
    configuringProject,
    openProjectAddModal,
    closeProjectAddModal,
    openProjectConfigModal,
    closeProjectConfigModal,
    openProjectExportModal,
    closeProjectExportModal,
    openProjectImportModal,
    closeProjectImportModal
  };
}
