<script setup lang="ts">
import { ref, computed } from "vue";
import type { ProjectConfig, IdeOption } from "../composables/types";
import { useI18n } from "vue-i18n";
import BaseModal from "./BaseModal.vue";

const { t } = useI18n();

const props = defineProps<{
  visible: boolean;
  project: ProjectConfig | null;
  ideOptions: IdeOption[];
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "save", projectId: string, ideTargets: string[]): void;
}>();

const selectedTargets = ref<string[]>([]);

const selectedIdeSet = computed(() => new Set(selectedTargets.value));

function toggleIdeTarget(ideLabel: string) {
  if (selectedIdeSet.value.has(ideLabel)) {
    selectedTargets.value = selectedTargets.value.filter((t) => t !== ideLabel);
  } else {
    selectedTargets.value = [...selectedTargets.value, ideLabel];
  }
}

function handleSave() {
  if (props.project) {
    emit("save", props.project.id, selectedTargets.value);
  }
}

function handleClose() {
  emit("close");
}

// Reset selected targets when project changes
if (props.project) {
  selectedTargets.value = [...props.project.ideTargets];
}
</script>

<template>
  <BaseModal :show="visible" :title="t('projects.configureTitle')" size="medium" @close="handleClose">
          <div v-if="project" class="modal-body">
            <div class="project-info">
              <div class="info-label">{{ t("projects.projectName") }}</div>
              <div class="info-value">{{ project.name }}</div>
            </div>
            <div class="project-info">
              <div class="info-label">{{ t("projects.projectPath") }}</div>
              <div class="info-value path">{{ project.path }}</div>
            </div>

            <div class="ide-selection">
              <div class="section-title">{{ t("projects.selectIdeTargets") }}</div>
              <div class="ide-grid">
                <button
                  v-for="option in ideOptions"
                  :key="option.id"
                  class="ide-checkbox"
                  :class="{ active: selectedIdeSet.has(option.label) }"
                  @click="toggleIdeTarget(option.label)"
                >
                  <input
                    type="checkbox"
                    :checked="selectedIdeSet.has(option.label)"
                    :disabled="false"
                  />
                  <span>{{ option.label }}</span>
                </button>
              </div>
            </div>

            <div class="hint">
              {{ t("projects.configureHint") }}
            </div>
          </div>

          <template #footer>
            <button class="ghost" @click="handleClose">{{ t("projects.cancel") }}</button>
            <button class="primary" @click="handleSave">{{ t("projects.save") }}</button>
          </template>
  </BaseModal>
</template>

<style scoped>
.modal-body {
  /* Content-specific styles only */
}

.project-info {
  margin-bottom: 16px;
}

.info-label {
  font-size: 12px;
  color: var(--color-muted);
  margin-bottom: 4px;
}

.info-value {
  font-size: 14px;
  color: var(--color-text);
  font-weight: 500;
}

.info-value.path {
  word-break: break-all;
  font-family: monospace;
  background: var(--color-card-bg);
  padding: 8px 12px;
  border-radius: 6px;
}

.ide-selection {
  margin-top: 24px;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text);
  margin-bottom: 12px;
}

.ide-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 8px;
}

.ide-checkbox {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  background: var(--color-card-bg);
  border: 1px solid var(--color-card-border);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 13px;
  color: var(--color-text);
}

.ide-checkbox:hover {
  border-color: var(--color-primary-bg);
  background: var(--color-tabs-bg);
}

/* Active state removed - no color change on selection */

.ide-checkbox input[type="checkbox"] {
  pointer-events: none;
}

</style>
