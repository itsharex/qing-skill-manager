<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import type { ProjectConfig, ProjectSkill, SkillPackage, SkillVersion } from "../../composables/types";

const props = defineProps<{
  skillPackage: SkillPackage;
  sortedVersions: SkillVersion[];
  projects?: ProjectConfig[];
  projectSkills?: ProjectSkill[];
  projectSkillsLoading?: boolean;
  selectedSourcePath?: string;
  selectedProjectId?: string | null;
}>();

const emit = defineEmits<{
  (e: "createVersion", version: string, displayName: string, sourcePath: string, parentVersion?: string): void;
  (e: "pickSourcePath"): void;
  (e: "pickProject", projectId: string): void;
}>();

const { t } = useI18n();

const createVersionMode = ref<"folder" | "project">("project");
const createVersionNumber = ref("");
const createVersionDisplayName = ref("");
const createVersionSourcePath = ref("");
const createVersionParentId = ref("");
const projectImportProjectId = ref("");
const projectImportSkillPath = ref("");
const showProjectPicker = ref(false);
const showProjectSkillPicker = ref(false);

watch(
  () => props.selectedSourcePath,
  (nextPath) => {
    if (nextPath) {
      createVersionSourcePath.value = nextPath;
    }
  }
);

watch(
  () => props.selectedProjectId,
  (projectId) => {
    if (projectId) {
      projectImportProjectId.value = projectId;
    }
  },
  { immediate: true }
);

watch(projectImportProjectId, (projectId, previous) => {
  if (!projectId || projectId === previous) return;
  projectImportSkillPath.value = "";
  emit("pickProject", projectId);
});

watch(projectImportSkillPath, (skillPath) => {
  if (!skillPath) return;
  createVersionSourcePath.value = skillPath;
});

const selectedProjectSkills = computed(() => props.projectSkills ?? []);
const filteredProjectSkills = computed(() => {
  if (!props.skillPackage) return selectedProjectSkills.value;
  return selectedProjectSkills.value.filter((skill) => skill.name === props.skillPackage?.name);
});

const selectedProject = computed(() => {
  return (props.projects ?? []).find((project) => project.id === projectImportProjectId.value) || null;
});

const selectedProjectSkill = computed(() => {
  return filteredProjectSkills.value.find((skill) => skill.path === projectImportSkillPath.value) || null;
});

const selectedProjectSkillMatchesName = computed(() => {
  if (!selectedProjectSkill.value || !props.skillPackage) return false;
  return selectedProjectSkill.value.name === props.skillPackage.name;
});

const selectedProjectSkillMatchesDefault = computed(() => {
  return selectedProjectSkill.value?.matchesDefaultVersion === true;
});

const selectedProjectSkillMatchedVersionName = computed(() => selectedProjectSkill.value?.matchedVersionName || "");

const selectedProjectSkillIsConflict = computed(() => selectedProjectSkill.value?.status === "conflict");

function chooseProject(projectId: string) {
  projectImportProjectId.value = projectId;
  showProjectPicker.value = false;
}

function chooseProjectSkill(skillPath: string) {
  projectImportSkillPath.value = skillPath;
  showProjectSkillPicker.value = false;
}

function handleCreateVersion() {
  if (!createVersionNumber.value.trim() || !createVersionDisplayName.value.trim() || !createVersionSourcePath.value.trim()) {
    return;
  }
  emit(
    "createVersion",
    createVersionNumber.value.trim(),
    createVersionDisplayName.value.trim(),
    createVersionSourcePath.value.trim(),
    createVersionParentId.value || undefined
  );
  createVersionNumber.value = "";
  createVersionDisplayName.value = "";
  createVersionSourcePath.value = "";
  createVersionParentId.value = "";
}

function reset() {
  createVersionNumber.value = "";
  createVersionDisplayName.value = "";
  createVersionSourcePath.value = "";
  createVersionParentId.value = "";
  createVersionMode.value = "project";
  projectImportProjectId.value = props.selectedProjectId || "";
  projectImportSkillPath.value = "";
  showProjectPicker.value = false;
  showProjectSkillPicker.value = false;
}

defineExpose({ reset });
</script>

