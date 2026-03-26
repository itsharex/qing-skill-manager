<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import type { LibrarySkill, LocalSkill, SkillPackage, SkillVersion } from "../../composables/types";

const { t } = useI18n();

const props = defineProps<{
  skill: LocalSkill | null;
  librarySkill: LibrarySkill | null;
  selectedVersionId: string | null;
  skillPackage: SkillPackage | null;
  loading: boolean;
}>();

const emit = defineEmits<{
  (e: "selectVersion", version: SkillVersion): void;
  (e: "compareVersions", versionId: string): void;
  (e: "createVersion"): void;
  (e: "setDefault", versionId: string): void;
  (e: "registerVersion", sourcePath: string): void;
  (e: "renameVersion", versionId: string, newName: string): void;
  (e: "deleteVersion", versionId: string): void;
}>();

const editingVersionId = ref<string | null>(null);
const editingName = ref("");

function startRename(version: SkillVersion): void {
  editingVersionId.value = version.id;
  editingName.value = version.displayName;
}

function confirmRename(versionId: string): void {
  const trimmed = editingName.value.trim();
  if (trimmed && trimmed !== editingVersionId.value) {
    emit("renameVersion", versionId, trimmed);
  }
  editingVersionId.value = null;
}

function cancelRename(): void {
  editingVersionId.value = null;
}

const sortedVersions = computed<SkillVersion[]>(() => {
  return [...(props.skillPackage?.versions || [])].sort((a, b) => b.createdAt - a.createdAt);
});

function isDefault(versionId: string): boolean {
  return versionId === props.skillPackage?.defaultVersion;
}

function isSelected(versionId: string): boolean {
  return versionId === props.selectedVersionId;
}

function getVersionUsage(versionId: string): { ideCount: number; projectCount: number } {
  const vs = props.librarySkill?.versions.find((v) => v.id === versionId);
  return { ideCount: vs?.ideCount ?? 0, projectCount: vs?.projectCount ?? 0 };
}

function getVersionDeployments(versionId: string) {
  // Only global installations — project-level shown via project mappings below
  const installations = props.librarySkill?.installations.filter(
    (i) => i.versionId === versionId && i.scope === "global"
  ) || [];
  const projects = props.librarySkill?.projectMappings.filter((p) => p.versionId === versionId) || [];
  return { installations, projects };
}

function getSyncClass(status: string): string {
  if (status === "synced" || status === "untracked") return "sync-ok";
  if (status === "modified") return "sync-warn";
  return "sync-muted";
}

function getSyncIcon(status: string): string {
  if (status === "synced" || status === "untracked") return "\u2713";
  if (status === "modified") return "\u26a0";
  return "?";
}

// Detect unregistered versions: only truly unmatched copies
const detectedVersions = computed(() => {
  if (!props.librarySkill) return [];
  const results: Array<{ id: string; label: string; scope: string; path: string }> = [];
  const seenPaths = new Set<string>();

  // Collect project paths that have conflict status
  const conflictProjectPaths = new Set<string>();
  for (const pm of props.librarySkill.projectMappings) {
    if (pm.status === "conflict") {
      conflictProjectPaths.add(pm.projectPath);
    }
  }

  // Global installations that are genuinely modified
  for (const inst of props.librarySkill.installations) {
    if (inst.syncStatus !== "modified") continue;
    // Skip if this path is inside a conflict project (will be shown via project mapping)
    const isProjectCopy = conflictProjectPaths.has(inst.skillPath) ||
      [...conflictProjectPaths].some((pp) => inst.skillPath.startsWith(pp + "/"));
    if (isProjectCopy) continue;

    if (!seenPaths.has(inst.skillPath)) {
      seenPaths.add(inst.skillPath);
      results.push({
        id: `detected_${inst.skillPath}`,
        label: `${inst.ideLabel} (${t("ide.scopeGlobal")})`,
        scope: "global",
        path: inst.skillPath
      });
    }
  }

  // Project mappings with conflict
  for (const pm of props.librarySkill.projectMappings) {
    if (pm.status !== "conflict") continue;
    const key = `project_${pm.projectId}`;
    if (!seenPaths.has(key)) {
      seenPaths.add(key);
      results.push({
        id: key,
        label: `${pm.projectName}`,
        scope: "project",
        path: pm.projectPath
      });
    }
  }

  return results;
});

