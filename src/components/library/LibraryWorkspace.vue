<script setup lang="ts">
import { computed, ref, watch, onUnmounted } from "vue";
import LibrarySidebar from "./LibrarySidebar.vue";
import LibraryDetailPanel from "./LibraryDetailPanel.vue";
import LibraryVersionRail from "./LibraryVersionRail.vue";
import type {
  DownloadTask,
  IdeOption,
  LibrarySkill,
  LocalSkill,
  ProjectConfig,
  SkillPackage,
  SkillVersion
} from "../../composables/types";

type CloneTargetProject = {
  id: string;
  name: string;
  ideTargets: string[];
};

const props = defineProps<{
  localSkills: LocalSkill[];
  localLoading: boolean;
  installingId: string | null;
  downloadQueue: DownloadTask[];
  ideOptions: IdeOption[];
  skillPackage: SkillPackage | null;
  versionLoading: boolean;
  projects: ProjectConfig[];
  librarySkills: LibrarySkill[];
}>();

const emit = defineEmits<{
  (e: "install", skill: LocalSkill): void;
  (e: "installMany", skills: LocalSkill[]): void;
  (e: "deleteLocal", skills: LocalSkill[]): void;
  (e: "openDir", path: string): void;
  (e: "refresh"): void;
  (e: "import"): void;
  (e: "retryDownload", taskId: string): void;
  (e: "removeFromQueue", taskId: string): void;
  (e: "setDefaultVersion", skillId: string, versionId: string): void;
  (e: "renameVersion", skillId: string, versionId: string, newName: string): void;
  (e: "deleteVersion", skillId: string, versionId: string): void;
  (e: "cloneToProject", project: CloneTargetProject, skillIds: string[]): void;
  (e: "compareVersions", fromVersionId: string, toVersionId: string): void;
  (e: "createVersion"): void;
  (e: "selectSkill", skill: LocalSkill): void;
  (e: "adoptToRepo", path: string): void;
  (e: "adoptManyToRepo", targets: Array<{ path: string; ideLabel: string }>): void;
  (e: "registerVersion", sourcePath: string, displayName: string, version: string): void;
  (e: "uninstallSkill", path: string): void;
}>();

const searchQuery = ref("");
const selectedSkillId = ref<string | null>(null);
const selectedIds = ref<string[]>([]);
const selectedVersionId = ref<string | null>(null);
const platformFilter = ref<string>("all");
const statusFilter = ref<string>("all");
const activeStatusTags = ref<Set<string>>(new Set(["managed", "unmanaged"]));

const platformOptions = computed(() => {
  const options = [{ id: "all", label: "all", count: props.localSkills.length }];
  for (const ide of props.ideOptions) {
    options.push({
      id: ide.id,
      label: ide.label,
      count: props.librarySkills.filter((skill) => skill.installations.some((item) => item.ideId === ide.id)).length
    });
  }
  return options;
});

const projectUsageMap = computed(() => {
  const map = new Map<string, string[]>();
  for (const ls of props.librarySkills) {
    if (ls.usedByProjectIds.length > 0) {
      map.set(ls.id, ls.usedByProjectIds);
    }
  }
  return map;
});

const skillStatusMap = computed(() => {
  const map = new Map<string, string>();
  for (const ls of props.librarySkills) {
    map.set(ls.id, ls.status);
  }
  return map;
});

const skillScopeMap = computed(() => {
  const map = new Map<string, string>();
  for (const ls of props.librarySkills) {
    map.set(ls.id, ls.skillScope);
  }
  return map;
});

const skillHasNewVersionsMap = computed(() => {
  const map = new Map<string, boolean>();
  for (const ls of props.librarySkills) {
    if (!ls.inRepo) continue;
    const hasModifiedInstall = ls.installations.some((i) => i.syncStatus === "modified" && i.scope !== "plugin");
    const hasUnmanagedInstall = ls.installations.some((i) => !i.isManaged && i.scope !== "plugin");
    const hasConflictProject = ls.projectMappings.some((p) => p.status === "conflict");
    if (hasModifiedInstall || hasUnmanagedInstall || hasConflictProject) {
      map.set(ls.id, true);
    }
  }
  return map;
});

const skillPluginCountMap = computed(() => {
  const map = new Map<string, number>();
  for (const ls of props.librarySkills) {
    if (!ls.inRepo) continue;
    const count = ls.installations.filter((i) => i.scope === "plugin").length;
    if (count > 0) map.set(ls.id, count);
  }
  return map;
});