<template>
  <section class="create-version-section section-card">
    <div class="section-header">
      <div>
        <h4>{{ t("version.createVersionSimpleTitle") }}</h4>
        <p class="section-help">{{ t("version.createVersionSimpleHelp") }}</p>
      </div>
    </div>
    <div class="import-mode-switch">
      <button class="ghost" :class="{ active: createVersionMode === 'folder' }" @click="createVersionMode = 'folder'">
        {{ t("version.importFromFolder") }}
      </button>
      <button class="ghost" :class="{ active: createVersionMode === 'project' }" @click="createVersionMode = 'project'">
        {{ t("version.importFromProject") }}
      </button>
    </div>

    <div v-if="createVersionMode === 'folder'" class="import-mode-card">
      <div class="mode-title">{{ t("version.importFromFolderTitle") }}</div>
      <p class="section-help">{{ t("version.importFromFolderHelp") }}</p>
      <div class="source-path-row source-input">
        <input v-model="createVersionSourcePath" class="input source-path-input" :placeholder="t('version.createVersionSourcePathPlaceholder')" />
        <button class="ghost" @click="$emit('pickSourcePath')">
          {{ t("version.pickSourcePath") }}
        </button>
      </div>
    </div>

    <div v-else class="import-mode-card">
      <div class="mode-title">{{ t("version.importFromProjectTitle") }}</div>
      <p class="section-help">{{ t("version.importFromProjectHelp") }}</p>
      <div class="project-import-grid">
        <div class="picker-shell">
          <button class="picker-trigger" @click="showProjectPicker = !showProjectPicker">
            <div class="picker-text">
              <span class="picker-label">{{ t("version.selectProject") }}</span>
              <strong>{{ selectedProject?.name || t("version.selectProject") }}</strong>
              <small v-if="selectedProject" class="picker-subtext">{{ selectedProject.path }}</small>
            </div>
            <span class="picker-chevron">&#9662;</span>
          </button>
          <div v-if="showProjectPicker" class="picker-panel">
            <button
              v-for="project in projects || []"
              :key="project.id"
              class="picker-option"
              :class="{ active: project.id === projectImportProjectId }"
              @click="chooseProject(project.id)"
            >
              <div class="picker-option-title">{{ project.name }}</div>
              <div class="picker-option-meta">{{ project.path }}</div>
            </button>
          </div>
        </div>

        <div class="picker-shell">
          <button class="picker-trigger" :disabled="!projectImportProjectId || projectSkillsLoading" @click="showProjectSkillPicker = !showProjectSkillPicker">
            <div class="picker-text">
              <span class="picker-label">{{ t("version.selectProjectSkill") }}</span>
              <strong>{{ selectedProjectSkill?.name || (projectSkillsLoading ? t("version.loadingProjectSkills") : t("version.selectProjectSkill")) }}</strong>
              <small v-if="selectedProjectSkill" class="picker-subtext">{{ selectedProjectSkill.path }}</small>
            </div>
            <span class="picker-chevron">&#9662;</span>
          </button>
          <div v-if="showProjectSkillPicker" class="picker-panel">
            <button
              v-for="skill in filteredProjectSkills"
              :key="skill.path"
              class="picker-option"
              :class="{ active: skill.path === projectImportSkillPath }"
              @click="chooseProjectSkill(skill.path)"
            >
              <div class="picker-option-title">{{ skill.name }}</div>
              <div class="picker-option-meta">{{ skill.path }}</div>
            </button>
            <div v-if="!projectSkillsLoading && filteredProjectSkills.length === 0" class="picker-empty-state">
              {{ t("version.noSameNameProjectSkills") }}
            </div>
          </div>
        </div>
      </div>
      <div v-if="projectImportSkillPath" class="project-import-preview">
        <span class="preview-label">{{ t("version.sourcePathPreview") }}:</span>
        <span class="preview-path">{{ projectImportSkillPath }}</span>
      </div>
      <div v-if="selectedProjectSkill" class="project-import-signals">
        <div v-if="selectedProjectSkillMatchesName" class="signal-chip same-name">
          {{ t("version.sameNameSkillDetected") }}
        </div>
        <div v-if="selectedProjectSkillMatchesDefault" class="signal-chip default-match">
          {{ t("version.matchesDefaultVersion") }}
        </div>
        <div v-else-if="selectedProjectSkillMatchedVersionName" class="signal-chip version-match">
          {{ t("version.matchesManagedVersion", { name: selectedProjectSkillMatchedVersionName }) }}
        </div>
        <div v-if="selectedProjectSkillIsConflict" class="signal-chip conflict">
          {{ t("version.projectSkillConflictDetected") }}
        </div>
      </div>
    </div>

    <div class="create-version-grid">
      <input v-model="createVersionNumber" class="input" :placeholder="t('version.createVersionNumberPlaceholder')" />
      <input v-model="createVersionDisplayName" class="input" :placeholder="t('version.createVersionDisplayNamePlaceholder')" />
      <select v-model="createVersionParentId" class="version-select">
        <option value="">{{ t("version.createVersionParentOptional") }}</option>
        <option v-for="version in sortedVersions" :key="version.id" :value="version.id">
          {{ version.displayName }}
        </option>
      </select>
      <button
        class="primary"
        :disabled="!createVersionNumber.trim() || !createVersionDisplayName.trim() || !createVersionSourcePath.trim()"
        @click="handleCreateVersion"
      >
        {{ t("version.createVersionConfirm") }}
      </button>
    </div>
  </section>
