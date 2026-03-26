<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { IdeOption, LibrarySkill, LocalSkill, ProjectConfig } from "../../composables/types";

const { t } = useI18n();

const props = defineProps<{
  skill: LocalSkill | null;
  librarySkill: LibrarySkill | null;
  selectedVersionId: string | null;
  installingId: string | null;
  ideOptions: IdeOption[];
  projects: ProjectConfig[];
}>();

defineEmits<{
  (e: "install", skill: LocalSkill): void;
  (e: "cloneToProject", projectId: string): void;
  (e: "openDir", path: string): void;
  (e: "delete", skill: LocalSkill): void;
  (e: "adoptToRepo", path: string): void;
  (e: "uninstallSkill", path: string): void;
}>();

const isInstalling = computed<boolean>(() => {
  return !!props.skill && props.installingId === props.skill.id;
});

const globalInstallations = computed(() => {
  const all = props.librarySkill?.installations.filter((i) => i.scope === "global") || [];
  if (!props.selectedVersionId) return all;
  return all.filter((i) => i.versionId === props.selectedVersionId);
});

const versionProjectMappings = computed(() => {
  const all = props.librarySkill?.projectMappings || [];
  if (!props.selectedVersionId) return all;
  return all.filter((m) => m.versionId === props.selectedVersionId);
});

const selectedVersionName = computed(() => {
  if (!props.selectedVersionId || !props.librarySkill) return null;
  const v = props.librarySkill.versions.find((v) => v.id === props.selectedVersionId);
  return v?.displayName || null;
});

function getSyncBadgeClass(status: string): string {
  if (status === "synced" || status === "untracked") return "success";
  if (status === "modified") return "warning";
  return "muted";
}

function getSyncLabel(status: string): string {
  if (status === "synced" || status === "untracked") return t("library.syncSynced");
  if (status === "modified") return t("library.syncModified");
  return t("library.syncUnknown");
}



const cloneProjects = computed(() => {
  if (!props.projects.length) {
    return [];
  }

  return props.projects.filter((project) => project.ideTargets.length > 0).map((project) => {
    const mapping = props.librarySkill?.projectMappings.find((item) => item.projectId === project.id) || null;
    return {
      id: project.id,
      name: project.name,
      ideTargets: project.ideTargets,
      mapped: !!mapping && mapping.status !== "missing",
      status: mapping?.status || "missing"
    };
  });
});

function getMappingBadgeClass(status: string): string {
  if (status === "synced") return "success";
  if (status === "conflict") return "danger";
  if (status === "modified") return "warning";
  return "muted";
}

function getMappingLabel(status: string): string {
  if (status === "synced") return t("library.mappingStatusSynced");
  if (status === "conflict") return t("library.mappingStatusConflict");
  if (status === "modified") return t("library.mappingStatusModified");
  return t("library.mappingStatusMissing");
}

function getMappingDescription(status: string): string {
  if (status === "synced") return t("library.mappingDescSynced");
  if (status === "conflict") return t("library.mappingDescConflict");
  if (status === "modified") return t("library.mappingDescModified");
  return t("library.mappingDescMissing");
}
</script>