// Skills that are only deployed via plugins (no global IDE or project usage)
const skillPluginOnlyMap = computed(() => {
  const map = new Map<string, boolean>();
  for (const ls of props.librarySkills) {
    if (!ls.inRepo) continue;
    const pluginCount = ls.installations.filter((i) => i.scope === "plugin").length;
    if (pluginCount === 0) continue;
    const globalCount = ls.installations.filter((i) => i.scope === "global" && i.isManaged).length;
    const projectCount = ls.usedByProjectIds.length;
    if (globalCount === 0 && projectCount === 0) {
      map.set(ls.id, true);
    }
  }
  return map;
});

const skillDefaultVersionMap = computed(() => {
  const map = new Map<string, { displayName: string; versionCount: number }>();
  for (const ls of props.librarySkills) {
    if (!ls.inRepo) continue;
    if (ls.defaultVersion) {
      map.set(ls.id, {
        displayName: ls.defaultVersion.displayName,
        versionCount: ls.versionCount
      });
    }
  }
  return map;
});

const statusOptions = computed(() => {
  const managed = props.librarySkills.filter((ls) => ls.inRepo).length;
  const unmanaged = props.librarySkills.filter((ls) => !ls.inRepo).length;
  const pluginOnly = skillPluginOnlyMap.value.size;

  return [
    { id: "all", label: "all", count: props.librarySkills.length },
    { id: "managed", label: "managed", count: managed },
    { id: "unmanaged", label: "unmanaged", count: unmanaged },
    { id: "pluginOnly", label: "pluginOnly", count: pluginOnly }
  ];
});

const statusTagCounts = computed(() => {
  const pluginOnly = skillPluginOnlyMap.value.size;
  const managed = props.librarySkills.filter((ls) => ls.inRepo).length - pluginOnly;
  const unmanaged = props.librarySkills.filter((ls) => !ls.inRepo).length;
  return { managed, unmanaged, pluginOnly };
});

function handleToggleStatusTag(tag: string) {
  const next = new Set(activeStatusTags.value);
  if (next.has(tag)) next.delete(tag);
  else next.add(tag);
  activeStatusTags.value = next;
}

const allSidebarSkills = computed<LocalSkill[]>(() => {
  // Start with repo skills
  const result: LocalSkill[] = [...props.localSkills];

  // Add unmanaged skills as synthetic LocalSkill entries
  for (const ls of props.librarySkills) {
    if (!ls.inRepo) {
      result.push({
        id: ls.id,
        name: ls.name,
        description: ls.description,
        path: ls.path,
        source: ls.source,
        usedBy: [],
        versionCount: 0
      });
    }
  }

  return result;
});

const filteredSidebarSkills = computed<LocalSkill[]>(() => {
  const keyword = searchQuery.value.trim().toLowerCase();

  return allSidebarSkills.value.filter((skill) => {
    const librarySkill = props.librarySkills.find((item) => item.id === skill.id);
    if (!librarySkill) {
      return false;
    }

    if (platformFilter.value !== "all") {
      if (librarySkill.inRepo) {
        if (!librarySkill.installations.some((item) => item.ideId === platformFilter.value)) {
          return false;
        }
      } else {
        if (!librarySkill.unmanagedSources.some((s) => s.ide === platformFilter.value)) {
          return false;
        }
      }
    }

    // Multi-select status tag filtering
    if (activeStatusTags.value.size > 0) {
      const isManaged = librarySkill.inRepo;
      const isPluginOnly = skillPluginOnlyMap.value.has(skill.id);
      let matched = false;
      if (activeStatusTags.value.has("managed") && isManaged && !isPluginOnly) matched = true;
      if (activeStatusTags.value.has("unmanaged") && !isManaged) matched = true;
      if (activeStatusTags.value.has("pluginOnly") && isPluginOnly) matched = true;
      if (!matched) return false;
    }

    if (!keyword) {
      return true;
    }

    return [skill.name, skill.description, skill.path].some((value) => value.toLowerCase().includes(keyword));
  });
});

const selectedSkill = computed<LocalSkill | null>(() => {
  if (!selectedSkillId.value) {
    return null;
  }
  return allSidebarSkills.value.find((skill) => skill.id === selectedSkillId.value) || null;
});

const selectedLibrarySkill = computed<LibrarySkill | null>(() => {
  if (!selectedSkillId.value) {
    return null;
  }
  return props.librarySkills.find((skill) => skill.id === selectedSkillId.value) || null;
});