</template>

<style scoped>
.section-card {
  background: var(--color-card-bg);
  border: 1px solid var(--color-card-border);
  border-radius: 12px;
  padding: 18px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: flex-start;
  margin-bottom: 14px;
}

.section-header h4 {
  margin: 0;
  font-size: 16px;
}

.section-help {
  margin: 8px 0 0 0;
  font-size: 13px;
  color: var(--color-muted);
}

.create-version-section {
  margin-bottom: 24px;
  padding-bottom: 20px;
  border-bottom: 1px solid var(--color-border);
}

.create-version-section h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
}

.create-version-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.import-mode-switch {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}

.import-mode-switch .ghost.active {
  border-color: var(--color-primary-bg);
  color: var(--color-primary-bg);
  background: color-mix(in srgb, var(--color-primary-bg) 8%, transparent);
}

.import-mode-card {
  margin-bottom: 14px;
  padding: 14px;
  border-radius: 10px;
  border: 1px dashed var(--color-card-border);
  background: color-mix(in srgb, var(--color-card-bg) 92%, var(--color-bg) 8%);
}

.mode-title {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 6px;
}

.project-import-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.picker-shell {
  position: relative;
}

.picker-trigger {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 14px 16px;
  border-radius: 12px;
  border: 1px solid var(--color-card-border);
  background: linear-gradient(180deg, color-mix(in srgb, var(--color-card-bg) 92%, var(--color-bg) 8%), var(--color-bg));
  color: var(--color-text);
  text-align: left;
  box-shadow: 0 10px 24px rgba(0, 0, 0, 0.08), inset 0 1px 0 rgba(255, 255, 255, 0.03);
  transition: border-color 0.18s ease, transform 0.18s ease, box-shadow 0.18s ease;
}

.picker-trigger:hover:not(:disabled) {
  border-color: color-mix(in srgb, var(--color-primary-bg) 55%, var(--color-card-border));
  transform: translateY(-1px);
  box-shadow: 0 14px 28px rgba(0, 0, 0, 0.12), inset 0 1px 0 rgba(255, 255, 255, 0.05);
}

.picker-trigger:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.picker-text {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.picker-label {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--color-muted);
}

.picker-subtext {
  color: var(--color-muted);
  font-size: 12px;
  word-break: break-all;
}

.picker-chevron {
  color: var(--color-muted);
  font-size: 14px;
  flex-shrink: 0;
}

.picker-panel {
  position: absolute;
  top: calc(100% + 8px);
  left: 0;
  right: 0;
  z-index: 20;
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 280px;
  overflow: auto;
  padding: 10px;
  border-radius: 14px;
  border: 1px solid var(--color-card-border);
  background: color-mix(in srgb, var(--color-bg) 92%, var(--color-card-bg) 8%);
  box-shadow: 0 18px 44px rgba(0, 0, 0, 0.22);
}

.picker-option {
  width: 100%;
  text-align: left;
  padding: 12px 14px;
  border-radius: 10px;
  border: 1px solid transparent;
  background: color-mix(in srgb, var(--color-card-bg) 92%, transparent 8%);
  color: var(--color-text);
  transition: border-color 0.18s ease, background-color 0.18s ease, transform 0.18s ease;
}

