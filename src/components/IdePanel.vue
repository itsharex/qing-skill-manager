<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { IdeSkill, IdeOption } from "../composables/types";
import { useI18n } from "vue-i18n";

const { t } = useI18n();

const props = defineProps<{
  ideOptions: IdeOption[];
  selectedIdeFilter: string;
  customIdeName: string;
  customIdeDir: string;
  customIdeOptions: IdeOption[];
  filteredIdeSkills: IdeSkill[];

  localLoading: boolean;
}>();

const emit = defineEmits<{
  (e: "update:selectedIdeFilter", value: string): void;
  (e: "update:customIdeName", value: string): void;
  (e: "update:customIdeDir", value: string): void;
  (e: "addCustomIde"): void;
  (e: "removeCustomIde", label: string): void;
  (e: "uninstall", path: string): void;
  (e: "uninstallMany", paths: string[]): void;
  (e: "openDir", path: string): void;
  (e: "adopt", skill: IdeSkill): void;
  (e: "adoptMany", skills: IdeSkill[]): void;
}>();

const selectedIds = ref<string[]>([]);
const showPlugin = ref(false);

watch(
  () => props.filteredIdeSkills,
  (skills) => {
    const available = new Set(skills.map((skill) => skill.id));
    selectedIds.value = selectedIds.value.filter((id) => available.has(id));
  },
  { deep: true }
);

const visibleSkills = computed(() =>
  showPlugin.value
    ? props.filteredIdeSkills
    : props.filteredIdeSkills.filter((skill) => skill.scope !== "plugin")
);

const globalSkills = computed(() => visibleSkills.value.filter((skill) => skill.scope === "global"));
const projectSkills = computed(() => visibleSkills.value.filter((skill) => skill.scope === "project"));
const pluginSkills = computed(() => visibleSkills.value.filter((skill) => skill.scope === "plugin"));
const pluginCount = computed(() => props.filteredIdeSkills.filter((skill) => skill.scope === "plugin").length);

const selectedSkills = computed(() =>
  visibleSkills.value.filter((skill) => selectedIds.value.includes(skill.id))
);

const selectedUnmanagedSkills = computed(() =>
  selectedSkills.value.filter((skill) => !skill.managed)
);

const allSelected = computed(
  () =>
    visibleSkills.value.length > 0 &&
    visibleSkills.value.filter((skill) => skill.scope !== "plugin").every((skill) => selectedIds.value.includes(skill.id))
);

function toggleSelectAll(checked: boolean) {
  const filteredIds = props.filteredIdeSkills.map((skill) => skill.id);
  if (checked) {
    selectedIds.value = Array.from(new Set([...selectedIds.value, ...filteredIds]));
    return;
  }
  selectedIds.value = selectedIds.value.filter((id) => !filteredIds.includes(id));
}

function toggleSelected(skillId: string, checked: boolean) {
  selectedIds.value = checked
    ? [...selectedIds.value, skillId]
    : selectedIds.value.filter((id) => id !== skillId);
}

function uninstallSelected() {
  if (selectedSkills.value.length === 0) return;
  emit("uninstallMany", selectedSkills.value.map((skill) => skill.path));
}

function adoptSelected() {
  if (selectedUnmanagedSkills.value.length === 0) return;
  emit("adoptMany", selectedUnmanagedSkills.value);
}
</script>

