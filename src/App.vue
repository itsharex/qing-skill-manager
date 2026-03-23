<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { i18n, supportedLocales, type SupportedLocale } from "./i18n";
import { useSkillsManager } from "./composables/useSkillsManager";
import { useUpdateStore } from "./composables/useUpdateStore";
import { useProjectConfig } from "./composables/useProjectConfig";
import { useToast } from "./composables/useToast";
import { getErrorMessage } from "./composables/utils";
import { invoke } from "@tauri-apps/api/core";
import MarketPanel from "./components/MarketPanel.vue";
import LocalPanel from "./components/LocalPanel.vue";
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
import type { ProjectSkill, LocalSkill, SkillDiff, SkillVersion } from "./composables/types";

const { t } = useI18n();

// Mark components as used for template
void [ProjectsPanel, ProjectAddModal, ProjectConfigModal, ProjectSkillImportModal, ConflictResolutionModal, VersionManagerModal, VersionDiffModal];

const localeKey = "skillsManager.locale";
const themeKey = "skillsManager.theme";

const theme = ref<"light" | "dark">("light");
const locale = ref<SupportedLocale>("zh-CN");

const applyTheme = (next: "light" | "dark") => {
  document.documentElement.setAttribute("data-theme", next);
};

const loadLocale = (): SupportedLocale => {
  const stored = localStorage.getItem(localeKey) as SupportedLocale | null;
  if (stored && supportedLocales.includes(stored)) return stored;
  const browser = navigator.language.startsWith("zh") ? "zh-CN" : "en-US";
  return browser as SupportedLocale;
};

const loadTheme = (): "light" | "dark" => {
  const stored = localStorage.getItem(themeKey);
  if (stored === "dark" || stored === "light") return stored;
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
};

onMounted(() => {
  locale.value = loadLocale();
  theme.value = loadTheme();
  i18n.global.locale.value = locale.value;
  applyTheme(theme.value);

  // Check for updates on startup
  checkOnStartup();
  
  // Load projects
  loadProjects();
});

watch(locale, (next) => {
  i18n.global.locale.value = next;
  localStorage.setItem(localeKey, next);
});

watch(theme, (next) => {
  applyTheme(next);
  localStorage.setItem(themeKey, next);
});

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
  closeVersionDiffModal
} = useSkillsManager();

// Update store for startup check and badge
const { updateAvailable, checkOnStartup } = useUpdateStore();

// Toast
const toast = useToast();

// Project management
const {
  projects,
  selectedProjectId,
  selectedProject,
  loadProjects,
  addProject,
  removeProject,
  updateProjectIdeTargets,
  updateDetectedIdeDirs
} = useProjectConfig();

const showProjectAddModal = ref(false);
const showProjectConfigModal = ref(false);
const configuringProject = ref<typeof selectedProject.value>(null);
const showProjectExportModal = ref(false);
const showProjectImportModal = ref(false);

async function handleAddProject() {
  showProjectAddModal.value = true;
}

async function handleRemoveProject(projectId: string) {
  removeProject(projectId);
}

async function handleSelectProject(projectId: string | null) {
  selectedProjectId.value = projectId;
}

async function handleConfigureProject(projectId: string) {
  configuringProject.value = projects.value.find((p) => p.id === projectId) || null;
  showProjectConfigModal.value = true;
}

async function handleProjectAddConfirm(path: string, name: string) {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const scanResult = await invoke("scan_project_ide_dirs", {
      request: { projectDir: path }
    }) as { detectedIdeDirs: Array<{ label: string; relativeDir: string; absolutePath: string }> };

    const project = addProject(path, name, []);
    if (project) {
      updateDetectedIdeDirs(project.id, scanResult.detectedIdeDirs);
    }
    showProjectAddModal.value = false;
  } catch (err) {
    console.error("Failed to scan project:", err);
  }
}

async function handleProjectConfigSave(projectId: string, ideTargets: string[]) {
  updateProjectIdeTargets(projectId, ideTargets);
  showProjectConfigModal.value = false;
  configuringProject.value = null;
}

async function handleExportSkills(projectId: string) {
  const project = projects.value.find((p) => p.id === projectId);
  if (!project) {
    toast.error(t("errors.projectNotFound"));
    return;
  }

  // Scan project for OpenCode skills
  const result = await scanProjectSkills(project.path);
  if (!result) return;

  if (result.skills.length === 0) {
    toast.info(t("projects.noSkillsFound"));
    return;
  }

  // Show export modal with scan results
  showProjectExportModal.value = true;
}