// Group unmanaged sources by contentHash — same content = same version
const groupedUnmanagedSources = computed(() => {
  if (!props.librarySkill) return [];
  const groups = new Map<string, typeof props.librarySkill.unmanagedSources>();
  for (const src of props.librarySkill.unmanagedSources) {
    const key = src.contentHash || src.path;
    const group = groups.get(key);
    if (group) {
      group.push(src);
    } else {
      groups.set(key, [src]);
    }
  }
  return [...groups.entries()].map(([hash, sources]) => ({ hash, sources }));
});
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
    <div v-else-if="sortedVersions.length === 0 && librarySkill && !librarySkill.inRepo && groupedUnmanagedSources.length > 0" class="versions-list">
      <article v-for="group in groupedUnmanagedSources" :key="group.hash" class="card version-card detected">
        <button class="version-main" @click="emit('registerVersion', group.sources[0].path)">
          <div class="version-header">
            <div class="card-title"><span class="repo-dot not-in-repo">○</span> {{ t("library.status.unmanaged") }}</div>
            <span class="badge muted">{{ group.sources.length }} {{ t("library.versions.locations") }}</span>
          </div>
          <div class="deployment-list">
            <div v-for="src in group.sources" :key="src.path" class="deployment-entry">
              <span class="deploy-name">{{ src.label }}</span>
              <span class="deploy-sync sync-muted">{{ src.ide }}</span>
            </div>
          </div>
        </button>
        <div class="version-actions">
          <button class="ghost action-btn" @click="emit('registerVersion', group.sources[0].path)">{{ t("library.adoptToRepo") }}</button>
        </div>
      </article>
    </div>
    <div v-else-if="sortedVersions.length === 0" class="empty-state hint">{{ t("library.versions.empty") }}</div>

    <div v-else class="versions-list">
      <article v-for="version in sortedVersions" :key="version.id" class="card version-card" :class="{ active: isSelected(version.id), default: isDefault(version.id) }">
        <button class="version-main" @click="emit('selectVersion', version)">
          <div class="version-header">
            <div class="card-title" v-if="editingVersionId !== version.id">
              <span class="repo-dot in-repo">●</span> {{ version.displayName }}
            </div>
            <div v-else class="rename-inline" @click.stop>
              <input
                v-model="editingName"
                class="rename-input"
                @keydown.enter="confirmRename(version.id)"
                @keydown.escape="cancelRename"
                @blur="confirmRename(version.id)"
              />
            </div>
            <div class="version-badges">
              <span v-if="isDefault(version.id)" class="badge success">{{ t("library.versions.default") }}</span>
              <span v-if="skill.currentVersion?.id === version.id" class="badge muted">{{ t("library.versions.active") }}</span>
            </div>
          </div>
          <div class="card-meta">{{ version.version }} · {{ new Date(version.createdAt * 1000).toLocaleDateString() }}</div>
          <div class="version-source">{{ version.source }}</div>
          <div v-if="getVersionUsage(version.id).ideCount > 0 || getVersionUsage(version.id).projectCount > 0" class="version-usage">
            <span v-if="getVersionUsage(version.id).ideCount > 0" class="usage-tag">{{ t("library.globalIdes", { count: getVersionUsage(version.id).ideCount }) }}</span>
            <span v-if="getVersionUsage(version.id).projectCount > 0" class="usage-tag">{{ t("library.projectUsage", { count: getVersionUsage(version.id).projectCount }) }}</span>
          </div>
          <div v-if="getVersionDeployments(version.id).installations.length > 0 || getVersionDeployments(version.id).projects.length > 0" class="deployment-list">
            <div v-for="inst in getVersionDeployments(version.id).installations" :key="inst.skillPath" class="deployment-entry">
              <span class="deploy-name">{{ inst.ideLabel }}<span v-if="inst.scope === 'project'" class="deploy-scope"> ({{ t("ide.scopeProject") }})</span></span>
              <span class="deploy-sync" :class="getSyncClass(inst.syncStatus)">{{ getSyncIcon(inst.syncStatus) }}</span>
            </div>
            <div v-for="proj in getVersionDeployments(version.id).projects" :key="proj.projectId" class="deployment-entry">
              <span class="deploy-name">{{ proj.projectName }}</span>
              <span class="deploy-sync" :class="getSyncClass(proj.status === 'synced' ? 'synced' : 'modified')">{{ proj.status === "synced" ? "\u2713" : "\u26a0" }}</span>
            </div>
          </div>
        </button>
        <div class="version-actions">
          <button class="ghost action-btn" @click.stop="startRename(version)">{{ t("library.versions.rename") }}</button>
          <button v-if="!isDefault(version.id)" class="ghost action-btn" @click="emit('setDefault', version.id)">{{ t("library.versions.setDefault") }}</button>
          <button v-if="skill.currentVersion?.id !== version.id" class="ghost action-btn" @click="emit('compareVersions', version.id)">{{ t("library.versions.compare") }}</button>
          <button v-if="sortedVersions.length > 1" class="ghost danger action-btn" @click.stop="emit('deleteVersion', version.id)">{{ t("library.versions.delete") }}</button>
        </div>
      </article>

      <!-- Unregistered versions (modified/conflicted copies) -->
      <article v-for="dv in detectedVersions" :key="dv.id" class="card version-card detected">
        <button class="version-main" @click="emit('registerVersion', dv.path)">
          <div class="version-header">
            <div class="card-title"><span class="repo-dot not-in-repo">○</span> {{ dv.label }}</div>
            <span class="badge muted">{{ t("library.status.unmanaged") }}</span>
          </div>
          <div class="card-meta">{{ dv.path }}</div>
        </button>
        <div class="version-actions">
          <button class="ghost action-btn" @click="emit('registerVersion', dv.path)">{{ t("library.versions.register") }}</button>
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

