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
  (e: "adoptManyToRepo", paths: string[]): void;
  (e: "registerVersion", sourcePath: string): void;
  (e: "uninstallSkill", path: string): void;
}>();

const searchQuery = ref("");
const selectedSkillId = ref<string | null>(null);
const selectedIds = ref<string[]>([]);
const selectedVersionId = ref<string | null>(null);
const platformFilter = ref<string>("all");
const statusFilter = ref<string>("all");

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

function isSkillUsed(skill: LocalSkill): boolean {
  return skill.usedBy.length > 0 || (projectUsageMap.value.get(skill.id)?.length ?? 0) > 0;
}

const statusOptions = computed(() => {
  const counts = {
    all: props.librarySkills.length,
    used: props.localSkills.filter((skill) => isSkillUsed(skill)).length,
    unused: props.localSkills.filter((skill) => !isSkillUsed(skill)).length
  };

  return [
    { id: "all", label: "all", count: counts.all },
    { id: "used", label: "used", count: counts.used },
    { id: "unused", label: "unused", count: counts.unused }
  ];
});

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

    if (statusFilter.value !== "all") {
      if (statusFilter.value === "used" && !librarySkill.inRepo) {
        return false;
      }
      if (statusFilter.value === "used" && librarySkill.inRepo && !isSkillUsed(skill)) {
        return false;
      }
      if (statusFilter.value === "unused" && isSkillUsed(skill)) {
        return false;
      }
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
  emit("selectSkill", skill);
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
  // Get paths of selected unmanaged skills
  const paths = selectedIds.value
    .map((id) => props.librarySkills.find((ls) => ls.id === id))
    .filter((ls) => ls && !ls.inRepo && ls.unmanagedSources.length > 0)
    .map((ls) => ls!.unmanagedSources[0].path);
  if (paths.length > 0) {
    emit("adoptManyToRepo", paths);
  }
}

function handleInstallSelected(): void {
  if (selectedSkills.value.length === 0) {
    return;
  }

  emit("installMany", selectedSkills.value);
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
      :platform-options="platformOptions"
      :status-options="statusOptions"
      :project-usage-map="projectUsageMap"
      :skill-status-map="skillStatusMap"
      :skill-scope-map="skillScopeMap"
      @select="handleSelectSkill"
      @toggle-selected="handleToggleSelected"
      @toggle-select-all="handleToggleSelectAll"
      @install-selected="handleInstallSelected"
      @adopt-selected="handleAdoptSelected"
      @delete-selected="handleDeleteSelected"
      @clear-selection="handleClearSelection"
      @delete-all="handleDeleteAll"
      @update:platform-filter="platformFilter = $event"
      @update:status-filter="statusFilter = $event"
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
      @register-version="$emit('registerVersion', $event)"
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