function handleImportSkills(projectId: string) {
  const project = projects.value.find((p) => p.id === projectId);
  if (!project) {
    toast.error(t("errors.projectNotFound"));
    return;
  }

  configuringProject.value = project;
  showProjectImportModal.value = true;
}

async function handleConflictResolution(resolution: "keep" | "overwrite" | "coexist", coexistName?: string) {
  if (!currentConflictSkill.value) return;

  await resolveConflict(currentConflictSkill.value, resolution, coexistName);
  closeConflictModal();

  // Refresh local skills after resolution
  await scanLocalSkills();

  // Re-scan project skills to update the import modal
  if (selectedProjectId.value) {
    const project = projects.value.find((p) => p.id === selectedProjectId.value);
    if (project) {
      await scanProjectSkills(project.path);
    }
  }
}

async function handleImportSelected(skillPaths: string[]) {
  if (skillPaths.length === 0) return;

  busy.value = true;
  busyText.value = t("messages.importing");

  try {
    let successCount = 0;
    let failCount = 0;

    for (const path of skillPaths) {
      try {
        await invoke("import_local_skill", {
          request: {
            sourcePath: path
          }
        });
        successCount++;
      } catch {
        failCount++;
      }
    }

    if (successCount > 0) {
      toast.success(t("messages.imported", { success: successCount, failed: failCount }));
    } else {
      toast.error(t("errors.importFailed"));
    }

    showProjectImportModal.value = false;
    await scanLocalSkills();
  } catch (err) {
    toast.error(getErrorMessage(err, t("errors.importFailed")));
  } finally {
    busy.value = false;
    busyText.value = "";
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

// Version management handlers
function handleManageVersions(skill: LocalSkill) {
  currentManagedSkillPath.value = skill.path;
  selectedCreateVersionSourcePath.value = "";
  openVersionManagerModal(skill.currentVersion?.skillId || skill.id);
}

const comparingFromVersion = ref<SkillVersion | null>(null);
const comparingToVersion = ref<SkillVersion | null>(null);
const currentDiff = ref<SkillDiff | null>(null);
const currentManagedSkillPath = ref("");
const selectedCreateVersionSourcePath = ref("");

async function handleCompareVersions(fromVersionId: string, toVersionId: string) {
  if (!currentSkillPackage.value) return;

  const fromVersion = currentSkillPackage.value.versions.find(v => v.id === fromVersionId) || null;
  const toVersion = currentSkillPackage.value.versions.find(v => v.id === toVersionId) || null;

  comparingFromVersion.value = fromVersion;
  comparingToVersion.value = toVersion;

  currentDiff.value = await compareVersions(currentSkillPackage.value.id, fromVersionId, toVersionId);
  openVersionDiffModal();
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

async function handleRenameVersion(versionId: string, newName: string) {
  if (!currentSkillPackage.value) return;
  await renameVersion(currentSkillPackage.value.id, versionId, newName);
}

async function handleDeleteVersion(versionId: string, strategy: "soft" | "archive" | "hard", force: boolean) {
  if (!currentSkillPackage.value) return;
  await deleteVersion(currentSkillPackage.value.id, versionId, strategy, force);
}

async function handleSetDefaultVersion(versionId: string) {
  if (!currentSkillPackage.value) return;
  await setDefaultVersion(currentSkillPackage.value.id, versionId);
}

async function handleCreateVariant(versionId: string, name: string, description?: string) {
  if (!currentSkillPackage.value) return;
  await createVariant({
    skillId: currentSkillPackage.value.id,
    versionId,
    name,
    description
  });
}

async function handleDeleteVariant(variantId: string) {
  if (!currentSkillPackage.value) return;
  await deleteVariant({
    skillId: currentSkillPackage.value.id,
    variantId
  });
}

async function handleUpdateVariant(
  variantId: string,
  newName?: string,
  newVersionId?: string,
  newDescription?: string
) {
  if (!currentSkillPackage.value) return;
  await updateVariant({
    skillId: currentSkillPackage.value.id,
    variantId,
    newName,
    newVersionId,
    newDescription
  });
}

async function handleLinkSkillsToProject(skillIds: string[], ideLabels: string[]) {
  if (!configuringProject.value) return;

  busy.value = true;
  busyText.value = t("messages.linkingSkills");

  try {
    let successCount = 0;
    let failCount = 0;

    for (const skillId of skillIds) {
      const skill = localSkills.value.find(s => s.id === skillId);
      if (!skill) continue;

      try {
        // Build link targets for the project's IDE directories
        const linkTargets = [];
        for (const ideLabel of ideLabels) {
          const target = ideOptions.value.find(opt => opt.label === ideLabel);
          if (!target) continue;

          const projectPath = configuringProject.value.path;
          // Use projectDir if available (for OpenCode it's .opencode/skills)
          // Otherwise fall back to globalDir behavior
          const ideDir = target.projectDir 
            ? `${projectPath}/${target.projectDir}`
            : target.globalDir.startsWith('.') 
              ? `${projectPath}/${target.globalDir}`
              : target.globalDir;
          
          linkTargets.push({
            name: ideLabel,
            path: ideDir
          });
        }

        if (linkTargets.length === 0) {
          failCount++;
          continue;
        }

        await invoke("link_local_skill", {
          request: {
            skillPath: skill.path,
            skillName: skill.name,
            linkTargets
          }
        });
        successCount++;
      } catch {
        failCount++;
      }
    }

    if (successCount > 0) {
      toast.success(t("messages.skillsLinked", { success: successCount, failed: failCount }));
      showProjectImportModal.value = false;
      await scanLocalSkills();
    } else {
      toast.error(t("errors.linkFailed"));
    }
  } catch (err) {
    toast.error(getErrorMessage(err, t("errors.linkFailed")));
  } finally {
    busy.value = false;
    busyText.value = "";
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
        <button
          class="tab"
          :class="{ active: activeTab === 'market' }"
          @click="activeTab = 'market'"
        >
          {{ t("app.tabs.market") }}
        </button>
        <button
          class="tab"
          :class="{ active: activeTab === 'ide' }"
          @click="activeTab = 'ide'"
        >
          {{ t("app.tabs.ide") }}
        </button>
        <button
          class="tab"
          :class="{ active: activeTab === 'projects' }"
          @click="activeTab = 'projects'"
        >
          {{ t("app.tabs.projects") }}
        </button>
        <button
          class="tab"
          :class="{ active: activeTab === 'settings' }"
          @click="activeTab = 'settings'"
        >
          {{ t("app.tabs.settings") }}
          <span v-if="updateAvailable" class="tab-badge"></span>
        </button>
      </div>
      <div class="header-controls">
        <div class="control">
          <button
            class="icon-toggle"
            type="button"
            :aria-label="t('app.header.language')"
            :title="locale === 'zh-CN' ? '中文' : 'English'"
            @click="locale = locale === 'zh-CN' ? 'en-US' : 'zh-CN'"
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
            @click="theme = theme === 'light' ? 'dark' : 'light'"
          >
            <svg v-if="theme === 'light'" class="icon" viewBox="0 0 24 24" aria-hidden="true">
              <path
                d="M12 4a1 1 0 011 1v1a1 1 0 11-2 0V5a1 1 0 011-1Zm6.36 2.64a1 1 0 010 1.41l-.7.7a1 1 0 11-1.41-1.41l.7-.7a1 1 0 011.41 0ZM20 11a1 1 0 010 2h-1a1 1 0 110-2h1Zm-8 2a3 3 0 100-6 3 3 0 000 6Zm-7 0a1 1 0 010-2H4a1 1 0 110-2h1a1 1 0 110 2H4a1 1 0 010 2Zm1.64-7.95a1 1 0 011.41 0l.7.7a1 1 0 11-1.41 1.41l-.7-.7a1 1 0 010-1.41ZM12 18a1 1 0 011 1v1a1 1 0 11-2 0v-1a1 1 0 011-1Zm7.07-1.07a1 1 0 010 1.41l-.7.7a1 1 0 11-1.41-1.41l.7-.7a1 1 0 011.41 0ZM6.34 16.93a1 1 0 011.41 0l.7.7a1 1 0 11-1.41 1.41l-.7-.7a1 1 0 010-1.41Z"
                fill="currentColor"
              />
            </svg>
            <svg v-else class="icon" viewBox="0 0 24 24" aria-hidden="true">
              <path
                d="M21 14.5A8.5 8.5 0 019.5 3a.9.9 0 00-.9.9 9.6 9.6 0 0010.5 10.5.9.9 0 00.9-.9Z"
                fill="currentColor"
              />
            </svg>
          </button>
        </div>
      </div>
    </header>

    <main class="content">
      <template v-if="activeTab === 'local'">
        <LocalPanel
          :local-skills="localSkills"
          :local-loading="localLoading"
          :installing-id="installingId"
          :download-queue="downloadQueue"
          :ide-options="ideOptions"
          @install="openInstallModal"
          @install-many="openInstallModal"
          @delete-local="openDeleteLocalModal"
          @open-dir="openSkillDirectory"
          @refresh="scanLocalSkills"
          @import="importLocalSkill"
          @retry-download="retryDownload"
          @remove-from-queue="removeFromQueue"
          @manage-versions="handleManageVersions"
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
          :local-loading="localLoading"
          @add-project="handleAddProject"
          @remove-project="handleRemoveProject"
          @select-project="handleSelectProject"
          @configure-project="handleConfigureProject"
          @export-skills="handleExportSkills"
          @import-skills="handleImportSkills"
        />
      </template>

      <template v-else-if="activeTab === 'settings'">
        <SettingsPanel />
      </template>
    </main>

    <InstallModal
      :visible="showInstallModal"
      :ide-options="ideOptions"
      :projects="projects"
      @confirm="confirmInstallToIde"
      @cancel="closeInstallModal"
    />

    <UninstallModal
      :visible="showUninstallModal"
      :target-name="uninstallTargetName"
      :mode="uninstallMode"
      @confirm="confirmUninstall"
      @cancel="cancelUninstall"
    />

    <ProjectAddModal
      :visible="showProjectAddModal"
      @close="showProjectAddModal = false"
      @confirm="handleProjectAddConfirm"
    />

    <ProjectConfigModal
      :visible="showProjectConfigModal"
      :project="configuringProject"
      :ide-options="ideOptions"
      @close="() => { showProjectConfigModal = false; configuringProject = null; }"
      @save="handleProjectConfigSave"
    />

    <ConflictResolutionModal
      :show="showConflictModal"
      :skill="currentConflictSkill"
      :conflict-analysis="currentConflictAnalysis"
      @close="closeConflictModal"
      @resolve="handleConflictResolution"
    />

    <ProjectSkillImportModal
      :show="showProjectExportModal"
      :scan-result="projectSkillScanResult"
      @close="showProjectExportModal = false"
      @import="handleImportSelected"
      @resolve-conflict="handleResolveConflictFromImport"
    />

    <ImportToProjectModal
      :show="showProjectImportModal"
      :project="configuringProject"
      :local-skills="localSkills"
      @close="showProjectImportModal = false"
      @link="handleLinkSkillsToProject"
    />

    <VersionManagerModal
      :show="showVersionManagerModal"
      :skill-package="currentSkillPackage"
      :current-skill-path="currentManagedSkillPath"
      :selected-source-path="selectedCreateVersionSourcePath"
      :loading="versionLoading"
      @close="closeVersionManagerModal"
      @rename="handleRenameVersion"
      @delete="handleDeleteVersion"
      @set-default="handleSetDefaultVersion"
      @compare="handleCompareVersions"
      @create-version="handleCreateVersion"
      @pick-source-path="handlePickSourcePath"
      @create-variant="handleCreateVariant"
      @update-variant="handleUpdateVariant"
      @delete-variant="handleDeleteVariant"
    />

    <VersionDiffModal
      :show="showVersionDiffModal"
      :diff="currentVersionDiff || currentDiff"
      :from-version="comparingFromVersion"
      :to-version="comparingToVersion"
      @close="closeVersionDiffModal"
    />

    <Toast />

    <LoadingOverlay :visible="busy" :text="busyText" />
  </div>
</template>

<style>
/* Global styles moved from App.vue */
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

/* Scrollbar styling */
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

.tab-badge {
  position: absolute;
  top: 6px;
  right: 6px;
  width: 8px;
  height: 8px;
  background: #ff3b30;
  border-radius: 50%;
  border: 2px solid var(--color-tab-active-bg);
}

.tab.active .tab-badge {
  border-color: var(--color-tab-active-bg);
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
}
</style>