.version-usage {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
  margin-top: 6px;
}

.usage-tag {
  padding: 2px 6px;
  border-radius: 6px;
  font-size: 11px;
  background: var(--color-chip-bg);
  border: 1px solid var(--color-chip-border);
  color: var(--color-muted);
}

.deployment-list {
  margin-top: 6px;
  padding-left: 4px;
  border-left: 2px solid var(--color-card-border);
}

.deployment-entry {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 2px 8px;
  font-size: 11px;
}

.deploy-name {
  color: var(--color-muted);
}

.deploy-sync {
  font-weight: 600;
  font-size: 11px;
}

.deploy-scope { font-size: 10px; opacity: 0.7; }

.deploy-sync.sync-ok { color: var(--color-success-text); }
.deploy-sync.sync-warn { color: #d97706; }
.deploy-sync.sync-muted { color: var(--color-muted); }

.repo-dot {
  font-size: 10px;
  margin-right: 4px;
}

.repo-dot.in-repo { color: var(--color-success-text); }
.repo-dot.not-in-repo { color: var(--color-muted); }

.detected-header {
  font-size: 12px;
  font-weight: 700;
  color: var(--color-muted);
  padding: 12px 0 4px;
}

.version-card.detected {
  opacity: 0.8;
  border-style: dashed;
}

.source-entry {
  padding: 4px 0;
}

.source-entry + .source-entry {
  border-top: 1px solid var(--color-card-border);
  margin-top: 4px;
  padding-top: 8px;
}

.rename-inline {
  flex: 1;
}

.rename-input {
  width: 100%;
  padding: 3px 6px;
  border: 1px solid var(--color-input-focus);
  border-radius: 6px;
  background: var(--color-input-bg);
  color: var(--color-text);
  font-size: 13px;
  font-weight: 600;
  outline: none;
}
</style>
