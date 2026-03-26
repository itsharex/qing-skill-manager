<script setup lang="ts">
import { computed, onMounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import { useSkillsManager } from "./composables/useSkillsManager";
import { usePreferences } from "./composables/usePreferences";
import { useProjectHandlers } from "./composables/useProjectHandlers";
import MarketPanel from "./components/MarketPanel.vue";
import LibraryWorkspace from "./components/library/LibraryWorkspace.vue";
import IdePanel from "./components/IdePanel.vue";
import SettingsPanel from "./components/SettingsPanel.vue";
import ProjectsPanel from "./components/ProjectsPanel.vue";
import InstallModal from "./components/InstallModal.vue";
import UninstallModal from "./components/UninstallModal.vue";
import LoadingOverlay from "./components/LoadingOverlay.vue";
import Toast from "./components/Toast.vue";
import ProjectAddModal from "./components/ProjectAddModal.vue";
import ProjectConfigModal from "./components/ProjectConfigModal.vue";
import ConflictResolutionModal from "./components/ConflictResolutionModal.vue";
import ProjectSkillImportModal from "./components/ProjectSkillImportModal.vue";
import ImportToProjectModal from "./components/ImportToProjectModal.vue";
import VersionManagerModal from "./components/VersionManagerModal.vue";
import VersionDiffModal from "./components/VersionDiffModal.vue";
import type { LocalSkill } from "./composables/types";

const { t } = useI18n();

const { theme, locale, toggleTheme, toggleLocale } = usePreferences();

const {
  activeTab,
  query,
  results,
  loading,
  installingId,
  updatingId,
  localSkills,
  localLoading,
  ideOptions,
  selectedIdeFilter,
  customIdeName,
  customIdeDir,
  customIdeOptions,
  filteredIdeSkills,
  showInstallModal,
  showUninstallModal,
  uninstallTargetName,
  uninstallMode,
  busy,
  busyText,
  hasMore,
  localSkillNameSet,
  searchMarketplace,
  downloadSkill,
  updateSkill,
  scanLocalSkills,
  openInstallModal,
  addCustomIde,
  removeCustomIde,
  openUninstallModal,
  openUninstallManyModal,
  openDeleteLocalModal,
  confirmInstallToIde,
  closeInstallModal,
  confirmUninstall,
  cancelUninstall,
  importLocalSkill,
  openSkillDirectory,
  adoptIdeSkill,
  adoptManyIdeSkills,
  marketConfigs,
  marketStatuses,
  enabledMarkets,
  saveMarketConfigs,
  downloadQueue,
  recentTaskStatus,
  retryDownload,
  removeFromQueue,
  projectSkillScanResult,
  showConflictModal,
  currentConflictSkill,
  scanProjectSkills,
  resolveConflict,
  openConflictModal,
  closeConflictModal,
  currentSkillPackage,
  showVersionManagerModal,
  versionLoading,
  currentConflictAnalysis,
  showVersionDiffModal,
  currentVersionDiff,
  analyzeConflict,
  loadSkillPackage,
  compareVersions,
  createVersion,
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
  librarySkills,
  projects,
  selectedProjectId,
  loadProjects,
  addProject,
  removeProject,
  updateProjectIdeTargets,
  updateDetectedIdeDirs,
  projectSkillSnapshots,
  refreshProjectSkillSnapshots,
  restartProjectSnapshotRefreshLoop,
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
} = useSkillsManager();

const {
  localBusy,
  localBusyText,
  showProjectAddModal,
  showProjectConfigModal,
  showProjectExportModal,
  showProjectImportModal,
  configuringProject,
  openProjectAddModal,
  closeProjectAddModal,
  openProjectConfigModal,
  closeProjectConfigModal,
  closeProjectExportModal,
  closeProjectImportModal,
  handleRemoveProject,
  handleSelectProject,
  handleProjectAddConfirm,
  handleProjectConfigSave,
  handleExportSkills,
  handleImportSkills,
  handleLibraryCloneToProject,
  handleConflictResolution: handleConflictResolutionRaw,
  handleImportSelected,
  handleResolveConflictFromImport,
  handleCloneSkillsToProject,
  rescanAllProjectIdes
} = useProjectHandlers({
  projects,
  selectedProjectId,
  localSkills,
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
  resolveConflict
});

const displayBusy = computed(() => localBusy.value || busy.value);
const displayBusyText = computed(() => localBusyText.value || busyText.value);

onMounted(() => {
  loadProjects();
  void refreshProjectSkillSnapshots();
  restartProjectSnapshotRefreshLoop();
  void rescanAllProjectIdes();
  // Periodic rescan every 5 minutes to detect new IDE usage
  setInterval(() => void rescanAllProjectIdes(), 5 * 60 * 1000);
});

watch(projects, () => {
  void refreshProjectSkillSnapshots();
  restartProjectSnapshotRefreshLoop();
}, { deep: true });

watch(activeTab, (tab) => {
  if (tab === "projects") {
    void refreshProjectSkillSnapshots();
  }
});

async function handleConflictResolution(resolution: "keep" | "overwrite" | "coexist", coexistName?: string) {
  await handleConflictResolutionRaw(currentConflictSkill.value, resolution, coexistName);
}

async function handleRegisterVersion(sourcePath: string, displayName: string, version: string) {
  if (!currentSkillPackage.value) return;
  const pkg = currentSkillPackage.value;
  // Add timestamp suffix to version to avoid ID collision with soft-deleted versions
  const uniqueLabel = `${version}-${Date.now().toString(36).slice(-4)}`;
  try {
    await createVersion({
      skillId: pkg.id,
      sourcePath,
      displayName,
      version: uniqueLabel,
      source: "import",
    });
  } catch (err) {
    console.error("Failed to register version:", err);
  }
}

async function handleAdoptToRepo(path: string) {
  const dirName = path.split("/").pop() || "";
  try {
    await invoke("import_local_skill", { request: { sourcePath: path } });
    await scanLocalSkills();
    // Find the newly imported skill and load its package
    const newSkill = localSkills.value.find((s) => s.name === dirName || s.path.endsWith(`/${dirName}`));
    if (newSkill?.currentVersion) {
      void loadSkillPackage(newSkill.currentVersion.skillId || newSkill.id);
    }
  } catch (err) {
    console.error("Failed to adopt skill:", err);
  }
}

async function handleAdoptManyToRepo(targets: Array<{ path: string; ideLabel: string }>) {
  localBusy.value = true;
  localBusyText.value = t("messages.adopting");
  let successCount = 0;
  let failCount = 0;
  try {
    for (const target of targets) {
      try {
        await invoke("adopt_ide_skill", {
          request: { targetPath: target.path, ideLabel: target.ideLabel || "IDE" }
        });
        successCount++;
      } catch (err) {
        failCount++;
        console.warn("Failed to adopt skill:", target.path, err);
      }
    }
    await scanLocalSkills();
  } finally {
    localBusy.value = false;
    localBusyText.value = "";
  }
}

async function handleLibraryUninstallSkill(path: string) {
  try {
    await invoke("uninstall_skill", {
      request: { targetPath: path, ideLabel: "", ideDirs: [], projectDir: null }
    });
    await scanLocalSkills();
  } catch (err) {
    console.error("Failed to uninstall skill:", err);
  }
}

function handleLibrarySelectSkill(skill: LocalSkill) {
  if (!skill.currentVersion) {
    // Unmanaged skill — clear previous package
    currentSkillPackage.value = null;
    return;
  }
  const skillId = skill.currentVersion.skillId || skill.id;
  void loadSkillPackage(skillId);
}

async function handleCompareVersions(fromVersionId: string, toVersionId: string) {
  if (!currentSkillPackage.value) return;

  const fromVersion = currentSkillPackage.value.versions.find((item) => item.id === fromVersionId) || null;
  const toVersion = currentSkillPackage.value.versions.find((item) => item.id === toVersionId) || null;
  const diff = await compareVersions(currentSkillPackage.value.id, fromVersionId, toVersionId);
  setComparisonVersions(fromVersion, toVersion, diff);
  openVersionDiffModal();
}

function handleOpenCreateVersionFromLibrary() {
  if (!currentSkillPackage.value) {
    return;
  }

  selectedCreateVersionSourcePath.value = currentManagedSkillPath.value || "";
  openVersionManagerModal(currentSkillPackage.value.id);
}

async function handleCreateVersion(version: string, displayName: string, sourcePath: string, parentVersion?: string) {
  if (!currentSkillPackage.value) return;
  await createVersion({
    skillId: currentSkillPackage.value.id,
    version,
    displayName,
    sourcePath,
    source: "import",
    parentVersion,
    sourceUrl: undefined
  });
}

async function handlePickSourcePath() {
  const { open } = await import("@tauri-apps/plugin-dialog");
  const selected = await open({
    directory: true,
    multiple: false,
    title: t("version.pickSourcePathTitle")
  });

  if (!selected || Array.isArray(selected)) {
    return;
  }

  selectedCreateVersionSourcePath.value = selected;
}

async function handlePickVersionImportProject(projectId: string) {
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


</script>

<template>
  <div class="app">
    <header class="header">
      <div class="header-spacer" />
      <div class="tabs">
        <button class="tab" :class="{ active: activeTab === 'local' }" @click="activeTab = 'local'">
          {{ t("app.tabs.local") }}
        </button>
        <button class="tab" :class="{ active: activeTab === 'market' }" @click="activeTab = 'market'">
          {{ t("app.tabs.market") }}
        </button>
        <button class="tab" :class="{ active: activeTab === 'ide' }" @click="activeTab = 'ide'">
          {{ t("app.tabs.ide") }}
        </button>
        <button class="tab" :class="{ active: activeTab === 'projects' }" @click="activeTab = 'projects'">
          {{ t("app.tabs.projects") }}
        </button>
        <button class="tab" :class="{ active: activeTab === 'settings' }" @click="activeTab = 'settings'">
          {{ t("app.tabs.settings") }}
        </button>
      </div>
      <div class="header-controls">
        <div class="control">
          <button
            class="icon-toggle"
            type="button"
            :aria-label="t('app.header.language')"
            :title="locale === 'zh-CN' ? '中文' : 'English'"
            @click="toggleLocale"
          >
            <span class="lang-badge">{{ locale === "zh-CN" ? "EN" : "中" }}</span>
          </button>
        </div>
        <div class="control">
          <button
            class="icon-toggle"
            type="button"
            :aria-label="t('app.header.theme')"
            :title="theme === 'light' ? t('app.header.themeLight') : t('app.header.themeDark')"
            @click="toggleTheme"
          >
            <svg v-if="theme === 'light'" class="icon" viewBox="0 0 24 24" aria-hidden="true">
              <path d="M12 4a1 1 0 011 1v1a1 1 0 11-2 0V5a1 1 0 011-1Zm6.36 2.64a1 1 0 010 1.41l-.7.7a1 1 0 11-1.41-1.41l.7-.7a1 1 0 011.41 0ZM20 11a1 1 0 010 2h-1a1 1 0 110-2h1Zm-8 2a3 3 0 100-6 3 3 0 000 6Zm-7 0a1 1 0 010-2H4a1 1 0 110-2h1a1 1 0 110 2H4a1 1 0 010 2Zm1.64-7.95a1 1 0 011.41 0l.7.7a1 1 0 11-1.41 1.41l-.7-.7a1 1 0 010-1.41ZM12 18a1 1 0 011 1v1a1 1 0 11-2 0v-1a1 1 0 011-1Zm7.07-1.07a1 1 0 010 1.41l-.7.7a1 1 0 11-1.41-1.41l.7-.7a1 1 0 011.41 0ZM6.34 16.93a1 1 0 011.41 0l.7.7a1 1 0 11-1.41 1.41l-.7-.7a1 1 0 010-1.41Z" fill="currentColor" />
            </svg>
            <svg v-else class="icon" viewBox="0 0 24 24" aria-hidden="true">
              <path d="M21 14.5A8.5 8.5 0 019.5 3a.9.9 0 00-.9.9 9.6 9.6 0 0010.5 10.5.9.9 0 00.9-.9Z" fill="currentColor" />
            </svg>
          </button>
        </div>
      </div>
    </header>

    <main class="content">
      <template v-if="activeTab === 'local'">
        <LibraryWorkspace
          :local-skills="localSkills"
          :local-loading="localLoading"
          :installing-id="installingId"
          :download-queue="downloadQueue"
          :ide-options="ideOptions"
          :skill-package="currentSkillPackage"
          :version-loading="versionLoading"
          :projects="projects"
          :library-skills="librarySkills"
          @install="openInstallModal"
          @install-many="openInstallModal"
          @delete-local="openDeleteLocalModal"
          @clone-to-project="handleLibraryCloneToProject"
          @open-dir="openSkillDirectory"
          @refresh="scanLocalSkills"
          @import="importLocalSkill"
          @retry-download="retryDownload"
          @remove-from-queue="removeFromQueue"
          @select-skill="handleLibrarySelectSkill"
          @adopt-to-repo="handleAdoptToRepo"
          @adopt-many-to-repo="handleAdoptManyToRepo"
          @register-version="handleRegisterVersion"
          @uninstall-skill="handleLibraryUninstallSkill"
          @compare-versions="handleCompareVersions"
          @create-version="handleOpenCreateVersionFromLibrary"
          @set-default-version="setDefaultVersion"
          @rename-version="renameVersion"
          @delete-version="(skillId, versionId) => deleteVersion(skillId, versionId, 'soft')"
        />
      </template>

      <template v-else-if="activeTab === 'market'">
        <MarketPanel
          v-model:query="query"
          :loading="loading"
          :results="results"
          :has-more="hasMore"
          :installing-id="installingId"
          :updating-id="updatingId"
          :local-skill-name-set="localSkillNameSet"
          :market-configs="marketConfigs"
          :market-statuses="marketStatuses"
          :enabled-markets="enabledMarkets"
          :download-queue="downloadQueue"
          :recent-task-status="recentTaskStatus"
          @search="searchMarketplace(true)"
          @refresh="searchMarketplace(true, true)"
          @loadMore="searchMarketplace(false)"
          @download="downloadSkill"
          @update="updateSkill"
          @saveConfigs="saveMarketConfigs"
        />
      </template>

      <template v-else-if="activeTab === 'ide'">
        <IdePanel
          :ide-options="ideOptions"
          :selected-ide-filter="selectedIdeFilter"
          :custom-ide-name="customIdeName"
          :custom-ide-dir="customIdeDir"
          :custom-ide-options="customIdeOptions"
          :filtered-ide-skills="filteredIdeSkills"
          :local-loading="localLoading"
          @update:selected-ide-filter="selectedIdeFilter = $event"
          @update:custom-ide-name="customIdeName = $event"
          @update:custom-ide-dir="customIdeDir = $event"
          @add-custom-ide="addCustomIde"
          @remove-custom-ide="removeCustomIde"
          @open-dir="openSkillDirectory"
          @adopt="adoptIdeSkill"
          @adopt-many="adoptManyIdeSkills"
          @uninstall="openUninstallModal"
          @uninstall-many="openUninstallManyModal"
        />
      </template>

      <template v-else-if="activeTab === 'projects'">
        <ProjectsPanel
          :projects="projects"
          :selected-project-id="selectedProjectId"
          :local-skills="localSkills"
          :ide-options="ideOptions"
          :project-skill-snapshots="projectSkillSnapshots"
          :local-loading="localLoading"
          @add-project="openProjectAddModal"
          @remove-project="handleRemoveProject"
          @select-project="handleSelectProject"
          @configure-project="(id) => { const project = projects.find((item) => item.id === id); if (project) openProjectConfigModal(project); }"
          @export-skills="handleExportSkills"
          @import-skills="handleImportSkills"
        />
      </template>

      <template v-else-if="activeTab === 'settings'">
        <SettingsPanel />
      </template>
    </main>

    <InstallModal :visible="showInstallModal" :ide-options="ideOptions" :projects="projects" @confirm="confirmInstallToIde" @cancel="closeInstallModal" />
    <UninstallModal :visible="showUninstallModal" :target-name="uninstallTargetName" :mode="uninstallMode" @confirm="confirmUninstall" @cancel="cancelUninstall" />
    <ProjectAddModal :visible="showProjectAddModal" @close="closeProjectAddModal" @confirm="handleProjectAddConfirm" />
    <ProjectConfigModal :visible="showProjectConfigModal" :project="configuringProject" :ide-options="ideOptions" @close="closeProjectConfigModal" @save="handleProjectConfigSave" />
    <ConflictResolutionModal :show="showConflictModal" :skill="currentConflictSkill" :conflict-analysis="currentConflictAnalysis" @close="closeConflictModal" @resolve="handleConflictResolution" />
    <ProjectSkillImportModal :show="showProjectExportModal" :scan-result="projectSkillScanResult" @close="closeProjectExportModal" @import="handleImportSelected" @resolve-conflict="handleResolveConflictFromImport" />
    <ImportToProjectModal :show="showProjectImportModal" :project="configuringProject" :local-skills="localSkills" @close="closeProjectImportModal" @clone="handleCloneSkillsToProject" />
    <VersionManagerModal
      :show="showVersionManagerModal"
      :skill-package="currentSkillPackage"
      :current-skill-path="currentManagedSkillPath"
      :selected-source-path="selectedCreateVersionSourcePath"
      :projects="projects"
      :project-skills="versionImportProjectSkills"
      :selected-project-id="versionImportProjectId"
      :project-skills-loading="versionImportProjectSkillsLoading"
      :loading="versionLoading"
      @close="closeVersionManagerModal"
      @rename="renameVersion"
      @delete="deleteVersion"
      @set-default="setDefaultVersion"
      @compare="handleCompareVersions"
      @create-version="handleCreateVersion"
      @pick-source-path="handlePickSourcePath"
      @pick-project="handlePickVersionImportProject"
      @create-variant="createVariant"
      @update-variant="updateVariant"
      @delete-variant="deleteVariant"
    />
    <VersionDiffModal :show="showVersionDiffModal" :diff="currentVersionDiff || currentDiff" :from-version="comparingFromVersion" :to-version="comparingToVersion" @close="closeVersionDiffModal" />
    <Toast />
    <LoadingOverlay :visible="displayBusy" :text="displayBusyText" />
  </div>
</template>

<style>
:root {
  --color-bg: #f5f5f7;
  --color-text: #1d1d1f;
  --color-muted: #6e6e73;
  --color-panel-bg: #ffffff;
  --color-panel-border: #d2d2d7;
  --color-panel-shadow: rgba(0, 0, 0, 0.04);
  --color-card-bg: #fafafa;
  --color-card-border: #e5e5ea;
  --color-input-bg: #ffffff;
  --color-input-border: #d2d2d7;
  --color-input-focus: #0071e3;
  --color-primary-bg: #0071e3;
  --color-primary-text: #ffffff;
  --color-chip-bg: #e8e8ed;
  --color-chip-border: #d2d2d7;
  --color-chip-text: #1d1d1f;
  --color-tabs-bg: #e8e8ed;
  --color-tab-text: #6e6e73;
  --color-tab-active-bg: #ffffff;
  --color-tab-active-text: #1d1d1f;
  --color-success-bg: #e3f9e5;
  --color-success-border: #b8e6bc;
  --color-success-text: #1e7e34;
  --color-error-bg: #fee2e2;
  --color-error-border: #fecaca;
  --color-error-text: #dc2626;
  --color-warning-bg: #fef3c7;
  --color-warning-border: #fde68a;
  --color-warning-text: #d97706;
  --color-progress-bg: #e5e5ea;
  --color-ghost-border: #d2d2d7;
  --color-ghost-text: #1d1d1f;
  --color-overlay-bg: rgba(0, 0, 0, 0.4);
}

[data-theme="dark"] {
  --color-bg: #1c1c1e;
  --color-text: #f5f5f7;
  --color-muted: #a1a1a6;
  --color-panel-bg: #2c2c2e;
  --color-panel-border: #3a3a3c;
  --color-panel-shadow: rgba(0, 0, 0, 0.3);
  --color-card-bg: #3a3a3c;
  --color-card-border: #48484a;
  --color-input-bg: #2c2c2e;
  --color-input-border: #48484a;
  --color-input-focus: #0a84ff;
  --color-primary-bg: #0a84ff;
  --color-primary-text: #ffffff;
  --color-chip-bg: #3a3a3c;
  --color-chip-border: #48484a;
  --color-chip-text: #f5f5f7;
  --color-tabs-bg: #3a3a3c;
  --color-tab-text: #a1a1a6;
  --color-tab-active-bg: #48484a;
  --color-tab-active-text: #f5f5f7;
  --color-success-bg: #1e3a2f;
  --color-success-border: #2d5a47;
  --color-success-text: #32d74b;
  --color-error-bg: #3d1f1f;
  --color-error-border: #5c3030;
  --color-error-text: #ff453a;
  --color-warning-bg: #3d3a1f;
  --color-warning-border: #5c5830;
  --color-warning-text: #ffd60a;
  --color-progress-bg: #48484a;
  --color-ghost-border: #48484a;
  --color-ghost-text: #f5f5f7;
  --color-overlay-bg: rgba(0, 0, 0, 0.6);


}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #app {
  height: 100%;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  background: var(--color-bg);
  color: var(--color-text);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

button {
  font-family: inherit;
}

::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: var(--color-chip-bg);
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: var(--color-muted);
}
</style>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
}

