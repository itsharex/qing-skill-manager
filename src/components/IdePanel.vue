<script setup lang="ts">
import type { IdeSkill, IdeOption } from "../composables/types";
import { useI18n } from "vue-i18n";

const { t } = useI18n();

defineProps<{
  ideOptions: IdeOption[];
  selectedIdeFilter: string;
  customIdeName: string;
  customIdeDir: string;
  customIdeOptions: IdeOption[];
  filteredIdeSkills: IdeSkill[];

  localLoading: boolean;
}>();

defineEmits<{
  (e: "update:selectedIdeFilter", value: string): void;
  (e: "update:customIdeName", value: string): void;
  (e: "update:customIdeDir", value: string): void;
  (e: "addCustomIde"): void;
  (e: "removeCustomIde", label: string): void;
  (e: "uninstall", path: string): void;
  (e: "openDir", path: string): void;
  (e: "adopt", skill: IdeSkill): void;
}>();
</script>

<template>
  <section class="panel">
    <div class="panel-title">{{ t("ide.title") }}</div>
    <div class="panel-summary">
      <span>{{ t("ide.total", { count: filteredIdeSkills.length }) }}</span>
      <div class="hint">{{ t("ide.switchHint") }}</div>
    </div>
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

    <div v-if="localLoading" class="hint">{{ t("ide.loading") }}</div>
    <div v-if="!localLoading && filteredIdeSkills.length === 0" class="hint">{{ t("ide.emptyHint") }}</div>
    <div v-if="filteredIdeSkills.length > 0" class="cards">
      <article
        v-for="(skill, index) in filteredIdeSkills"
        :key="skill.id"
        class="card"
        :class="{ unmanaged: !skill.managed }"
      >
        <div class="card-header">
          <div>
            <div class="card-title-row">
              <div class="card-title">{{ index + 1 }}. {{ skill.name }}</div>
              <span v-if="!skill.managed" class="status-badge unmanaged">
                {{ t("ide.unmanaged") }}
              </span>
            </div>
            <div class="card-meta">
              {{
                skill.ide +
                " · " +
                (skill.source === "link" ? t("ide.sourceLink") : t("ide.sourceLocal")) +
                (!skill.managed ? ` · ${t("ide.unmanaged")}` : "")
              }}
            </div>
          </div>
          <div class="card-actions">
            <button class="ghost" @click="$emit('openDir', skill.path)">{{ t("ide.openDir") }}</button>
            <button
              v-if="!skill.managed"
              class="ghost"
              @click="$emit('adopt', skill)"
            >
              {{ t("ide.adopt") }}
            </button>
            <button class="ghost" @click="$emit('uninstall', skill.path)">{{ t("ide.uninstall") }}</button>
          </div>
        </div>
        <div class="card-link">{{ skill.path }}</div>
      </article>
    </div>
  </section>
</template>

<style scoped>
.panel-summary {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: center;
  margin-bottom: 12px;
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

.card-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 8px;
}

.card-title-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
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
</style>