const selectedVersion = computed<SkillVersion | null>(() => {
  if (!selectedVersionId.value || !props.skillPackage) {
    return null;
  }
  return props.skillPackage.versions.find((version) => version.id === selectedVersionId.value) || null;
});

const selectedSkills = computed<LocalSkill[]>(() =>
  allSidebarSkills.value.filter((skill) => selectedIds.value.includes(skill.id))
);

watch(
  allSidebarSkills,
  (skills) => {
    const availableIds = new Set(skills.map((skill) => skill.id));
    selectedIds.value = selectedIds.value.filter((id) => availableIds.has(id));

    if (selectedSkillId.value && !availableIds.has(selectedSkillId.value)) {
      // If old selection was unmanaged, try to find the managed version with same name
      if (selectedSkillId.value.startsWith("unmanaged_")) {
        const oldSkill = props.librarySkills.find((s) => s.id === selectedSkillId.value);
        if (oldSkill) {
          const managed = skills.find((s) => s.name === oldSkill.name && !s.id.startsWith("unmanaged_"));
          if (managed) {
            selectedSkillId.value = managed.id;
            return;
          }
        }
      }
      selectedSkillId.value = null;
    }

    if (skills.length > 0 && !selectedSkillId.value) {
      selectedSkillId.value = skills[0].id;
    }
  },
  { immediate: true }
);

watch(selectedSkillId, (skillId) => {
  if (!skillId) {
    selectedVersionId.value = null;
    return;
  }

  // Ensure package is loaded whenever selection changes (including auto-select)
  const skill = allSidebarSkills.value.find((s) => s.id === skillId);
  if (skill) {
    emit("selectSkill", skill);
  }

  const currentVersion = props.skillPackage?.versions.find((version) => version.isActive)
    || props.skillPackage?.versions.find((version) => version.id === props.skillPackage?.defaultVersion)
    || null;
  selectedVersionId.value = currentVersion?.id || null;
});

watch(
  () => props.skillPackage,
  (skillPackage) => {
    if (!skillPackage) {
      selectedVersionId.value = null;
      return;
    }

    if (selectedVersionId.value && skillPackage.versions.some((version) => version.id === selectedVersionId.value)) {
      return;
    }

    const nextVersion = skillPackage.versions.find((version) => version.isActive)
      || skillPackage.versions.find((version) => version.id === skillPackage.defaultVersion)
      || skillPackage.versions[0]
      || null;
    selectedVersionId.value = nextVersion?.id || null;
  },
  { immediate: true }
);

function handleSelectSkill(skill: LocalSkill): void {
  selectedSkillId.value = skill.id;
  // selectSkill emit is handled by the selectedSkillId watcher
}

function handleToggleSelected(skillId: string, checked: boolean): void {
  selectedIds.value = checked
    ? Array.from(new Set([...selectedIds.value, skillId]))
    : selectedIds.value.filter((id) => id !== skillId);
}

function handleToggleSelectAll(checked: boolean, filteredIds: string[]): void {
  if (checked) {
    selectedIds.value = Array.from(new Set([...selectedIds.value, ...filteredIds]));
    return;
  }

  selectedIds.value = selectedIds.value.filter((id) => !filteredIds.includes(id));
}

watch(filteredSidebarSkills, (skills) => {
  const visibleIds = new Set(skills.map((skill) => skill.id));
  selectedIds.value = selectedIds.value.filter((id) => visibleIds.has(id) || props.localSkills.some((skill) => skill.id === id));

  if (selectedSkillId.value && !visibleIds.has(selectedSkillId.value)) {
    selectedSkillId.value = skills[0]?.id || null;
  }
});

function handleDelete(skill: LocalSkill): void {
  emit("deleteLocal", [skill]);
  if (selectedSkillId.value === skill.id) {
    selectedSkillId.value = null;
  }
}

function handleDeleteSelected(): void {
  if (selectedSkills.value.length === 0) {
    return;
  }

  emit("deleteLocal", selectedSkills.value);
}

function handleClearSelection(): void {
  selectedIds.value = [];
}

function handleAdoptSelected(): void {
  // Get paths + IDE labels of selected unmanaged skills
  const unmanagedSkills = selectedIds.value
    .map((id) => props.librarySkills.find((ls) => ls.id === id))
    .filter((ls): ls is NonNullable<typeof ls> => !!ls && !ls.inRepo);

  const targets: Array<{ path: string; ideLabel: string }> = [];
  for (const ls of unmanagedSkills) {
    if (ls.unmanagedSources.length > 0) {
      for (const src of ls.unmanagedSources) {
        targets.push({ path: src.path, ideLabel: src.ide });
      }
    } else if (ls.path) {
      targets.push({ path: ls.path, ideLabel: "" });
    }
  }

  if (targets.length > 0) {
    emit("adoptManyToRepo", targets);
  }
}

