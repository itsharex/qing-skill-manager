<script setup lang="ts">
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { ref } from "vue";
import type { ProjectConfig, LocalSkill, IdeOption, ProjectSkill } from "../composables/types";
import { useI18n } from "vue-i18n";

const { t } = useI18n();

const props = defineProps<{
  projects: ProjectConfig[];
  selectedProjectId: string | null;
  localSkills: LocalSkill[];
  ideOptions: IdeOption[];
  projectSkillSnapshots?: Record<string, ProjectSkill[]>;
  localLoading: boolean;
}>();

const expandedProjectId = ref<string | null>(null);

const emit = defineEmits<{
  (e: "addProject"): void;
  (e: "removeProject", projectId: string): void;
  (e: "selectProject", projectId: string | null): void;
  (e: "configureProject", projectId: string): void;
  (e: "exportSkills", projectId: string): void;
  (e: "importSkills", projectId: string): void;
}>();

async function handleOpenDirectory(project: ProjectConfig) {
  try {
    await revealItemInDir(project.path);
  } catch (err) {
    console.error("Failed to open directory:", err);
  }
}

function getProjectSkillStats(projectId: string) {
  const skills = props.projectSkillSnapshots?.[projectId] ?? [];
  const conflicts = skills.filter((skill) => skill.status === "conflict").length;
  const synced = skills.filter((skill) => skill.status === "duplicate" || skill.status === "managed_version").length;
  return { conflicts, synced, total: skills.length };
}

function toggleProjectDetails(projectId: string) {
  expandedProjectId.value = expandedProjectId.value === projectId ? null : projectId;
}

function getProjectSkillDetails(projectId: string) {
  const skills = props.projectSkillSnapshots?.[projectId] ?? [];
  return skills.filter((skill) => skill.status === "conflict" || skill.status === "duplicate" || skill.status === "managed_version");
}
</script>

<template>
  <section class="panel">
    <div class="panel-header">
      <div class="panel-title">{{ t("projects.title") }}</div>
      <button class="primary" @click="emit('addProject')">
        {{ t("projects.add") }}
      </button>
    </div>
    <div class="hint">{{ t("projects.hint") }}</div>

    <div v-if="projects.length === 0" class="hint">{{ t("projects.emptyHint") }}</div>

    <div v-else class="project-list">
      <div v-for="project in projects" :key="project.id" class="project-item">
        <div class="project-header">
          <div class="project-info">
            <div class="project-name">{{ project.name }}</div>
            <div class="project-path">{{ project.path }}</div>
          </div>
          <div class="project-header-actions">
            <button class="ghost btn-sm" :title="t('projects.openDirectory')" @click="handleOpenDirectory(project)">
              <svg class="btn-icon" viewBox="0 0 24 24"><path d="M20 6h-8l-2-2H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2zm0 12H4V8h16v10z" fill="currentColor"/></svg>
            </button>
            <button class="ghost btn-sm" :title="t('projects.configure')" @click="emit('configureProject', project.id)">
              <svg class="btn-icon" viewBox="0 0 24 24"><path d="M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58a.49.49 0 0 0 .12-.61l-1.92-3.32a.49.49 0 0 0-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54a.484.484 0 0 0-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.07.62-.07.94s.02.64.07.94l-2.03 1.58a.49.49 0 0 0-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6A3.6 3.6 0 1 1 12 8.4a3.6 3.6 0 0 1 0 7.2z" fill="currentColor"/></svg>
            </button>
            <button class="ghost danger btn-sm" :title="t('projects.remove')" @click="emit('removeProject', project.id)">
              <svg class="btn-icon" viewBox="0 0 24 24"><path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z" fill="currentColor"/></svg>
            </button>
          </div>
        </div>

        <div class="project-meta">
          <span v-for="ide in project.ideTargets" :key="ide" class="ide-badge active">{{ ide }}</span>
          <span v-if="project.ideTargets.length === 0" class="ide-badge">{{ t("projects.noIdeTargets") }}</span>
        </div>

        <div v-if="getProjectSkillStats(project.id).total > 0" class="stats-row">
          <span v-if="getProjectSkillStats(project.id).synced > 0" class="meta-item success">
            {{ t("projects.syncedCount", { count: getProjectSkillStats(project.id).synced }) }}
          </span>
          <span v-if="getProjectSkillStats(project.id).conflicts > 0" class="meta-item warning">
            {{ t("projects.conflictCount", { count: getProjectSkillStats(project.id).conflicts }) }}
          </span>
          <span class="meta-item">
            {{ t("projects.totalSkills", { count: getProjectSkillStats(project.id).total }) }}
          </span>
          <button class="ghost btn-xs toggle-detail" @click="toggleProjectDetails(project.id)">
            {{ expandedProjectId === project.id ? t("projects.hideSkillDetails") : t("projects.showSkillDetails") }}
          </button>
        </div>

        <div class="project-primary-actions">
          <button class="primary btn-sm" @click="emit('exportSkills', project.id)">
            {{ t("projects.scanProjectSkills") }}
          </button>
          <button class="ghost btn-sm" :disabled="localLoading" @click="emit('importSkills', project.id)">
            {{ t("projects.cloneSkillsToProject") }}
          </button>
        </div>

        <div v-if="expandedProjectId === project.id" class="project-skill-details">
          <div v-if="getProjectSkillDetails(project.id).length === 0" class="detail-empty">
            {{ t("projects.noMonitoredSkillChanges") }}
          </div>
          <div v-for="skill in getProjectSkillDetails(project.id)" :key="skill.path" class="detail-item">
            <div class="detail-name-row">
              <span class="detail-name">{{ skill.name }}</span>
              <span class="detail-status" :class="skill.status">{{ skill.status }}</span>
            </div>
            <div class="detail-path">{{ skill.path }}</div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.panel-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text);
  margin: 0;
}

