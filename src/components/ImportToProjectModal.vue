<script setup lang="ts">
import { ref, computed } from "vue";
import { useI18n } from "vue-i18n";
import type { LocalSkill, ProjectConfig } from "../composables/types";

const { t } = useI18n();

const props = defineProps<{
  show: boolean;
  project: ProjectConfig | null;
  localSkills: LocalSkill[];
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "link", skillIds: string[], ideLabels: string[]): void;
}>();

const searchQuery = ref("");
const selectedSkills = ref<Set<string>>(new Set());

const projectIdeLabels = computed(() => {
  return props.project?.ideTargets ?? [];
});

const filteredSkills = computed(() => {
  if (!searchQuery.value.trim()) return props.localSkills;
  const query = searchQuery.value.toLowerCase();
  return props.localSkills.filter(skill => 
    skill.name.toLowerCase().includes(query) ||
    skill.description.toLowerCase().includes(query)
  );
});

const alreadyLinkedSkills = computed(() => {
  const labels = projectIdeLabels.value;
  return props.localSkills.filter(skill => 
    labels.some(label => skill.usedBy.includes(label))
  );
});

const availableSkills = computed(() => {
  const linkedIds = new Set(alreadyLinkedSkills.value.map(s => s.id));
  return filteredSkills.value.filter(skill => !linkedIds.has(skill.id));
});

function toggleSkill(skillId: string) {
  if (selectedSkills.value.has(skillId)) {
    selectedSkills.value.delete(skillId);
  } else {
    selectedSkills.value.add(skillId);
  }
}

function selectAll() {
  availableSkills.value.forEach(s => selectedSkills.value.add(s.id));
}

function deselectAll() {
  selectedSkills.value.clear();
}

function handleClose() {
  selectedSkills.value.clear();
  searchQuery.value = "";
  emit("close");
}

function handleLink() {
  const ids = Array.from(selectedSkills.value);
  if (ids.length === 0 || projectIdeLabels.value.length === 0) return;
  
  emit("link", ids, projectIdeLabels.value);
  selectedSkills.value.clear();
  searchQuery.value = "";
}

function getLinkedLabels(skill: LocalSkill): string[] {
  return projectIdeLabels.value.filter(label => skill.usedBy.includes(label));
}
</script>

<template>
  <Teleport to="body">
    <div v-if="show && project" class="modal-overlay" @click.self="handleClose">
      <div class="modal">
        <div class="modal-header">
          <h3>{{ t("projects.importSkills") }}</h3>
          <button class="close-btn" @click="handleClose">×</button>
        </div>

        <div class="modal-content">
          <div class="project-info">
            <div class="project-name">{{ project.name }}</div>
            <div class="project-path">{{ project.path }}</div>
            <div class="ide-targets">
              <span class="label">{{ t("projects.ideTargets", { count: project.ideTargets.length }) }}:</span>
              <span 
                v-for="label in project.ideTargets" 
                :key="label"
                class="ide-badge"
              >
                {{ label }}
              </span>
            </div>
          </div>

          <div class="search-row">
            <input
              v-model="searchQuery"
              type="text"
              class="search-input"
              :placeholder="t('local.searchPlaceholder')"
            />
          </div>

          <div class="list-header">
            <div class="stats">
              {{ t("local.total", { count: availableSkills.length }) }} 
              <span v-if="alreadyLinkedSkills.length > 0" class="linked-count">
                ({{ alreadyLinkedSkills.length }} {{ t("local.linked") }})
              </span>
            </div>
            <div class="actions">
              <button class="text-btn" @click="selectAll">
                {{ t("common.selectAll") }}
              </button>
              <button class="text-btn" @click="deselectAll">
                {{ t("common.deselectAll") }}
              </button>
            </div>
          </div>

          <div class="skills-list">
            <!-- Already linked skills (show as disabled) -->
            <div 
              v-for="skill in alreadyLinkedSkills" 
              :key="skill.id"
              class="skill-item linked"
            >
              <input type="checkbox" checked disabled />
              
              <div class="skill-info">
                <div class="skill-name">{{ skill.name }}</div>
                <div class="skill-desc">{{ skill.description || "-" }}</div>
                <div class="skill-linked-labels">
                  <span class="linked-badge">
                    {{ t("local.linked") }}: {{ getLinkedLabels(skill).join(", ") }}
                  </span>
                </div>
              </div>
            </div>

            <!-- Available skills -->
            <div 
              v-for="skill in availableSkills" 
              :key="skill.id"
              class="skill-item"
              :class="{ selected: selectedSkills.has(skill.id) }"
              @click="toggleSkill(skill.id)"
            >
              <input
                type="checkbox"
                :checked="selectedSkills.has(skill.id)"
                @click.stop
                @change="toggleSkill(skill.id)"
              />
              
              <div class="skill-info">
                <div class="skill-name">{{ skill.name }}</div>
                <div class="skill-desc">{{ skill.description || "-" }}</div>
                <div v-if="skill.usedBy.length > 0" class="skill-usedby">
                  <span class="usedby-label">
                    {{ t("local.usedBy") }}: {{ skill.usedBy.join(", ") }}
                  </span>
                </div>
              </div>
            </div>

            <div v-if="availableSkills.length === 0 && alreadyLinkedSkills.length === 0" class="empty-state">
              {{ t("local.emptyHint") }}
            </div>

            <div v-else-if="availableSkills.length === 0" class="empty-state">
              {{ t("projects.allSkillsLinked") }}
            </div>
          </div>
        </div>

        <div class="modal-footer">
          <button class="ghost" @click="handleClose">
            {{ t("common.cancel") }}
          </button>
          <button
            class="primary"
            :disabled="selectedSkills.size === 0 || project.ideTargets.length === 0"
            @click="handleLink"
          >
            {{ t("projects.linkSelected", { count: selectedSkills.size }) }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.modal {
  background: var(--color-bg);
  border-radius: 12px;
  max-width: 800px;
  width: 100%;
  max-height: 90vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-border);
}

