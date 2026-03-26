<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import type { IdeOption, LocalSkill } from "../../composables/types";

const { t } = useI18n();

type SortKey = "name" | "versions" | "usage";
const sortBy = ref<SortKey>("name");
const collapsedGroups = ref<Set<string>>(new Set());

const props = defineProps<{
  skills: LocalSkill[];
  selectedSkillId: string | null;
  selectedIds: string[];
  searchQuery: string;
  loading: boolean;
  ideOptions: IdeOption[];
  platformFilter: string;
  statusFilter: string;
  platformOptions: Array<{ id: string; label: string; count: number }>;
  statusOptions: Array<{ id: string; label: string; count: number }>;
  projectUsageMap: Map<string, string[]>;
  skillStatusMap: Map<string, string>;
  skillScopeMap: Map<string, string>;
  skillHasNewVersionsMap: Map<string, boolean>;
  skillDefaultVersionMap: Map<string, { displayName: string; versionCount: number }>;
  skillPluginCountMap: Map<string, number>;
  skillPluginOnlyMap: Map<string, boolean>;
  activeStatusTags: Set<string>;
  statusTagCounts: { managed: number; unmanaged: number; pluginOnly: number };
}>();

const emit = defineEmits<{
  (e: "update:searchQuery", value: string): void;
  (e: "update:platformFilter", value: string): void;
  (e: "update:statusFilter", value: string): void;
  (e: "toggleStatusTag", tag: string): void;
  (e: "select", skill: LocalSkill): void;
  (e: "toggleSelected", skillId: string, checked: boolean): void;
  (e: "toggleSelectAll", checked: boolean, filteredIds: string[]): void;
  (e: "installSelected"): void;
  (e: "cloneSelected"): void;
  (e: "deleteSelected"): void;
  (e: "adoptSelected"): void;
  (e: "clearSelection"): void;
  (e: "deleteAll"): void;
  (e: "refresh"): void;
  (e: "import"): void;
}>();

const filteredSkills = computed<LocalSkill[]>(() => props.skills);

function getSkillSortValue(skill: LocalSkill): [number, string] {
  if (sortBy.value === "versions") {
    const vc = props.skillDefaultVersionMap.get(skill.id)?.versionCount ?? skill.versionCount;
    return [-vc, skill.name.toLowerCase()];
  }
  if (sortBy.value === "usage") {
    const usage = skill.usedBy.length + (props.projectUsageMap.get(skill.id)?.length ?? 0);
    return [-usage, skill.name.toLowerCase()];
  }
  return [0, skill.name.toLowerCase()];
}

function sortSkills(skills: LocalSkill[]): LocalSkill[] {
  return [...skills].sort((a, b) => {
    const [av, an] = getSkillSortValue(a);
    const [bv, bn] = getSkillSortValue(b);
    if (av !== bv) return av - bv;
    return an.localeCompare(bn);
  });
}

function getNamespace(skill: LocalSkill): string {
  return skill.currentVersion?.metadata?.namespace || "default";
}

const repoSkills = computed(() => sortSkills(
  filteredSkills.value.filter((s) => props.skillScopeMap.get(s.id) === "repo")
));

const repoNamespaceGroups = computed(() => {
  const map = new Map<string, LocalSkill[]>();
  for (const s of repoSkills.value) {
    const ns = getNamespace(s);
    const group = map.get(ns);
    if (group) group.push(s);
    else map.set(ns, [s]);
  }
  return [...map.entries()].sort((a, b) => {
    if (a[0] === "default") return 1;
    if (b[0] === "default") return -1;
    return a[0].localeCompare(b[0]);
  });
});

function toggleGroup(ns: string) {
  const next = new Set(collapsedGroups.value);
  if (next.has(ns)) next.delete(ns);
  else next.add(ns);
  collapsedGroups.value = next;
}

const globalSkills = computed(() => sortSkills(
  filteredSkills.value.filter((s) => props.skillScopeMap.get(s.id) === "global")
));

const projectSkills = computed(() => sortSkills(
  filteredSkills.value.filter((s) => props.skillScopeMap.get(s.id) === "project")
));

const allSelected = computed<boolean>(() => {
  return filteredSkills.value.length > 0
    && filteredSkills.value.every((skill) => props.selectedIds.includes(skill.id));
});