.project-list {
  margin-top: 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.project-item {
  padding: 14px 16px;
  background: var(--color-card-bg);
  border: 1px solid var(--color-card-border);
  border-radius: 10px;
}

.project-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
}

.project-info {
  flex: 1;
  min-width: 0;
}

.project-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text);
  margin-bottom: 2px;
}

.project-path {
  font-size: 11px;
  color: var(--color-muted);
  word-break: break-all;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.project-header-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.btn-icon {
  width: 14px;
  height: 14px;
}

.project-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
  margin-top: 10px;
}

.ide-badge {
  padding: 3px 8px;
  border-radius: 999px;
  border: 1px solid var(--color-chip-border);
  background: transparent;
  color: var(--color-muted);
  font-size: 11px;
  line-height: 1.2;
}

.ide-badge.active {
  border-color: var(--color-success-border);
  background: var(--color-success-bg);
  color: var(--color-success-text);
  font-weight: 600;
}

.stats-row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 8px;
}

.meta-item {
  padding: 2px 8px;
  background: var(--color-chip-bg);
  border-radius: 999px;
  font-size: 11px;
  color: var(--color-muted);
}

.meta-item.warning {
  background: var(--color-warning-bg);
  color: var(--color-warning-text);
}

.meta-item.success {
  background: var(--color-success-bg);
  color: var(--color-success-text);
}

.toggle-detail {
  margin-left: auto;
}

.project-primary-actions {
  display: flex;
  gap: 8px;
  margin-top: 10px;
}

.project-skill-details {
  margin-top: 10px;
  padding: 10px;
  border-radius: 8px;
  border: 1px solid var(--color-card-border);
  background: var(--color-bg);
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.detail-empty {
  color: var(--color-muted);
  font-size: 12px;
}

.detail-item {
  padding: 8px 10px;
  border-radius: 6px;
  background: var(--color-card-bg);
}

.detail-name-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.detail-name {
  font-size: 13px;
  font-weight: 600;
}

.detail-status {
  font-size: 10px;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: 999px;
  background: var(--color-chip-bg);
  color: var(--color-muted);
}

.detail-status.conflict {
  background: var(--color-warning-bg);
  color: var(--color-warning-text);
}

.detail-status.duplicate,
.detail-status.managed_version {
  background: var(--color-success-bg);
  color: var(--color-success-text);
}

.detail-path {
  font-size: 11px;
  color: var(--color-muted);
  margin-top: 2px;
}
</style>