function handleInstallSelected(): void {
  if (selectedSkills.value.length === 0) {
    return;
  }

  emit("installMany", selectedSkills.value);
}

function handleCloneSelected(): void {
  // Clone selected repo skills to the first project that has IDE targets configured
  const repoSkillIds = selectedIds.value
    .filter((id) => {
      const ls = props.librarySkills.find((s) => s.id === id);
      return ls && ls.inRepo;
    });

  if (repoSkillIds.length === 0) return;

  const project = props.projects.find((p) => p.ideTargets.length > 0);
  if (!project) return;

  emit("cloneToProject", {
    id: project.id,
    name: project.name,
    ideTargets: project.ideTargets
  }, repoSkillIds);
}

function handleDeleteAll(): void {
  if (props.localSkills.length === 0) {
    return;
  }

  emit("deleteLocal", props.localSkills);
}

function handleSetDefaultVersion(versionId: string): void {
  if (!props.skillPackage) return;
  emit("setDefaultVersion", props.skillPackage.id, versionId);
}

function handleRenameVersion(versionId: string, newName: string): void {
  if (!props.skillPackage) return;
  emit("renameVersion", props.skillPackage.id, versionId, newName);
}

function handleDeleteVersion(versionId: string): void {
  if (!props.skillPackage) return;
  emit("deleteVersion", props.skillPackage.id, versionId);
}

function handleCloneToProject(projectId: string): void {
  const project = props.projects.find((item) => item.id === projectId);
  if (!project || !selectedSkill.value) {
    return;
  }

  emit("cloneToProject", {
    id: project.id,
    name: project.name,
    ideTargets: project.ideTargets
  }, [selectedSkill.value.id]);
}

function handleCompareSelectedVersion(versionId: string): void {
  if (!selectedSkill.value?.currentVersion || selectedSkill.value.currentVersion.id === versionId) {
    return;
  }

  emit("compareVersions", selectedSkill.value.currentVersion.id, versionId);
}

function handleSelectVersion(version: SkillVersion): void {
  selectedVersionId.value = version.id;
}

// --- Draggable splitter logic ---
const LAYOUT_STORAGE_KEY = "qingSkillManager.libraryLayout";
const MIN_SIDEBAR = 220;
const MIN_DETAIL = 280;
const MIN_RAIL = 200;
const DEFAULT_SIDEBAR = 340;
const DEFAULT_RAIL = 260;

function loadLayout(): { sidebar: number; rail: number } {
  try {
    const raw = localStorage.getItem(LAYOUT_STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      if (typeof parsed.sidebar === "number" && typeof parsed.rail === "number") {
        return parsed;
      }
    }
  } catch { /* ignore */ }
  return { sidebar: DEFAULT_SIDEBAR, rail: DEFAULT_RAIL };
}

function saveLayout(sidebar: number, rail: number): void {
  localStorage.setItem(LAYOUT_STORAGE_KEY, JSON.stringify({ sidebar, rail }));
}

const saved = loadLayout();
const sidebarWidth = ref(saved.sidebar);
const railWidth = ref(saved.rail);
const workspaceRef = ref<HTMLElement | null>(null);

let dragging: "left" | "right" | null = null;
let startX = 0;
let startValue = 0;

function onSplitterDown(side: "left" | "right", e: MouseEvent): void {
  e.preventDefault();
  dragging = side;
  startX = e.clientX;
  startValue = side === "left" ? sidebarWidth.value : railWidth.value;
  document.addEventListener("mousemove", onMouseMove);
  document.addEventListener("mouseup", onMouseUp);
  document.body.style.cursor = "col-resize";
  document.body.style.userSelect = "none";
}

function onMouseMove(e: MouseEvent): void {
  if (!dragging || !workspaceRef.value) return;
  const totalWidth = workspaceRef.value.offsetWidth;
  const splitterTotal = 8; // 2 splitters * 4px

  if (dragging === "left") {
    const newSidebar = Math.max(MIN_SIDEBAR, Math.min(startValue + (e.clientX - startX), totalWidth - railWidth.value - MIN_DETAIL - splitterTotal));
    sidebarWidth.value = newSidebar;
  } else {
    const newRail = Math.max(MIN_RAIL, Math.min(startValue - (e.clientX - startX), totalWidth - sidebarWidth.value - MIN_DETAIL - splitterTotal));
    railWidth.value = newRail;
  }
}