function getSkillStatus(skill: LocalSkill): { label: string; type: "used" | "unused" | "modified" | "unmanaged" | "conflict" | "outdated" } {
  const status = props.skillStatusMap.get(skill.id);
  if (status === "unmanaged") {
    return { label: t("library.status.unmanaged"), type: "unmanaged" };
  }
  if (status === "modified") {
    return { label: t("library.status.modified"), type: "modified" };
  }
  if (status === "conflict") {
    return { label: t("library.statusConflict"), type: "conflict" };
  }
  if (status === "outdated") {
    return { label: t("library.statusOutdated"), type: "outdated" };
  }

  const projectIds = props.projectUsageMap.get(skill.id);
  if (skill.usedBy.length > 0 || (projectIds && projectIds.length > 0)) {
    return { label: t("library.status.used"), type: "used" };
  }

  return { label: t("library.status.unused"), type: "unused" };
}


function getLinkedIdeCount(skill: LocalSkill): number {
  return skill.usedBy.length;
}

const selectedUnmanagedCount = computed(() =>
  props.selectedIds.filter((id) => props.skillStatusMap.get(id) === "unmanaged").length
);

function getProjectCount(skill: LocalSkill): number {
  return props.projectUsageMap.get(skill.id)?.length ?? 0;
}

function handleToggleAll(checked: boolean): void {
  emit("toggleSelectAll", checked, filteredSkills.value.map((skill) => skill.id));
}
</script>

