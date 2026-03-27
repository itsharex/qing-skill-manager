<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import BaseModal from "./BaseModal.vue";
import type { SkillDiff, SkillVersion } from "../composables/types";

const props = defineProps<{
  show: boolean;
  diff: SkillDiff | null;
  fromVersion: SkillVersion | null;
  toVersion: SkillVersion | null;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

const { t } = useI18n();

const hasChanges = computed(() => {
  if (!props.diff) return false;
  return props.diff.additions > 0 || props.diff.deletions > 0 || props.diff.metadataChanges.length > 0;
});

const similarityPercentage = computed(() => {
  if (!props.diff) return 0;
  return Math.round(props.diff.similarityScore * 100);
});

function handleClose() {
  emit("close");
}
</script>

<template>
  <BaseModal :show="show" :title="t('diff.title')" size="large" @close="handleClose">
        <div class="modal-content">
          <div v-if="!diff" class="empty-state">
            {{ t("diff.noData") }}
          </div>

          <div v-else>
            <div class="diff-summary">
              <div class="version-comparison">
                <div class="version-box">
                  <span class="version-label">{{ t("diff.fromVersion") }}</span>
                  <span class="version-name">{{ fromVersion?.displayName || diff.fromVersion }}</span>
                </div>
                <div class="arrow">→</div>
                <div class="version-box">
                  <span class="version-label">{{ t("diff.toVersion") }}</span>
                  <span class="version-name">{{ toVersion?.displayName || diff.toVersion }}</span>
                </div>
              </div>

              <div class="stats-grid">
                <div class="stat-item">
                  <span class="stat-value additions">+{{ diff.additions }}</span>
                  <span class="stat-label">{{ t("diff.additions") }}</span>
                </div>
                <div class="stat-item">
                  <span class="stat-value deletions">-{{ diff.deletions }}</span>
                  <span class="stat-label">{{ t("diff.deletions") }}</span>
                </div>
                <div class="stat-item">
                  <span class="stat-value">{{ diff.filesChanged.length }}</span>
                  <span class="stat-label">{{ t("diff.filesChanged") }}</span>
                </div>
                <div class="stat-item">
                  <span class="stat-value">{{ similarityPercentage }}%</span>
                  <span class="stat-label">{{ t("diff.similarity") }}</span>
                </div>
              </div>
            </div>

            <div v-if="diff.metadataChanges.length > 0" class="metadata-section">
              <h4>{{ t("diff.metadataChanges") }}</h4>
              <div class="metadata-list">
                <div
                  v-for="(change, index) in diff.metadataChanges"
                  :key="index"
                  class="metadata-item"
                >
                  <span class="field-name">{{ change.field }}:</span>
                  <div class="change-values">
                    <span v-if="change.oldValue" class="old-value">{{ change.oldValue }}</span>
                    <span v-else class="empty-value">{{ t("diff.empty") }}</span>
                    <span class="change-arrow">→</span>
                    <span v-if="change.newValue" class="new-value">{{ change.newValue }}</span>
                    <span v-else class="empty-value">{{ t("diff.empty") }}</span>
                  </div>
                </div>
              </div>
            </div>

            <div v-if="diff.filesChanged.length > 0" class="files-section">
              <h4>{{ t("diff.filesChanged") }}</h4>
              <ul class="files-list">
                <li v-for="file in diff.filesChanged" :key="file" class="file-item">
                  {{ file }}
                </li>
              </ul>
            </div>

            <div v-if="diff.contentDiff" class="content-section">
              <h4>{{ t("diff.contentDiff") }}</h4>
              <pre class="diff-content"><code>{{ diff.contentDiff }}</code></pre>
            </div>

            <div v-if="!hasChanges" class="no-changes">
              {{ t("diff.noChanges") }}
            </div>
          </div>
        </div>

        <template #footer>
          <button class="ghost" @click="handleClose">
            {{ t("common.cancel") }}
          </button>
        </template>
  </BaseModal>
</template>

<style scoped>
.empty-state {
  text-align: center;
  padding: 40px;
  color: var(--color-muted);
}

.diff-summary {
  background: var(--color-card-bg);
  border-radius: 8px;
  padding: 20px;
  margin-bottom: 20px;
}

.version-comparison {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 20px;
  margin-bottom: 20px;
}

.version-box {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 12px 24px;
  background: var(--color-bg);
  border-radius: 8px;
  min-width: 120px;
}

.version-label {
  font-size: 12px;
  color: var(--color-muted);
}

.version-name {
  font-size: 14px;
  font-weight: 600;
}

.arrow {
  font-size: 24px;
  color: var(--color-muted);
  font-weight: 300;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  text-align: center;
}

.stat-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.stat-value {
  font-size: 24px;
  font-weight: 600;
  color: var(--color-text);
}

.stat-value.additions {
  color: var(--color-success-text);
}

.stat-value.deletions {
  color: var(--color-error-text);
}

.stat-label {
  font-size: 12px;
  color: var(--color-muted);
}

.metadata-section,
.files-section,
.content-section {
  margin-bottom: 20px;
}

.metadata-section h4,
.files-section h4,
.content-section h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  color: var(--color-muted);
}

.metadata-list {
  background: var(--color-card-bg);
  border-radius: 8px;
  padding: 12px;
}

.metadata-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 0;
  border-bottom: 1px solid var(--color-border);
}

.metadata-item:last-child {
  border-bottom: none;
}

.field-name {
  font-size: 13px;
  font-weight: 600;
  min-width: 100px;
  color: var(--color-muted);
}

.change-values {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
}

.old-value {
  color: var(--color-error-text);
  text-decoration: line-through;
  font-size: 13px;
}

.new-value {
  color: var(--color-success-text);
  font-weight: 600;
  font-size: 13px;
}

.empty-value {
  color: var(--color-muted);
  font-style: italic;
  font-size: 13px;
}

.change-arrow {
  color: var(--color-muted);
}

.files-list {
  list-style: none;
  padding: 0;
  margin: 0;
  background: var(--color-card-bg);
  border-radius: 8px;
  padding: 12px;
}

.file-item {
  padding: 6px 0;
  font-size: 13px;
  font-family: monospace;
  border-bottom: 1px solid var(--color-border);
}

.file-item:last-child {
  border-bottom: none;
}

.diff-content {
  background: var(--color-card-bg);
  border-radius: 8px;
  padding: 16px;
  overflow-x: auto;
  font-size: 12px;
  line-height: 1.6;
  font-family: monospace;
}

.no-changes {
  text-align: center;
  padding: 40px;
  color: var(--color-muted);
  font-style: italic;
}
</style>