<template>
  <section class="panel">
    <div class="panel-title">{{ t("ide.title") }}</div>
    <div class="hint">{{ t("ide.switchHint") }}</div>
    <div class="ide-filter-grid">
      <button
        v-for="option in ideOptions"
        :key="option.id"
        class="ghost ide-filter-btn"
        :class="{ active: selectedIdeFilter === option.label }"
        @click="$emit('update:selectedIdeFilter', option.label)"
      >
        {{ option.label }}
      </button>
    </div>
    <div class="hint">{{ t("ide.addHint") }}</div>
    <div class="row">
      <input
        :value="customIdeName"
        class="input small"
        :placeholder="t('ide.namePlaceholder')"
        @input="$emit('update:customIdeName', ($event.target as HTMLInputElement).value)"
      />
      <input
        :value="customIdeDir"
        class="input small"
        :placeholder="t('ide.dirPlaceholder')"
        @input="$emit('update:customIdeDir', ($event.target as HTMLInputElement).value)"
      />
      <button class="primary" @click="$emit('addCustomIde')">{{ t("ide.addButton") }}</button>
    </div>
    <div v-if="customIdeOptions.length > 0" class="chips">
      <div v-for="option in customIdeOptions" :key="option.id" class="chip">
        <span>{{ option.label }}</span>
        <button class="ghost" @click="$emit('removeCustomIde', option.label)">{{ t("ide.deleteButton") }}</button>
      </div>
    </div>

    <div class="scope-toggle">
      <label class="checkbox">
        <input type="checkbox" v-model="showPlugin" />
        {{ t("ide.showPlugin", { count: pluginCount }) }}
      </label>
    </div>

    <div class="actions">
      <div class="buttons">
        <button
          class="primary"
          :disabled="selectedUnmanagedSkills.length === 0 || localLoading"
          @click="adoptSelected"
        >
          {{ t("ide.adoptSelected", { count: selectedUnmanagedSkills.length }) }}
        </button>
        <button
          class="ghost danger btn-sm"
          :disabled="selectedSkills.length === 0 || localLoading"
          @click="uninstallSelected"
        >
          {{ t("ide.uninstallSelected", { count: selectedSkills.length }) }}
        </button>
      </div>
    </div>

    <div v-if="localLoading" class="hint">{{ t("ide.loading") }}</div>
    <div v-if="!localLoading && visibleSkills.length === 0" class="hint">{{ t("ide.emptyHint") }}</div>

    <div v-if="visibleSkills.length > 0" class="skill-list-header">
      <span class="skill-list-count">{{ t("ide.total", { count: visibleSkills.length }) }}</span>
      <label class="checkbox select-all">
        <input
          type="checkbox"
          :checked="allSelected"
          :disabled="globalSkills.length + projectSkills.length === 0"
          @change="toggleSelectAll(($event.target as HTMLInputElement).checked)"
        />
        {{ t("ide.selectAll") }}
      </label>
    </div>

    <div v-if="visibleSkills.length > 0" class="skill-groups">
      <!-- Global Skills -->
      <div v-if="globalSkills.length > 0" class="skill-group">
        <div class="group-header">{{ t("ide.scopeGlobal") }} <span class="group-count">{{ globalSkills.length }}</span></div>
        <div class="cards">
          <article v-for="skill in globalSkills" :key="skill.id" class="card" :class="{ unmanaged: !skill.managed }">
            <div class="card-header">
              <div class="card-title-row">
                <label class="checkbox card-select">
                  <input type="checkbox" :checked="selectedIds.includes(skill.id)" @change="toggleSelected(skill.id, ($event.target as HTMLInputElement).checked)" />
                </label>
                <div>
                  <div class="card-title">{{ skill.name }}</div>
                  <div class="card-meta">{{ skill.source === "managed" ? t("ide.sourceManaged") : t("ide.sourceLocal") }}{{ !skill.managed ? ` · ${t("ide.unmanaged")}` : "" }}</div>
                </div>
              </div>
              <div class="card-actions">
                <button class="ghost" @click="$emit('openDir', skill.path)">{{ t("ide.openDir") }}</button>
                <button v-if="!skill.managed" class="ghost" @click="$emit('adopt', skill)">{{ t("ide.adopt") }}</button>
                <button class="ghost" @click="$emit('uninstall', skill.path)">{{ t("ide.uninstall") }}</button>
              </div>
            </div>
            <div class="card-link">{{ skill.path }}</div>
          </article>
        </div>
      </div>

      <!-- Project Skills -->
      <div v-if="projectSkills.length > 0" class="skill-group">
        <div class="group-header">{{ t("ide.scopeProject") }} <span class="group-count">{{ projectSkills.length }}</span></div>
        <div class="cards">
          <article v-for="skill in projectSkills" :key="skill.id" class="card" :class="{ unmanaged: !skill.managed }">
            <div class="card-header">
              <div class="card-title-row">
                <label class="checkbox card-select">
                  <input type="checkbox" :checked="selectedIds.includes(skill.id)" @change="toggleSelected(skill.id, ($event.target as HTMLInputElement).checked)" />
                </label>
                <div>
                  <div class="card-title">{{ skill.name }}</div>
                  <div class="card-meta">{{ skill.source === "managed" ? t("ide.sourceManaged") : t("ide.sourceLocal") }}{{ !skill.managed ? ` · ${t("ide.unmanaged")}` : "" }}</div>
                </div>
              </div>
              <div class="card-actions">
                <button class="ghost" @click="$emit('openDir', skill.path)">{{ t("ide.openDir") }}</button>
                <button v-if="!skill.managed" class="ghost" @click="$emit('adopt', skill)">{{ t("ide.adopt") }}</button>
                <button class="ghost" @click="$emit('uninstall', skill.path)">{{ t("ide.uninstall") }}</button>
              </div>
            </div>
            <div class="card-link">{{ skill.path }}</div>
          </article>
        </div>
      </div>

      <!-- Plugin Skills -->
      <div v-if="pluginSkills.length > 0" class="skill-group">
        <div class="group-header">{{ t("ide.scopePlugin") }} <span class="group-count">{{ pluginSkills.length }}</span></div>
        <div class="cards">
          <article v-for="skill in pluginSkills" :key="skill.id" class="card plugin">
            <div class="card-header">
              <div class="card-title-row">
                <div>
                  <div class="card-title">{{ skill.name }}</div>
                  <div class="card-meta"><span class="plugin-badge">{{ t("ide.sourcePlugin") }}</span></div>
                </div>
              </div>
              <div class="card-actions">
                <button class="ghost" @click="$emit('openDir', skill.path)">{{ t("ide.openDir") }}</button>
              </div>
            </div>
            <div class="card-link">{{ skill.path }}</div>
          </article>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.panel-summary {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: center;
  font-size: 13px;
  color: var(--color-muted);
}