.modal-header h3 {
  margin: 0;
  font-size: 18px;
}

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: var(--color-muted);
  padding: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
}

.close-btn:hover {
  background: var(--color-hover);
}

.modal-content {
  padding: 20px;
  overflow-y: auto;
  flex: 1;
}

.project-info {
  background: var(--color-card-bg);
  padding: 16px;
  border-radius: 8px;
  margin-bottom: 20px;
}

.project-name {
  font-weight: 600;
  font-size: 16px;
  margin-bottom: 4px;
}

.project-path {
  font-size: 12px;
  color: var(--color-muted);
  font-family: monospace;
  word-break: break-all;
  margin-bottom: 12px;
}

.ide-targets {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.ide-targets .label {
  font-size: 12px;
  color: var(--color-muted);
}

.ide-badge {
  padding: 4px 8px;
  background: var(--color-primary-bg);
  color: var(--color-primary-text);
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
}

.search-row {
  margin-bottom: 16px;
}

.search-input {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  font-size: 14px;
  background: var(--color-bg);
  color: var(--color-text);
}

.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.stats {
  font-size: 14px;
  color: var(--color-text);
}

.linked-count {
  color: var(--color-success-text);
  font-weight: 600;
}

.actions {
  display: flex;
  gap: 16px;
}

.text-btn {
  background: none;
  border: none;
  color: var(--color-primary-bg);
  cursor: pointer;
  font-size: 13px;
  padding: 4px 8px;
}

.text-btn:hover {
  text-decoration: underline;
}

.skills-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.skill-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 12px;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
}

.skill-item:hover {
  background: var(--color-hover);
}

.skill-item.selected {
  border-color: var(--color-primary-bg);
  background: var(--color-primary-bg-alpha);
}

.skill-item.linked {
  opacity: 0.7;
  background: var(--color-success-bg);
  border-color: var(--color-success-border);
  cursor: default;
}

.skill-item input[type="checkbox"] {
  margin-top: 2px;
}

.skill-info {
  flex: 1;
  min-width: 0;
}

.skill-name {
  font-weight: 600;
  font-size: 14px;
  margin-bottom: 4px;
}

.skill-desc {
  font-size: 12px;
  color: var(--color-muted);
  margin-bottom: 4px;
}

.skill-linked-labels {
  margin-top: 4px;
}

.linked-badge {
  padding: 2px 8px;
  background: var(--color-success-bg);
  color: var(--color-success-text);
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
}

.skill-usedby {
  margin-top: 4px;
}

.usedby-label {
  padding: 2px 8px;
  background: var(--color-chip-bg);
  color: var(--color-muted);
  border-radius: 4px;
  font-size: 11px;
}

.empty-state {
  text-align: center;
  padding: 40px;
  color: var(--color-muted);
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 16px 20px;
  border-top: 1px solid var(--color-border);
}

.modal-footer button {
  padding: 8px 20px;
}

.modal-footer button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
