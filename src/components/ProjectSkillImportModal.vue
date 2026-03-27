<script setup lang="ts">
import { ref, computed } from "vue";
import { useI18n } from "vue-i18n";
import BaseModal from "./BaseModal.vue";
import type { ProjectSkill } from "../composables/types";

const { t } = useI18n();

const props = defineProps<{
  show: boolean;
  scanResult: {
    projectPath: string;
    skills: ProjectSkill[];
    newCount: number;
    duplicateCount: number;
    managedVersionCount: number;
    conflictCount: number;
  } | null;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "import", skillPaths: string[]): void;
  (e: "resolveConflict", skill: ProjectSkill): void;
}>();

const selectedSkills = ref<Set<string>>(new Set());

const hasSkills = computed(() => props.scanResult?.skills.length ?? 0 > 0);

const newSkills = computed(() => 
  props.scanResult?.skills.filter(s => s.status === "new") ?? []
);

const duplicateSkills = computed(() => 
  props.scanResult?.skills.filter(s => s.status === "duplicate") ?? []
);

const managedVersionSkills = computed(() =>
  props.scanResult?.skills.filter(s => s.status === "managed_version") ?? []
);

const conflictSkills = computed(() => 
  props.scanResult?.skills.filter(s => s.status === "conflict") ?? []
);

function toggleSkill(path: string) {
  if (selectedSkills.value.has(path)) {
    selectedSkills.value.delete(path);
  } else {
    selectedSkills.value.add(path);
  }
}

function selectAllNew() {
  newSkills.value.forEach(s => selectedSkills.value.add(s.path));
}

function deselectAll() {
  selectedSkills.value.clear();
}

function handleClose() {
  selectedSkills.value.clear();
  emit("close");
}

function handleImport() {
  const paths = Array.from(selectedSkills.value);
  if (paths.length === 0) return;
  emit("import", paths);
  selectedSkills.value.clear();
}

function handleResolveConflict(skill: ProjectSkill) {
  emit("resolveConflict", skill);
}

function getStatusLabel(status: string): string {
  switch (status) {
    case "new": return t("projects.newSkills");
    case "duplicate": return t("projects.duplicateSkills");
    case "managed_version": return t("projects.importedManagedVersions");
    case "conflict": return t("projects.conflictSkills");
    default: return status;
  }
}

function getStatusClass(status: string): string {
  switch (status) {
    case "new": return "status-new";
    case "duplicate": return "status-duplicate";
    case "managed_version": return "status-duplicate";
    case "conflict": return "status-conflict";
    default: return "";
  }
}
</script>