<template>
  <main class="library-detail-panel">
    <div v-if="!skill || !librarySkill" class="empty-state">
      <div class="empty-content">
        <h3 class="empty-title">{{ t("library.detail.selectSkill") }}</h3>
        <p class="empty-desc">{{ t("library.detail.selectSkillDesc") }}</p>
      </div>
    </div>

    <!-- Unmanaged skill detail -->
    <div v-else-if="!librarySkill.inRepo" class="skill-detail">
      <div class="detail-header">
        <div class="skill-identity">
          <h1 class="skill-title"><span class="repo-dot not-in-repo">○</span> {{ skill.name }}</h1>
          <div class="skill-subtitle">
            <span class="status-badge unmanaged">{{ t("library.status.unmanaged") }}</span>
            <span class="version-meta-text">{{ librarySkill.unmanagedSources.length }} {{ t("library.versions.locations") }}</span>
          </div>
        </div>
        <div class="header-actions">
          <button class="primary" @click="$emit('adoptToRepo', librarySkill.unmanagedSources[0]?.path || skill.path)">{{ t("library.adoptToRepo") }}</button>
        </div>
      </div>

      <section class="panel hero-panel">
        <p class="card-desc">{{ t("library.unmanagedDesc") }}</p>
      </section>

      <section class="panel section-panel">
        <div class="section-title-row">
          <div class="panel-title section-title-text">{{ t("library.versions.sources") }}</div>
        </div>
        <div class="install-list">
          <div v-for="src in librarySkill.unmanagedSources" :key="src.path" class="install-entry">
            <div class="install-info">
              <span class="install-ide">{{ src.label }}</span>
              <span class="mapping-badge muted">{{ src.ide }} · {{ src.scope === "global" ? t("ide.scopeGlobal") : t("ide.scopeProject") }}</span>
            </div>
            <div class="install-actions">
              <button class="ghost btn-xs" @click="$emit('openDir', src.path)">{{ t("ide.openDir") }}</button>
              <button class="ghost danger btn-xs" @click="$emit('uninstallSkill', src.path)">{{ t("ide.uninstall") }}</button>
            </div>
          </div>
        </div>
      </section>
    </div>

    <!-- Managed skill detail -->
    <div v-else class="skill-detail">
      <div class="detail-header">
        <div class="skill-identity">
          <h1 class="skill-title"><span class="repo-dot in-repo">●</span> {{ skill.name }}</h1>
          <div class="skill-subtitle">
            <span v-if="skill.currentVersion" class="version-chip">{{ skill.currentVersion.displayName }}</span>
            <span class="version-meta-text">{{ t("library.detail.versionCount", { count: skill.versionCount }) }}</span>
            <span class="version-meta-text">{{ t("library.usedInProjects", { count: librarySkill.usedByProjectIds.length }) }}</span>
          </div>
        </div>

        <div class="header-actions">
          <button class="ghost" @click="$emit('openDir', skill.path)">{{ t("library.detail.openRepoDir") }}</button>
          <button class="ghost danger btn-sm" @click="$emit('delete', skill)">{{ t("library.detail.deleteFromRepo") }}</button>
        </div>
      </div>

      <section class="panel hero-panel">
        <p class="card-desc">{{ skill.description || t("library.detail.noDescription") }}</p>
        <div class="detail-meta-row">
          <span class="detail-label">{{ t("library.detail.path") }}</span>
          <code class="card-link path-value">{{ skill.path }}</code>
        </div>
        <div v-if="skill.source" class="detail-meta-row">
          <span class="detail-label">{{ t("library.detail.source") }}</span>
          <span>{{ skill.source }}</span>
        </div>
        <div class="actions buttons">
          <button class="primary" :disabled="isInstalling" @click="$emit('install', skill)">
            {{ isInstalling ? t("library.detail.installing") : t("library.detail.installToIde") }}
          </button>
        </div>
      </section>

      <div v-if="selectedVersionName" class="version-filter-bar">
        <span class="version-filter-label">{{ t("library.filterByVersion") }}</span>
        <span class="version-chip">{{ selectedVersionName }}</span>
      </div>

      <section class="panel section-panel">
        <div class="section-title-row">
          <div class="panel-title section-title-text">{{ t("library.globalInstallations") }}</div>
          <div class="hint">{{ globalInstallations.length > 0 ? `${globalInstallations.length} IDE` : "" }}</div>
        </div>
        <div v-if="globalInstallations.length === 0" class="hint">{{ selectedVersionId ? t("library.noInstallForVersion") : t("library.notInstalled") }}</div>
        <div v-else class="install-list">
          <div v-for="inst in globalInstallations" :key="inst.skillPath" class="install-entry">
            <div class="install-info">
              <span class="install-ide">{{ inst.ideLabel }}</span>
              <span class="mapping-badge" :class="getSyncBadgeClass(inst.syncStatus)">{{ getSyncLabel(inst.syncStatus) }}</span>
            </div>
            <div class="install-actions">
              <button class="ghost btn-xs" @click="$emit('openDir', inst.skillPath)">{{ t("ide.openDir") }}</button>
              <button class="ghost danger btn-xs" @click="$emit('uninstallSkill', inst.skillPath)">{{ t("ide.uninstall") }}</button>
            </div>
          </div>
        </div>
      </section>

      <section class="panel section-panel">
        <div class="section-title-row">
          <div class="panel-title section-title-text">{{ t("library.projectDeployments") }}</div>
        </div>
        <div v-if="versionProjectMappings.length === 0" class="hint">{{ selectedVersionId ? t("library.noProjectForVersion") : t("library.notUsedInProjects") }}</div>
        <div v-else class="mapping-list">
          <article v-for="mapping in versionProjectMappings" :key="mapping.projectId" class="card mapping-card">
            <div class="mapping-header">
              <div class="mapping-title-block">
                <div class="card-title">{{ mapping.projectName }}</div>
                <span class="mapping-badge" :class="getMappingBadgeClass(mapping.status)">{{ getMappingLabel(mapping.status) }}</span>
              </div>
            </div>
            <div class="mapping-detail">
              <div class="mapping-detail-row">
                <span class="detail-label">{{ t("library.detail.path") }}</span>
                <code class="card-link path-value">{{ mapping.projectPath }}</code>
              </div>
              <div class="mapping-detail-row">
                <span class="detail-label">{{ t("library.versionLabel") }}</span>
                <span>{{ mapping.versionName || t("library.mappingEmptyVersion") }}</span>
              </div>
              <div class="mapping-detail-row">
                <span class="detail-label">IDE</span>
                <span>{{ mapping.ideTargets.join(", ") || "—" }}</span>
              </div>
              <div class="mapping-status-desc hint">{{ getMappingDescription(mapping.status) }}</div>
            </div>
            <div class="mapping-action">
              <template v-if="mapping.status === 'missing'">
                <button class="ghost btn-sm" @click="$emit('cloneToProject', mapping.projectId)">{{ t("library.actions.clone") }}</button>
              </template>
              <template v-else>
                <button class="ghost btn-xs" @click="$emit('openDir', mapping.projectPath)">{{ t("ide.openDir") }}</button>
                <button class="ghost danger btn-xs" @click="$emit('uninstallSkill', mapping.projectPath + '/' + (skill?.name || ''))">{{ t("ide.uninstall") }}</button>
              </template>
            </div>
          </article>
        </div>
        <div v-if="!selectedVersionId && cloneProjects.some((p) => !p.mapped)" class="clone-grid">
          <button v-for="project in cloneProjects.filter((p) => !p.mapped)" :key="project.id" class="ghost clone-button" @click="$emit('cloneToProject', project.id)">
            <span>{{ t("library.actions.clone") }} · {{ project.name }}</span>
            <span class="hint">{{ project.ideTargets.join(", ") || t("projects.emptyHint") }}</span>
          </button>
        </div>
      </section>
    </div>
  </main>