<template>
  <aside class="library-sidebar panel">
    <div class="sidebar-header">
      <div>
        <div class="panel-title sidebar-title">{{ t("library.title") }}</div>
        <div class="hint sidebar-hint">{{ t("local.hint") }}</div>
      </div>
      <div class="sidebar-actions">
        <button class="ghost icon-btn" :disabled="loading" :title="t('library.refresh')" @click="$emit('refresh')">
          <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="M17.65 6.35A7.95 7.95 0 0 0 12 4a8 8 0 0 0-8 8 8 8 0 0 0 8 8c3.73 0 6.84-2.55 7.73-6h-2.08A5.99 5.99 0 0 1 12 18a6 6 0 0 1-6-6 6 6 0 0 1 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z" fill="currentColor" /></svg>
        </button>
        <button class="primary icon-btn" :disabled="loading" :title="t('library.import')" @click="$emit('import')">
          <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z" fill="currentColor" /></svg>
        </button>
      </div>
    </div>

    <div class="panel-summary sidebar-summary">
      <span>{{ t("library.stats", { count: filteredSkills.length, total: skills.length }) }}</span>
      <label class="checkbox select-all">
        <input type="checkbox" :checked="allSelected" :disabled="filteredSkills.length === 0" @change="handleToggleAll(($event.target as HTMLInputElement).checked)" />
        {{ t("local.selectAll") }}
      </label>
    </div>

    <div v-if="selectedIds.length > 0" class="bulk-actions-bar">
      <div class="bulk-summary">{{ t("local.deleteSelectedCount", { count: selectedIds.length }) }}</div>
      <div class="bulk-actions">
        <button v-if="selectedUnmanagedCount > 0" class="primary btn-sm" @click="$emit('adoptSelected')">{{ t("library.adoptSelectedCount", { count: selectedUnmanagedCount }) }}</button>
        <button class="primary btn-sm" @click="$emit('installSelected')">{{ t("local.installSelected", { count: selectedIds.length }) }}</button>
        <button class="ghost btn-sm" @click="$emit('cloneSelected')">{{ t("library.detail.cloneToProject") }}</button>
        <button class="ghost danger btn-sm" @click="$emit('deleteSelected')">{{ t("local.deleteSelected", { count: selectedIds.length }) }}</button>
        <button class="ghost btn-sm" @click="$emit('clearSelection')">{{ t("common.cancel") }}</button>
      </div>
    </div>

    <div class="search-box">
      <svg class="search-icon" viewBox="0 0 24 24" aria-hidden="true"><path d="M15.5 14h-.79l-.28-.27A6.47 6.47 0 0 0 16 9.5 6.5 6.5 0 1 0 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z" fill="currentColor" /></svg>
      <input :value="searchQuery" type="text" class="input search-input" :placeholder="t('library.searchPlaceholder')" @input="$emit('update:searchQuery', ($event.target as HTMLInputElement).value)" />
    </div>

    <div class="filter-row">
      <label class="filter-group">
        <span class="filter-label">{{ t("library.platformFilter") }}</span>
        <select class="filter-select" :value="platformFilter" @change="$emit('update:platformFilter', ($event.target as HTMLSelectElement).value)">
          <option v-for="option in platformOptions" :key="option.id" :value="option.id">
            {{ option.id === 'all' ? t('common.all') : option.label }}
          </option>
        </select>
      </label>

      <label class="filter-group">
        <span class="filter-label">{{ t("library.sortBy") }}</span>
        <select class="filter-select" :value="sortBy" @change="sortBy = ($event.target as HTMLSelectElement).value as SortKey">
          <option value="name">{{ t("library.sortName") }}</option>
          <option value="versions">{{ t("library.sortVersions") }}</option>
          <option value="usage">{{ t("library.sortUsage") }}</option>
        </select>
      </label>
    </div>

    <div class="status-tags">
      <button class="status-tag" :class="{ active: activeStatusTags.has('managed') }" @click="$emit('toggleStatusTag', 'managed')">
        {{ t("library.status.managed") }} <span class="tag-count">{{ statusTagCounts.managed }}</span>
      </button>
      <button class="status-tag" :class="{ active: activeStatusTags.has('unmanaged') }" @click="$emit('toggleStatusTag', 'unmanaged')">
        {{ t("library.status.unmanaged") }} <span class="tag-count">{{ statusTagCounts.unmanaged }}</span>
      </button>
      <button class="status-tag" :class="{ active: activeStatusTags.has('pluginOnly') }" @click="$emit('toggleStatusTag', 'pluginOnly')">
        {{ t("library.status.pluginOnly") }} <span class="tag-count">{{ statusTagCounts.pluginOnly }}</span>
      </button>
    </div>

    <div class="skills-list">
      <div v-if="loading" class="loading-state hint">
        <div class="loading-spinner" />
        <span>{{ t("library.loading") }}</span>
      </div>

      <div v-else-if="skills.length === 0" class="empty-state hint">
        <svg class="empty-icon" viewBox="0 0 24 24" aria-hidden="true"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6zm4 18H6V4h7v5h5v11z" fill="currentColor" /></svg>
        <span>{{ t("library.empty.list") }}</span>
      </div>

      <div v-else-if="filteredSkills.length === 0" class="empty-state hint">
        <span>{{ t("library.empty.listNoResults") }}</span>
      </div>

      <template v-if="repoSkills.length > 0">
        <template v-for="[ns, skills] in repoNamespaceGroups" :key="ns">
          <div v-if="repoNamespaceGroups.length > 1" class="group-header ns-header" @click="toggleGroup(ns)">
            <span class="collapse-icon">{{ collapsedGroups.has(ns) ? '▸' : '▾' }}</span>
            <span class="ns-label">{{ ns === 'default' ? t("library.groupRepo") : ns }}</span>
            <span class="group-count">{{ skills.length }}</span>
          </div>
          <template v-if="!collapsedGroups.has(ns)">
            <article v-for="skill in skills" :key="skill.id" class="card skill-card" :class="{ active: selectedSkillId === skill.id, linked: getSkillStatus(skill).type === 'used' }">
              <div class="skill-card-top">
                <label class="checkbox card-select" @click.stop>
                  <input type="checkbox" :checked="selectedIds.includes(skill.id)" @change="$emit('toggleSelected', skill.id, ($event.target as HTMLInputElement).checked)" />
                </label>
                <span class="repo-indicator in-repo" :title="t('library.inRepo')">●</span>
                <button class="skill-main" @click="$emit('select', skill)">
                  <div class="skill-item-header">
                    <span class="skill-name">{{ skill.name }}</span>
                    <span v-if="skillPluginOnlyMap.get(skill.id)" class="status-badge plugin-only">{{ t("library.status.pluginOnly") }}</span>
                  </div>
                  <div class="card-meta skill-description-line">{{ skill.description || skill.path }}</div>
                  <div class="skill-meta">
                    <span v-if="skillDefaultVersionMap.get(skill.id)?.displayName || skill.currentVersion" class="version-chip">{{ skillDefaultVersionMap.get(skill.id)?.displayName || skill.currentVersion?.displayName }}</span>
                    <span class="version-meta-text">{{ t("version.totalVersions") }}: {{ skillDefaultVersionMap.get(skill.id)?.versionCount ?? skill.versionCount }}</span>
                    <span v-if="skillHasNewVersionsMap.get(skill.id)" class="new-version-badge">{{ t("library.versions.detected") }}</span>
                    <span v-if="getLinkedIdeCount(skill) > 0" class="ide-count global">{{ t("library.globalIdes", { count: getLinkedIdeCount(skill) }) }}</span>
                    <span v-if="getProjectCount(skill) > 0" class="ide-count project">{{ t("library.projectUsage", { count: getProjectCount(skill) }) }}</span>
                    <span v-if="skillPluginCountMap.get(skill.id)" class="ide-count plugin">{{ t("library.pluginCount", { count: skillPluginCountMap.get(skill.id) }) }}</span>
                  </div>
                </button>
              </div>
            </article>
          </template>
        </template>
      </template>

      <template v-if="globalSkills.length > 0">
        <div class="group-header">{{ t("library.groupGlobal") }} <span class="group-count">{{ globalSkills.length }}</span></div>
        <article v-for="skill in globalSkills" :key="skill.id" class="card skill-card unmanaged" :class="{ active: selectedSkillId === skill.id }">
          <div class="skill-card-top">
            <label class="checkbox card-select" @click.stop>
              <input type="checkbox" :checked="selectedIds.includes(skill.id)" @change="$emit('toggleSelected', skill.id, ($event.target as HTMLInputElement).checked)" />
            </label>
            <span class="repo-indicator not-in-repo" :title="t('library.notInRepo')">○</span>
            <button class="skill-main" @click="$emit('select', skill)">
              <div class="skill-item-header">
                <span class="skill-name">{{ skill.name }}</span>
                <span class="status-badge unmanaged">{{ t("library.status.unmanaged") }}</span>
              </div>
              <div class="card-meta skill-description-line">{{ skill.path }}</div>
            </button>
          </div>
        </article>
      </template>

      <template v-if="projectSkills.length > 0">
        <div class="group-header">{{ t("library.groupProject") }} <span class="group-count">{{ projectSkills.length }}</span></div>
        <article v-for="skill in projectSkills" :key="skill.id" class="card skill-card unmanaged" :class="{ active: selectedSkillId === skill.id }">
          <div class="skill-card-top">
            <label class="checkbox card-select" @click.stop>
              <input type="checkbox" :checked="selectedIds.includes(skill.id)" @change="$emit('toggleSelected', skill.id, ($event.target as HTMLInputElement).checked)" />
            </label>
            <span class="repo-indicator not-in-repo" :title="t('library.notInRepo')">○</span>
            <button class="skill-main" @click="$emit('select', skill)">
              <div class="skill-item-header">
                <span class="skill-name">{{ skill.name }}</span>
                <span class="status-badge unmanaged">{{ t("library.status.unmanaged") }}</span>
              </div>
              <div class="card-meta skill-description-line">{{ skill.path }}</div>
            </button>
          </div>
        </article>
      </template>

    </div>
  </aside>