.ide-filter-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  margin-bottom: 12px;
}

.ide-filter-btn.active {
  background: var(--color-primary-bg);
  border-color: var(--color-primary-bg);
  color: var(--color-primary-text);
}

.card.unmanaged {
  border-color: rgba(245, 158, 11, 0.35);
  box-shadow: inset 0 0 0 1px rgba(245, 158, 11, 0.22);
}

.card.plugin {
  opacity: 0.7;
  border-color: var(--color-card-border);
}

.plugin-badge {
  padding: 1px 6px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
  background: var(--color-chip-bg);
  border: 1px solid var(--color-chip-border);
}

.card-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 8px;
}

.card-title-row {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  gap: 10px;
}

.card-select {
  padding-top: 2px;
}

.status-badge {
  padding: 3px 8px;
  border-radius: 999px;
  font-size: 11px;
  line-height: 1.2;
  font-weight: 600;
}

.status-badge.unmanaged {
  color: #8a4b00;
  background: rgba(245, 158, 11, 0.16);
  border: 1px solid rgba(245, 158, 11, 0.28);
}

.select-all {
  justify-content: flex-end;
}

.buttons {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin-top: 12px;
}

.skill-list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 12px;
  padding: 8px 0;
  font-size: 13px;
  color: var(--color-muted);
}

.skill-list-count {
  font-weight: 600;
}

.scope-toggle {
  margin-top: 12px;
  font-size: 13px;
  color: var(--color-muted);
}

.skill-groups {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.skill-group {
  margin-top: 8px;
}

.group-header {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-muted);
  margin-bottom: 8px;
  display: flex;
  align-items: center;
  gap: 6px;
}

.group-count {
  padding: 1px 6px;
  border-radius: 999px;
  font-size: 11px;
  background: var(--color-chip-bg);
  border: 1px solid var(--color-chip-border);
}
</style>