</template>

<style scoped>
.library-detail-panel {
  flex: 1;
  min-width: 0;
  height: 100%;
  overflow-y: auto;
  padding: 0 16px;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
}

.empty-content {
  text-align: center;
}

.empty-title {
  margin: 0;
  font-size: 18px;
}

.empty-desc {
  margin-top: 8px;
  color: var(--color-muted);
}

.skill-detail {
  padding: 16px 0;
}

.detail-header {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  align-items: flex-start;
  margin-bottom: 16px;
}

.skill-title {
  margin: 0;
  font-size: 24px;
}

.skill-subtitle {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  margin-top: 8px;
}

.header-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.hero-panel,
.section-panel {
  margin-bottom: 16px;
}

.detail-meta-row {
  display: flex;
  gap: 12px;
  margin-top: 12px;
  align-items: flex-start;
}

.detail-label {
  min-width: 60px;
  color: var(--color-muted);
  font-size: 13px;
}

.path-value {
  margin: 0;
}

.section-title-row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: baseline;
  margin-bottom: 12px;
}

.section-title-text {
  margin-bottom: 0;
}

.ide-badges {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.ide-badge {
  padding: 4px 8px;
  border-radius: 999px;
  border: 1px solid var(--color-chip-border);
  background: transparent;
  color: var(--color-meta);
  font-size: 11px;
}

.ide-badge.active {
  border-color: var(--color-success-border);
  background: var(--color-success-bg);
  color: var(--color-success-text);
  font-weight: 600;
}

.mapping-list {
  display: grid;
  gap: 12px;
}

.mapping-card {
  padding: 12px 14px;
}

.mapping-header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: flex-start;
}

.mapping-title-block {
  display: flex;
  align-items: center;
  gap: 10px;
}

.mapping-detail {
  margin-top: 10px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.mapping-detail-row {
  display: flex;
  gap: 10px;
  align-items: baseline;
  font-size: 13px;
}

.mapping-status-desc {
  margin-top: 4px;
  font-size: 12px;
  font-style: italic;
}

.mapping-action {
  margin-top: 8px;
}

.mapping-header .card-title,
.mapping-header .card-meta,
.card-desc,
.path-value,
.clone-button span:first-child,
.clone-button .hint {
  min-width: 0;
  overflow-wrap: anywhere;
}

.mapping-badge {
  padding: 3px 8px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
}

.mapping-badge.success {
  background: var(--color-success-bg);
  border: 1px solid var(--color-success-border);
  color: var(--color-success-text);
}

.mapping-badge.danger {
  background: var(--color-error-bg);
  border: 1px solid var(--color-error-border);
  color: var(--color-error-text);
}

.mapping-badge.warning {
  background: var(--color-chip-bg);
  border: 1px solid var(--color-chip-border);
  color: var(--color-text);
}

.mapping-badge.muted {
  background: var(--color-card-bg);
  border: 1px solid var(--color-card-border);
  color: var(--color-muted);
}

.clone-grid {
  display: grid;
  gap: 10px;
  margin-top: 12px;
}

.clone-button {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: center;
  text-align: left;
}

.clone-button .hint {
  text-align: right;
}

.install-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.install-entry {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 10px;
  border-radius: 8px;
  background: var(--color-card-bg);
  border: 1px solid var(--color-card-border);
}

.install-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.install-ide {
  font-weight: 600;
  font-size: 13px;
}

.install-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.btn-xs {
  padding: 2px 6px;
  font-size: 11px;
}

.repo-dot {
  font-size: 12px;
  margin-right: 6px;
}

.repo-dot.in-repo { color: var(--color-success-text); }
.repo-dot.not-in-repo { color: var(--color-muted); }

.version-filter-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  margin-bottom: 12px;
  border-radius: 8px;
  background: var(--color-chip-bg);
  border: 1px solid var(--color-chip-border);
  font-size: 13px;
}

.version-filter-label {
  color: var(--color-muted);
}

.status-badge.unmanaged {
  background: var(--color-card-bg);
  border: 1px solid var(--color-card-border);
  color: var(--color-muted);
  padding: 3px 8px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
}
</style>