</template>

<style scoped>
.library-sidebar {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 16px;
  border-radius: 14px 0 0 14px;
  border-right: none;
  box-shadow: none;
}

.sidebar-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.sidebar-title {
  margin-bottom: 2px;
}

.sidebar-hint {
  margin-top: 0;
}

.sidebar-actions {
  display: flex;
  gap: 8px;
}

.icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  padding: 0;
}

.icon {
  width: 16px;
  height: 16px;
}

.sidebar-summary {
  margin-top: 12px;
}

.select-all {
  justify-content: flex-end;
}

.search-box {
  position: relative;
  margin-top: 12px;
}

.bulk-actions-bar {
  margin-top: 12px;
  padding: 10px 12px;
  border: 1px solid var(--color-card-border);
  border-radius: 10px;
  background: var(--color-card-bg);
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.bulk-summary {
  font-size: 12px;
  color: var(--color-muted);
}

.bulk-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.filter-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
  margin-top: 12px;
}

.status-tags {
  display: flex;
  gap: 6px;
  margin-top: 8px;
  flex-wrap: wrap;
}

.status-tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border-radius: 999px;
  border: 1px solid var(--color-card-border);
  background: transparent;
  color: var(--color-muted);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}

.status-tag:hover {
  border-color: var(--color-input-focus);
}

.status-tag.active {
  background: var(--color-primary-bg);
  border-color: var(--color-primary-bg);
  color: var(--color-primary-text);
}

.tag-count {
  font-size: 11px;
  opacity: 0.7;
}

.filter-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.filter-label {
  font-size: 12px;
  color: var(--color-muted);
}

.filter-select {
  padding: 9px 10px;
  border-radius: 10px;
  border: 1px solid var(--color-input-border);
  background: var(--color-input-bg);
  color: var(--color-text);
  font-size: 13px;
}