.header {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 12px 20px;
  background: var(--color-panel-bg);
  border-bottom: 1px solid var(--color-panel-border);
  flex-shrink: 0;
}

.header-spacer {
  flex: 1;
  min-width: 120px;
}

.tabs {
  display: flex;
  gap: 4px;
  padding: 4px;
  background: var(--color-tabs-bg);
  border-radius: 10px;
}

.tab {
  position: relative;
  padding: 8px 20px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--color-tab-text);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s ease, color 0.2s ease;
}

.tab:hover {
  color: var(--color-text);
}

.tab.active {
  background: var(--color-tab-active-bg);
  color: var(--color-tab-active-text);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
}

.header-controls {
  flex: 1;
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  min-width: 120px;
}

.control {
  display: flex;
  align-items: center;
}

.icon-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--color-muted);
  cursor: pointer;
  transition: background 0.2s ease, color 0.2s ease;
}

.icon-toggle:hover {
  background: var(--color-tabs-bg);
  color: var(--color-text);
}

.icon {
  width: 20px;
  height: 20px;
}

.lang-badge {
  font-size: 12px;
  font-weight: 600;
}

.content {
  flex: 1;
  min-height: 0;
  overflow: auto;
  position: relative;
}

/* Library workspace full-bleed layout */
.content > :deep(.library-workspace) {
  position: absolute;
  inset: 0;
}
</style>