function onMouseUp(): void {
  if (dragging) {
    saveLayout(sidebarWidth.value, railWidth.value);
  }
  dragging = null;
  document.removeEventListener("mousemove", onMouseMove);
  document.removeEventListener("mouseup", onMouseUp);
  document.body.style.cursor = "";
  document.body.style.userSelect = "";
}

onUnmounted(() => {
  document.removeEventListener("mousemove", onMouseMove);
  document.removeEventListener("mouseup", onMouseUp);
});
</script>

<template>
  <div ref="workspaceRef" class="library-workspace">
    <LibrarySidebar
      v-model:search-query="searchQuery"
      :style="{ width: sidebarWidth + 'px', minWidth: MIN_SIDEBAR + 'px', flexShrink: 0 }"
      :skills="filteredSidebarSkills"
      :selected-skill-id="selectedSkillId"
      :selected-ids="selectedIds"
      :loading="localLoading"
      :ide-options="ideOptions"
      :platform-filter="platformFilter"
      :status-filter="statusFilter"
      :active-status-tags="activeStatusTags"
      :status-tag-counts="statusTagCounts"
      :platform-options="platformOptions"
      :status-options="statusOptions"
      :project-usage-map="projectUsageMap"
      :skill-status-map="skillStatusMap"
      :skill-scope-map="skillScopeMap"
      :skill-has-new-versions-map="skillHasNewVersionsMap"
      :skill-default-version-map="skillDefaultVersionMap"
      :skill-plugin-count-map="skillPluginCountMap"
      :skill-plugin-only-map="skillPluginOnlyMap"
      @select="handleSelectSkill"
      @toggle-selected="handleToggleSelected"
      @toggle-select-all="handleToggleSelectAll"
      @install-selected="handleInstallSelected"
      @clone-selected="handleCloneSelected"
      @adopt-selected="handleAdoptSelected"
      @delete-selected="handleDeleteSelected"
      @clear-selection="handleClearSelection"
      @delete-all="handleDeleteAll"
      @update:platform-filter="platformFilter = $event"
      @update:status-filter="statusFilter = $event"
      @toggle-status-tag="handleToggleStatusTag"
      @refresh="$emit('refresh')"
      @import="$emit('import')"
    />

    <div class="splitter" @mousedown="onSplitterDown('left', $event)" />

    <LibraryDetailPanel
      :skill="selectedSkill"
      :library-skill="selectedLibrarySkill"
      :selected-version-id="selectedVersionId"
      :installing-id="installingId"
      :ide-options="ideOptions"
      :projects="projects"
      @install="$emit('install', $event)"
      @clone-to-project="handleCloneToProject"
      @open-dir="$emit('openDir', $event)"
      @delete="handleDelete"
      @adopt-to-repo="$emit('adoptToRepo', $event)"
      @uninstall-skill="$emit('uninstallSkill', $event)"
    />

    <div class="splitter" @mousedown="onSplitterDown('right', $event)" />

    <LibraryVersionRail
      :style="{ width: railWidth + 'px', minWidth: MIN_RAIL + 'px', flexShrink: 0 }"
      :skill="selectedSkill"
      :library-skill="selectedLibrarySkill"
      :selected-version-id="selectedVersion?.id || null"
      :skill-package="skillPackage"
      :loading="versionLoading"
      @select-version="handleSelectVersion"
      @compare-versions="handleCompareSelectedVersion"
      @create-version="$emit('createVersion')"
      @set-default="handleSetDefaultVersion"
      @rename-version="handleRenameVersion"
      @delete-version="handleDeleteVersion"
      @register-version="(path, name, ver) => $emit('registerVersion', path, name, ver)"
      @adopt-to-repo="$emit('adoptToRepo', $event)"
    />
  </div>
</template>

<style scoped>
.library-workspace {
  display: flex;
  height: 100%;
  width: 100%;
  overflow: hidden;
  background: var(--color-bg);
}

.splitter {
  flex-shrink: 0;
  width: 4px;
  cursor: col-resize;
  background: var(--color-card-border, #e0e0e0);
  transition: background 0.15s;
}

.splitter:hover {
  background: var(--color-primary, #4a90d9);
}
</style>
