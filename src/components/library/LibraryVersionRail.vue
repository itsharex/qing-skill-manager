<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { LocalSkill, SkillPackage, SkillVersion } from "../../composables/types";

const { t } = useI18n();

const props = defineProps<{
  skill: LocalSkill | null;
  selectedVersionId: string | null;
  skillPackage: SkillPackage | null;
  loading: boolean;
}>();

defineEmits<{
  (e: "selectVersion", version: SkillVersion): void;
  (e: "compareVersions", versionId: string): void;
  (e: "createVersion"): void;
  (e: "setDefault", versionId: string): void;
}>();

const sortedVersions = computed<SkillVersion[]>(() => {
  return [...(props.skillPackage?.versions || [])].sort((a, b) => b.createdAt - a.createdAt);
});

function isDefault(versionId: string): boolean {
  return versionId === props.skillPackage?.defaultVersion;
}

function isSelected(versionId: string): boolean {
  return versionId === props.selectedVersionId;
}
</script>

<template>
  <aside class="library-version-rail panel">
    <div class="rail-header">
      <div>
        <div class="panel-title rail-title">{{ t("library.versions.title") }}</div>
        <div class="hint">{{ t("library.versionRailHint") }}</div>
      </div>
      <button v-if="skill" class="primary icon-btn" @click="$emit('createVersion')">+</button>
    </div>

    <div v-if="!skill" class="empty-state hint">{{ t("library.versions.noSkill") }}</div>
    <div v-else-if="loading" class="empty-state hint">{{ t("library.versions.loading") }}</div>
    <div v-else-if="sortedVersions.length === 0" class="empty-state hint">{{ t("library.versions.empty") }}</div>

    <div v-else class="versions-list">
      <article v-for="version in sortedVersions" :key="version.id" class="card version-card" :class="{ active: isSelected(version.id), default: isDefault(version.id) }">
        <button class="version-main" @click="$emit('selectVersion', version)">
          <div class="version-header">
            <div class="card-title">{{ version.displayName }}</div>
            <div class="version-badges">
              <span v-if="isDefault(version.id)" class="badge success">{{ t("library.versions.default") }}</span>
              <span v-if="skill.currentVersion?.id === version.id" class="badge muted">{{ t("library.versions.active") }}</span>
            </div>
          </div>
          <div class="card-meta">{{ version.version }} · {{ new Date(version.createdAt).toLocaleDateString() }}</div>
          <div class="version-source">{{ version.source }}</div>
        </button>
        <div class="version-actions">
          <button v-if="!isDefault(version.id)" class="ghost action-btn" @click="$emit('setDefault', version.id)">{{ t("library.versions.setDefault") }}</button>
          <button v-if="skill.currentVersion?.id !== version.id" class="ghost action-btn" @click="$emit('compareVersions', version.id)">{{ t("library.versions.compare") }}</button>
        </div>
      </article>
    </div>
  </aside>
</template>

<style scoped>
.library-version-rail {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 16px;
  border-radius: 0 14px 14px 0;
  box-shadow: none;
}

.rail-header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: flex-start;
}

.rail-title {
  margin-bottom: 2px;
}

.icon-btn {
  width: 34px;
  height: 34px;
  padding: 0;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 160px;
  text-align: center;
}

.versions-list {
  margin-top: 12px;
  overflow-y: auto;
  padding-right: 4px;
}

.version-card {
  margin-bottom: 10px;
}

.version-card.active {
  border-color: var(--color-input-focus);
}

.version-card.default {
  box-shadow: inset 0 0 0 1px var(--color-success-border);
}

.version-main {
  width: 100%;
  border: none;
  background: transparent;
  padding: 0;
  text-align: left;
  cursor: pointer;
}

.version-main:hover .card-title {
  color: var(--color-primary-bg);
}

.version-header {
  display: flex;
  justify-content: space-between;
  gap: 8px;
  align-items: flex-start;
}

.version-badges {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.badge {
  padding: 3px 8px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
}

.badge.success {
  background: var(--color-success-bg);
  border: 1px solid var(--color-success-border);
  color: var(--color-success-text);
}

.badge.muted {
  background: var(--color-chip-bg);
  border: 1px solid var(--color-chip-border);
  color: var(--color-text);
}

.version-source {
  margin-top: 8px;
  color: var(--color-muted);
  font-size: 12px;
  overflow-wrap: anywhere;
}

.version-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  margin-top: 10px;
}

.action-btn {
  padding: 6px 10px;
}
</style>