<template>
  <BaseModal :show="show && !!scanResult" :title="t('projects.importFromProject')" size="large" @close="handleClose">
        <div v-if="scanResult" class="modal-content">
          <div class="scan-summary">
            <div class="summary-item">
              <span class="count new">{{ scanResult.newCount }}</span>
              <span class="label">{{ t("projects.newSkills") }}</span>
            </div>
            <div class="summary-item">
              <span class="count duplicate">{{ scanResult.duplicateCount }}</span>
              <span class="label">{{ t("projects.duplicateSkills") }}</span>
            </div>
            <div v-if="scanResult.managedVersionCount > 0" class="summary-item">
              <span class="count duplicate">{{ scanResult.managedVersionCount }}</span>
              <span class="label">{{ t("projects.importedManagedVersions") }}</span>
            </div>
            <div class="summary-item">
              <span class="count conflict">{{ scanResult.conflictCount }}</span>
              <span class="label">{{ t("projects.conflictSkills") }}</span>
            </div>
          </div>

          <div v-if="!hasSkills" class="empty-state">
            {{ t("projects.noSkillsFound") }}
          </div>

          <div v-else class="skills-list">
            <div class="list-header">
              <button class="text-btn" @click="selectAllNew">
                {{ t("common.selectAllNew") }}
              </button>
              <button class="text-btn" @click="deselectAll">
                {{ t("common.deselectAll") }}
              </button>
            </div>

            <!-- New Skills -->
            <div v-if="newSkills.length > 0" class="skill-group">
              <h4 class="group-title">{{ t("projects.newSkills") }}</h4>
              <div
                v-for="skill in newSkills"
                :key="skill.path"
                class="skill-item"
                :class="{ selected: selectedSkills.has(skill.path) }"
                @click="toggleSkill(skill.path)"
              >
                <input
                  type="checkbox"
                  :checked="selectedSkills.has(skill.path)"
                  @click.stop
                  @change="toggleSkill(skill.path)"
                />
                <div class="skill-info">
                  <div class="skill-name">{{ skill.name }}</div>
                  <div class="skill-desc">{{ skill.description || "-" }}</div>
                  <div v-if="skill.currentVersion" class="skill-version">
                    {{ skill.currentVersion.displayName }}
                  </div>
                  <div class="skill-path">{{ skill.path }}</div>
                </div>
                <span class="status-badge" :class="getStatusClass(skill.status)">
                  {{ getStatusLabel(skill.status) }}
                </span>
              </div>
            </div>

            <!-- Duplicate Skills -->
            <div v-if="duplicateSkills.length > 0" class="skill-group">
              <h4 class="group-title">{{ t("projects.duplicateSkills") }}</h4>
              <div
                v-for="skill in duplicateSkills"
                :key="skill.path"
                class="skill-item disabled"
              >
                <input type="checkbox" disabled />
                <div class="skill-info">
                  <div class="skill-name">{{ skill.name }}</div>
                  <div class="skill-desc">{{ skill.description || "-" }}</div>
                  <div v-if="skill.currentVersion" class="skill-version">
                    {{ skill.currentVersion.displayName }}
                  </div>
                  <div class="skill-path">{{ skill.path }}</div>
                </div>
                <span class="status-badge" :class="getStatusClass(skill.status)">
                  {{ getStatusLabel(skill.status) }}
                </span>
              </div>
            </div>

            <div v-if="managedVersionSkills.length > 0" class="skill-group">
              <h4 class="group-title">{{ t("projects.importedManagedVersions") }}</h4>
              <div
                v-for="skill in managedVersionSkills"
                :key="skill.path"
                class="skill-item disabled"
              >
                <input type="checkbox" disabled />
                <div class="skill-info">
                  <div class="skill-name">{{ skill.name }}</div>
                  <div class="skill-desc">{{ skill.description || "-" }}</div>
                  <div v-if="skill.currentVersion" class="skill-version">
                    {{ t("version.sourceProject") }} · {{ skill.currentVersion.displayName }}
                  </div>
                  <div class="skill-path">{{ skill.path }}</div>
                  <div v-if="skill.matchedVersionName" class="existing-info">
                    <span class="existing-label">{{ t("projects.matchesManagedVersion", { name: skill.matchedVersionName }) }}</span>
                  </div>
                </div>
                <span class="status-badge" :class="getStatusClass(skill.status)">
                  {{ getStatusLabel(skill.status) }}
                </span>
              </div>
            </div>

            <!-- Conflict Skills -->
            <div v-if="conflictSkills.length > 0" class="skill-group">
              <h4 class="group-title">{{ t("projects.conflictSkills") }}</h4>
              <div
                v-for="skill in conflictSkills"
                :key="skill.path"
                class="skill-item conflict"
              >
                <div class="skill-info">
                  <div class="skill-name">
                    {{ skill.name }}
                    <span v-if="skill.existingRegistrySkill" class="version-badge">
                      {{ t("projects.hasVersions") }}
                    </span>
                  </div>
                   <div class="skill-desc">{{ skill.description || "-" }}</div>
                   <div v-if="skill.currentVersion" class="skill-version">
                     {{ t("version.sourceProject") }} · {{ skill.currentVersion.displayName }}
                   </div>
                   <div class="skill-path">{{ skill.path }}</div>
                   <div v-if="skill.existingRegistrySkill" class="existing-info">
                     <span class="existing-label">{{ t("projects.existingVersion") }}:</span>
                     <span class="existing-path">
                       {{ skill.existingRegistrySkill.currentVersion?.displayName || skill.existingRegistrySkill.path }}
                     </span>
                   </div>
                 </div>
                <button
                  class="resolve-btn"
                  @click="handleResolveConflict(skill)"
                >
                  {{ t("projects.resolveConflict") }}
                </button>
              </div>
            </div>
          </div>
        </div>

        <template #footer>
          <button class="ghost" @click="handleClose">
            {{ t("common.cancel") }}
          </button>
          <button
            class="primary"
            :disabled="selectedSkills.size === 0"
            @click="handleImport"
          >
            {{ t("projects.importSelected") }} ({{ selectedSkills.size }})
          </button>
        </template>
  </BaseModal>