.picker-option:hover {
  border-color: var(--color-card-border);
  background: color-mix(in srgb, var(--color-primary-bg) 8%, var(--color-card-bg));
  transform: translateY(-1px);
}

.picker-option.active {
  border-color: color-mix(in srgb, var(--color-primary-bg) 65%, var(--color-card-border));
  background: color-mix(in srgb, var(--color-primary-bg) 14%, var(--color-card-bg));
}

.picker-option-title {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 4px;
}

.picker-option-meta {
  font-size: 12px;
  color: var(--color-muted);
  word-break: break-all;
}

.picker-empty-state {
  padding: 14px;
  border-radius: 10px;
  background: color-mix(in srgb, var(--color-card-bg) 88%, transparent 12%);
  color: var(--color-muted);
  font-size: 13px;
  text-align: center;
}

.create-version-grid .version-select {
  width: 100%;
}

.project-import-preview {
  margin-top: 10px;
  display: flex;
  gap: 8px;
  align-items: flex-start;
  font-size: 12px;
}

.preview-label {
  color: var(--color-muted);
}

.preview-path {
  font-family: monospace;
  word-break: break-all;
}

.project-import-signals {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 10px;
}

.signal-chip {
  padding: 6px 10px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 600;
  border: 1px solid transparent;
}

.signal-chip.same-name {
  background: color-mix(in srgb, var(--color-primary-bg) 12%, transparent);
  color: var(--color-primary-bg);
  border-color: color-mix(in srgb, var(--color-primary-bg) 35%, transparent);
}

.signal-chip.default-match {
  background: color-mix(in srgb, var(--color-success-bg) 60%, transparent);
  color: var(--color-success-text);
  border-color: var(--color-success-border);
}

.signal-chip.version-match {
  background: color-mix(in srgb, var(--color-primary-bg) 12%, transparent);
  color: var(--color-primary-bg);
  border-color: color-mix(in srgb, var(--color-primary-bg) 40%, transparent);
}

.signal-chip.conflict {
  background: color-mix(in srgb, var(--color-warning-bg) 60%, transparent);
  color: var(--color-warning-text);
  border-color: var(--color-warning-border);
}

.source-input {
  grid-column: 1 / -1;
}

.source-path-row {
  display: flex;
  gap: 8px;
  align-items: stretch;
}

.source-path-input {
  flex: 1;
}

.version-select {
  appearance: none;
  -webkit-appearance: none;
  -moz-appearance: none;
  padding: 12px 42px 12px 14px;
  border: 1px solid var(--color-card-border);
  border-radius: 10px;
  background-color: var(--color-bg);
  background-image:
    linear-gradient(45deg, transparent 50%, var(--color-muted) 50%),
    linear-gradient(135deg, var(--color-muted) 50%, transparent 50%),
    linear-gradient(to right, transparent, transparent);
  background-position:
    calc(100% - 18px) calc(50% - 3px),
    calc(100% - 12px) calc(50% - 3px),
    100% 0;
  background-size: 6px 6px, 6px 6px, 2.5em 2.5em;
  background-repeat: no-repeat;
  color: var(--color-text);
  font-size: 14px;
  min-width: 180px;
  box-shadow: 0 1px 0 rgba(255, 255, 255, 0.03), inset 0 1px 2px rgba(0, 0, 0, 0.06);
  transition: border-color 0.18s ease, box-shadow 0.18s ease, transform 0.18s ease, background-color 0.18s ease;
}

.version-select:hover {
  border-color: color-mix(in srgb, var(--color-primary-bg) 45%, var(--color-card-border));
  background-color: color-mix(in srgb, var(--color-bg) 88%, var(--color-card-bg) 12%);
}

.version-select:focus {
  outline: none;
  border-color: var(--color-primary-bg);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-primary-bg) 18%, transparent), inset 0 1px 2px rgba(0, 0, 0, 0.04);
}

.version-select:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  background-color: color-mix(in srgb, var(--color-bg) 80%, var(--color-card-bg) 20%);
}

.version-select option {
  color: var(--color-text);
  background: var(--color-bg);
}
</style>
