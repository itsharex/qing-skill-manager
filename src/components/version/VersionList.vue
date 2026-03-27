<script setup lang="ts">
import { useI18n } from "vue-i18n";
import type { SkillPackage, SkillVersion } from "../../composables/types";

defineProps<{
  skillPackage: SkillPackage;
  defaultVersion: SkillVersion | null;
  otherVersions: SkillVersion[];
}>();

const emit = defineEmits<{
  (e: "setDefault", versionId: string): void;
  (e: "compare", fromVersionId: string, toVersionId: string): void;
  (e: "openRename", version: SkillVersion): void;
  (e: "openDelete", version: SkillVersion): void;
}>();

const { t } = useI18n();

function formatDate(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleString();
}

function getSourceLabel(source: string): string {
  const labels: Record<string, string> = {
    market: t("version.sourceMarket"),
    project: t("version.sourceProject"),
    import: t("version.sourceImport"),
    clone: t("version.sourceClone"),
    migration: t("version.sourceMigration")
  };
  return labels[source] || source;
}
</script>

<template>
  <section class="versions-list section-card">
    <div class="section-header">
      <div>
        <h4>{{ t("version.allVersions") }}</h4>
        <p class="section-help">{{ t("version.versionListHelp") }}</p>
      </div>
    </div>

    <div v-if="defaultVersion" class="version-item default focused-default">
      <div class="version-header">
        <div class="version-badges">
          <span class="badge default-badge">
            {{ t("version.default") }}
          </span>
        </div>
        <div>
          <div class="version-name">{{ defaultVersion.displayName }}</div>
          <div class="version-summary-text">{{ t("version.currentVersionSummary") }}</div>
        </div>
      </div>

      <div class="version-meta">
        <div class="meta-row">
          <span class="meta-label">{{ t("version.defaultVersionId") }}:</span>
          <span class="meta-value code">{{ defaultVersion.version }}</span>
        </div>
        <div class="meta-row">
          <span class="meta-label">{{ t("version.source") }}:</span>
          <span class="meta-value">{{ getSourceLabel(defaultVersion.source) }}</span>
        </div>
        <div class="meta-row">
          <span class="meta-label">{{ t("version.createdAt") }}:</span>
          <span class="meta-value">{{ formatDate(defaultVersion.createdAt) }}</span>
        </div>
      </div>
    </div>

    <div
      v-for="version in otherVersions"
      :key="version.id"
      class="version-item"
      :class="{ active: version.isActive }"
    >
      <div class="version-header">
        <div class="version-badges">
          <span v-if="version.isActive" class="badge active-badge">
            {{ t("version.active") }}
          </span>
        </div>
        <div>
          <div class="version-name">{{ version.displayName }}</div>
          <div class="version-summary-text">{{ t("version.switchToThisVersion") }}</div>
        </div>
      </div>

      <div class="version-meta">
        <div class="meta-row">
          <span class="meta-label">{{ t("version.versionId") }}:</span>
          <span class="meta-value code">{{ version.version }}</span>
        </div>
        <div class="meta-row">
          <span class="meta-label">{{ t("version.contentHash") }}:</span>
          <span class="meta-value code">{{ version.contentHash.slice(0, 8) }}</span>
        </div>
        <div class="meta-row">
          <span class="meta-label">{{ t("version.createdAt") }}:</span>
          <span class="meta-value">{{ formatDate(version.createdAt) }}</span>
        </div>
        <div class="meta-row">
          <span class="meta-label">{{ t("version.source") }}:</span>
          <span class="meta-value">{{ getSourceLabel(version.source) }}</span>
        </div>
      </div>

      <div class="version-actions">
        <button class="primary" @click="emit('setDefault', version.id)">
          {{ t("version.setAsDefault") }}
        </button>
        <button
          class="ghost"
          @click="emit('compare', skillPackage.defaultVersion, version.id)"
        >
          {{ t("version.compareWithCurrent") }}
        </button>
        <button class="ghost" @click="emit('openRename', version)">
          {{ t("version.rename") }}
        </button>
        <button
          class="ghost danger btn-sm"
          :disabled="version.id === skillPackage?.defaultVersion"
          @click="emit('openDelete', version)"
        >
          {{ t("version.delete") }}
        </button>
      </div>
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

.section-help,
.version-summary-text {
  margin: 8px 0 0 0;
  font-size: 13px;
  color: var(--color-muted);
}

.versions-list h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
}

.version-item {
  background: var(--color-card-bg);
  border: 1px solid var(--color-card-border);
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 12px;
}

.version-item.active {
  border-color: var(--color-success-border);
  box-shadow: inset 0 0 0 1px var(--color-success-border);
}

.version-item.default {
  border-color: var(--color-primary-bg);
  box-shadow: inset 0 0 0 1px var(--color-primary-bg);
}

.focused-default {
  margin-bottom: 12px;
  background: color-mix(in srgb, var(--color-card-bg) 92%, var(--color-primary-bg) 8%);
}

.version-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.version-badges {
  display: flex;
  gap: 6px;
}

.badge {
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
}

.default-badge {
  background: var(--color-primary-bg);
  color: var(--color-primary-text);
}

.active-badge {
  background: var(--color-success-bg);
  color: var(--color-success-text);
  border: 1px solid var(--color-success-border);
}

.version-name {
  font-size: 15px;
  font-weight: 600;
}

.version-meta {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 8px;
  margin-bottom: 12px;
  padding: 12px;
  background: var(--color-bg);
  border-radius: 6px;
}

.meta-row {
  display: flex;
  gap: 6px;
}

.meta-label {
  color: var(--color-muted);
  font-size: 12px;
}

.meta-value {
  font-size: 12px;
}

.meta-value.code {
  font-family: monospace;
  background: var(--color-card-bg);
  padding: 2px 6px;
  border-radius: 4px;
}

.version-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.version-actions button {
  padding: 6px 12px;
  font-size: 12px;
}

button.danger {
  background: var(--color-error-bg);
  color: var(--color-error-text);
  border-color: var(--color-error-border);
}

button.danger:hover {
  background: var(--color-error-border);
}
</style>