.search-icon {
  position: absolute;
  left: 12px;
  top: 50%;
  transform: translateY(-50%);
  width: 16px;
  height: 16px;
  color: var(--color-muted);
  pointer-events: none;
}

.search-input {
  padding-left: 36px;
}

.skills-list {
  flex: 1;
  margin-top: 12px;
  overflow-y: auto;
  padding-right: 4px;
}

.loading-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 180px;
  gap: 12px;
  text-align: center;
}

.loading-spinner {
  width: 24px;
  height: 24px;
  border: 2px solid var(--color-panel-border);
  border-top-color: var(--color-primary-bg);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.empty-icon {
  width: 40px;
  height: 40px;
  opacity: 0.45;
}

.skill-card {
  margin-bottom: 10px;
}

.skill-card.active {
  border-color: var(--color-input-focus);
  box-shadow: 0 10px 24px rgba(16, 18, 27, 0.08);
}

.skill-card.linked {
  box-shadow: inset 0 0 0 1px var(--color-success-border);
}

.skill-card-top {
  display: flex;
  align-items: flex-start;
  gap: 10px;
}

.card-select {
  padding-top: 2px;
}

.skill-main {
  flex: 1;
  border: none;
  background: transparent;
  padding: 0;
  text-align: left;
  cursor: pointer;
}

.skill-item-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px;
}

.skill-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text);
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.skill-description-line {
  margin-top: 4px;
}

.status-badge {
  flex-shrink: 0;
  padding: 3px 8px;
  border-radius: 999px;
  font-size: 11px;
  line-height: 1.2;
  font-weight: 600;
}

.status-badge.used {
  background: var(--color-success-bg);
  border: 1px solid var(--color-success-border);
  color: var(--color-success-text);
}

.status-badge.unused {
  background: var(--color-card-bg);
  border: 1px solid var(--color-card-border);
  color: var(--color-muted);
}

.status-badge.modified {
  background: rgba(245, 158, 11, 0.12);
  border: 1px solid rgba(245, 158, 11, 0.3);
  color: #d97706;
}

.status-badge.unmanaged {
  background: var(--color-card-bg);
  border: 1px solid var(--color-card-border);
  color: var(--color-muted);
  font-style: italic;
}

.status-badge.plugin-only {
  background: var(--color-chip-bg);
  border: 1px solid var(--color-chip-border);
  color: var(--color-muted);
  font-style: italic;
}

.status-badge.conflict {
  background: var(--color-error-bg);
  border: 1px solid var(--color-error-border);
  color: var(--color-error-text);
}

.status-badge.outdated {
  background: var(--color-warning-bg);
  border: 1px solid var(--color-warning-border);
  color: var(--color-warning-text);
}

.skill-card.unmanaged {
  opacity: 0.85;
}


.repo-indicator {
  flex-shrink: 0;
  font-size: 10px;
  line-height: 1;
  padding-top: 4px;
}

.repo-indicator.in-repo {
  color: var(--color-success-text);
}

.repo-indicator.not-in-repo {
  color: var(--color-muted);
}

.group-header {
  font-size: 12px;
  font-weight: 700;
  color: var(--color-muted);
  padding: 10px 0 4px;
  display: flex;
  align-items: center;
  gap: 6px;
}

.ns-header {
  cursor: pointer;
  user-select: none;
  border-radius: 6px;
  padding: 6px 4px;
  margin: 2px 0;
}

.ns-header:hover {
  background: var(--color-card-bg);
}

.collapse-icon {
  font-size: 10px;
  width: 12px;
  text-align: center;
  flex-shrink: 0;
}

.ns-label {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.group-count {
  padding: 1px 6px;
  border-radius: 999px;
  font-size: 10px;
  background: var(--color-chip-bg);
  border: 1px solid var(--color-chip-border);
  font-weight: 500;
}

.skill-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  margin-top: 10px;
}

.version-chip {
  padding: 4px 8px;
  border-radius: 999px;
  background: var(--color-chip-bg);
  border: 1px solid var(--color-chip-border);
  font-size: 11px;
  font-weight: 600;
}

.version-meta-text,
.ide-count {
  color: var(--color-muted);
  font-size: 12px;
}

.new-version-badge {
  padding: 2px 6px;
  border-radius: 6px;
  font-size: 10px;
  font-weight: 600;
  background: var(--color-warning-bg);
  border: 1px solid var(--color-warning-border);
  color: var(--color-warning-text);
  white-space: nowrap;
}
</style>
