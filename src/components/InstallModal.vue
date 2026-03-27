<script setup lang="ts">
import { ref } from "vue";
import type { IdeOption, ProjectConfig } from "../composables/types";
import { useI18n } from "vue-i18n";
import BaseModal from "./BaseModal.vue";

const props = defineProps<{
  visible: boolean;
  ideOptions: IdeOption[];
  projects: ProjectConfig[];
}>();

const emit = defineEmits<{
  (e: "confirm", installTarget: "ide" | "project", targetIds: string[], projects: ProjectConfig[]): void;
  (e: "cancel"): void;
}>();

const { t } = useI18n();

const selectedIdeTargets = ref<string[]>([]);
const selectedProjectIds = ref<string[]>([]);

function toggleIdeTarget(ideId: string) {
  const index = selectedIdeTargets.value.indexOf(ideId);
  if (index === -1) {
    selectedIdeTargets.value.push(ideId);
  } else {
    selectedIdeTargets.value.splice(index, 1);
  }
}

function toggleProject(projectId: string) {
  const index = selectedProjectIds.value.indexOf(projectId);
  if (index === -1) {
    selectedProjectIds.value.push(projectId);
  } else {
    selectedProjectIds.value.splice(index, 1);
  }
}

function confirmInstallToIde() {
  if (selectedIdeTargets.value.length === 0) {
    return;
  }
  emit("confirm", "ide", [...selectedIdeTargets.value], props.projects);
  selectedIdeTargets.value = [];
}

function confirmInstallToProject() {
  if (selectedProjectIds.value.length === 0) {
    return;
  }
  emit("confirm", "project", [...selectedProjectIds.value], props.projects);
  selectedProjectIds.value = [];
}

function close() {
  selectedIdeTargets.value = [];
  selectedProjectIds.value = [];
  emit("cancel");
}
</script>

<template>
  <BaseModal :show="visible" :title="t('installModal.selectTargetTitle')" size="large" @close="close">
    <div class="two-columns">
      <!-- IDE Column -->
      <div class="column">
        <div class="column-header">
          <h3 class="column-title">
            <span class="icon">IDE</span>
            {{ t("installModal.globalIde") }}
          </h3>
          <span class="count">{{ selectedIdeTargets.length }} / {{ ideOptions.length }}</span>
        </div>
        <div class="options-list">
          <label
            v-for="ide in ideOptions"
            :key="ide.id"
            class="option-item"
            :class="{ selected: selectedIdeTargets.includes(ide.label) }"
          >
            <input
              type="checkbox"
              :checked="selectedIdeTargets.includes(ide.label)"
              @change="toggleIdeTarget(ide.label)"
            />
            <span class="option-label">{{ ide.label }}</span>
          </label>
        </div>
      </div>

      <!-- Project Column -->
      <div class="column">
        <div class="column-header">
          <h3 class="column-title">
            <span class="icon">Project</span>
            {{ t("installModal.project") }}
          </h3>
          <span class="count">{{ selectedProjectIds.length }} / {{ projects.length }}</span>
        </div>
        <div v-if="projects.length === 0" class="empty-hint">
          {{ t("installModal.noProjects") }}
        </div>
        <div v-else class="options-list">
          <label
            v-for="project in projects"
            :key="project.id"
            class="option-item project-item"
            :class="{ selected: selectedProjectIds.includes(project.id) }"
          >
            <input
              type="checkbox"
              :checked="selectedProjectIds.includes(project.id)"
              @change="toggleProject(project.id)"
            />
            <div class="option-content">
              <span class="option-label">{{ project.name }}</span>
              <span class="option-desc">{{ project.path }}</span>
            </div>
          </label>
        </div>
      </div>
    </div>

    <template #footer>
      <button class="primary" :disabled="selectedIdeTargets.length === 0" @click="confirmInstallToIde">
        {{ t("installModal.installToIde") }}
      </button>
      <button class="primary" :disabled="selectedProjectIds.length === 0 || projects.length === 0" @click="confirmInstallToProject">
        {{ t("installModal.cloneToProject") }}
      </button>
      <button class="ghost" @click="close">{{ t("installModal.cancel") }}</button>
    </template>
  </BaseModal>
</template>

<style scoped>
.two-columns {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 24px;
}

.column {
  display: flex;
  flex-direction: column;
  min-height: 300px;
  max-height: 50vh;
  border: 1px solid var(--color-card-border);
  border-radius: 8px;
  overflow: hidden;
}

.column-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: var(--color-tabs-bg);
  border-bottom: 1px solid var(--color-card-border);
}

.column-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text);
  margin: 0;
}

.column-title .icon {
  font-size: 18px;
}

.count {
  font-size: 13px;
  color: var(--color-muted);
  font-weight: 500;
}

.options-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.option-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 12px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.2s ease;
}

.option-item:hover {
  background: var(--color-tabs-bg);
}

.option-item input[type="checkbox"] {
  margin-top: 2px;
  cursor: pointer;
}

.option-label {
  font-size: 14px;
  font-weight: 500;
}

.project-item {
  flex-direction: column;
  gap: 4px;
}

.option-content {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.option-desc {
  font-size: 12px;
  opacity: 0.7;
  word-break: break-all;
}

.empty-hint {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-muted);
  font-size: 14px;
  padding: 40px 20px;
  text-align: center;
}

@media (max-width: 768px) {
  .two-columns {
    grid-template-columns: 1fr;
  }

  .column {
    max-height: 40vh;
  }
}
</style>