</template>

<style scoped>

.scan-summary {
  display: flex;
  gap: 24px;
  margin-bottom: 20px;
  padding: 16px;
  background: var(--color-card-bg);
  border-radius: 8px;
}

.summary-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.summary-item .count {
  font-size: 24px;
  font-weight: 600;
}

.summary-item .count.new {
  color: var(--color-success-text);
}

.summary-item .count.duplicate {
  color: var(--color-muted);
}

.summary-item .count.conflict {
  color: var(--color-warning-text);
}

.summary-item .label {
  font-size: 12px;
  color: var(--color-muted);
}

.empty-state {
  text-align: center;
  padding: 40px;
  color: var(--color-muted);
}

.list-header {
  display: flex;
  gap: 16px;
  margin-bottom: 12px;
}

.text-btn {
  background: none;
  border: none;
  color: var(--color-primary-bg);
  cursor: pointer;
  font-size: 13px;
  padding: 4px 8px;
}

.text-btn:hover {
  text-decoration: underline;
}

.skill-group {
  margin-bottom: 20px;
}

.group-title {
  font-size: 14px;
  color: var(--color-muted);
  margin: 0 0 8px 0;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--color-border);
}

.skill-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 12px;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  margin-bottom: 8px;
  cursor: pointer;
  transition: all 0.2s;
}

.skill-item:hover {
  background: var(--color-hover);
}

.skill-item.selected {
  border-color: var(--color-primary-bg);
  background: var(--color-primary-bg-alpha);
}

.skill-item.disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.skill-item.conflict {
  border-color: var(--color-warning-border);
  background: var(--color-warning-bg);
  cursor: default;
}

.skill-item input[type="checkbox"] {
  margin-top: 2px;
}

.skill-info {
  flex: 1;
  min-width: 0;
}

.skill-name {
  font-weight: 600;
  font-size: 14px;
  margin-bottom: 4px;
}

.skill-desc {
  font-size: 12px;
  color: var(--color-muted);
  margin-bottom: 4px;
}

.skill-version {
  font-size: 12px;
  color: var(--color-primary-bg);
  margin-bottom: 4px;
  font-weight: 600;
}

.skill-path {
  font-size: 11px;
  color: var(--color-muted);
  font-family: monospace;
  word-break: break-all;
}

.version-badge {
  margin-left: 8px;
  padding: 2px 8px;
  background: var(--color-primary-bg);
  color: var(--color-primary-text);
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
}

.existing-info {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 6px;
  padding: 6px 10px;
  background: var(--color-card-bg);
  border-radius: 4px;
  font-size: 11px;
}

.existing-label {
  color: var(--color-muted);
}

.existing-path {
  color: var(--color-text);
  font-family: monospace;
  word-break: break-all;
}

.status-badge {
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
}

.status-badge.status-new {
  background: var(--color-success-bg);
  color: var(--color-success-text);
}

.status-badge.status-duplicate {
  background: var(--color-chip-bg);
  color: var(--color-muted);
}

.status-badge.status-conflict {
  background: var(--color-warning-bg);
  color: var(--color-warning-text);
}

.resolve-btn {
  padding: 6px 12px;
  font-size: 12px;
  background: var(--color-warning-bg);
  color: var(--color-warning-text);
  border: 1px solid var(--color-warning-border);
  border-radius: 4px;
  cursor: pointer;
}

.resolve-btn:hover {
  background: var(--color-warning-border);
}

</style>
