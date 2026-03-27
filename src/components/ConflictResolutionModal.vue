<script setup lang="ts">
import { ref, computed } from "vue";
import { useI18n } from "vue-i18n";
import BaseModal from "./BaseModal.vue";
import type { ProjectSkill, ConflictAnalysis } from "../composables/types";

const { t } = useI18n();

const props = defineProps<{
  show: boolean;
  skill: ProjectSkill | null;
  conflictAnalysis: ConflictAnalysis | null;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "resolve", resolution: "keep" | "overwrite" | "coexist", coexistName?: string): void;
}>();

const selectedResolution = ref<"keep" | "overwrite" | "coexist">("keep");
const coexistName = ref("");

const canSubmit = computed(() => {
  if (selectedResolution.value === "coexist") {
    return coexistName.value.trim().length > 0;
  }
  return true;
});

const existingSkill = computed(() => props.skill?.existingRegistrySkill);

function handleClose() {
  emit("close");
}

function handleSubmit() {
  if (!canSubmit.value) return;
  emit("resolve", selectedResolution.value, selectedResolution.value === "coexist" ? coexistName.value.trim() : undefined);
  emit("close");
}
</script>

<template>
  <BaseModal :show="show" :title="t('conflict.title')" size="large" @close="handleClose">
         <div v-if="skill" class="modal-content">
            <div class="conflict-info">
              <p class="conflict-desc">{{ t("conflict.description", { name: skill.name }) }}</p>
            </div>

            <div v-if="conflictAnalysis" class="analysis-section">
              <h4>{{ t("conflict.analysis") }}</h4>
              <div class="analysis-badges">
                <span class="analysis-badge" :class="'type-' + conflictAnalysis.conflictType">
                  {{ t(`conflict.type.${conflictAnalysis.conflictType}`) }}
                </span>
                <span class="analysis-badge" :class="'severity-' + conflictAnalysis.severity">
                  {{ t(`conflict.severity.${conflictAnalysis.severity}`) }}
                </span>
                <span v-if="conflictAnalysis.autoResolvable" class="analysis-badge auto-resolvable">
                  {{ t("conflict.autoResolvable") }}
                </span>
              </div>
              <div v-if="conflictAnalysis.suggestions.length > 0" class="suggestions">
                <div
                  v-for="(suggestion, index) in conflictAnalysis.suggestions"
                  :key="index"
                  class="suggestion-item"
                >
                  <span class="suggestion-action">{{ t(`conflict.action.${suggestion.action}`) }}</span>
                  <span class="suggestion-desc">{{ suggestion.description }}</span>
                  <span class="suggestion-confidence">{{ Math.round(suggestion.confidence * 100) }}%</span>
                </div>
              </div>
            </div>

           <div class="comparison">
             <div class="skill-box existing" :class="{ 'no-data': !existingSkill }">
               <h4>{{ t("conflict.existingSkill") }}</h4>
                <div v-if="existingSkill" class="skill-details">
                 <div class="detail-row">
                   <span class="label">{{ t("conflict.name") }}:</span>
                   <span class="value">{{ existingSkill.name }}</span>
                 </div>
                 <div class="detail-row">
                   <span class="label">{{ t("conflict.descLabel") }}:</span>
                   <span class="value">{{ existingSkill.description || "-" }}</span>
                 </div>
                  <div class="detail-row">
                    <span class="label">{{ t("conflict.path") }}:</span>
                    <span class="value path">{{ existingSkill.path }}</span>
                  </div>
                  <div v-if="existingSkill.currentVersion" class="detail-row">
                    <span class="label">{{ t("version.defaultVersion") }}:</span>
                    <span class="value">{{ existingSkill.currentVersion.displayName }}</span>
                  </div>
                </div>
               <div v-else class="no-data-message">
                 {{ t("conflict.noExistingData") }}
               </div>
             </div>

             <div class="vs-divider">VS</div>

             <div class="skill-box incoming">
               <h4>{{ t("conflict.incomingSkill") }}</h4>
                <div class="skill-details">
                 <div class="detail-row">
                   <span class="label">{{ t("conflict.name") }}:</span>
                   <span class="value">{{ skill.name }}</span>
                 </div>
                 <div class="detail-row">
                   <span class="label">{{ t("conflict.descLabel") }}:</span>
                   <span class="value">{{ skill.description || "-" }}</span>
                 </div>
                  <div class="detail-row">
                    <span class="label">{{ t("conflict.path") }}:</span>
                    <span class="value path">{{ skill.path }}</span>
                  </div>
                  <div v-if="skill.currentVersion" class="detail-row">
                    <span class="label">{{ t("version.defaultVersion") }}:</span>
                    <span class="value">{{ skill.currentVersion.displayName }}</span>
                  </div>
                </div>
              </div>
            </div>

            <div v-if="conflictAnalysis?.diff" class="diff-preview">
              <h4>{{ t("diff.title") }}</h4>
              <div class="diff-stats">
                <span>+{{ conflictAnalysis.diff.additions }}</span>
                <span>-{{ conflictAnalysis.diff.deletions }}</span>
                <span>{{ Math.round(conflictAnalysis.diff.similarityScore * 100) }}%</span>
              </div>
              <pre v-if="conflictAnalysis.diff.contentDiff" class="diff-content"><code>{{ conflictAnalysis.diff.contentDiff }}</code></pre>
            </div>

           <div class="resolution-options">
            <h4>{{ t("conflict.chooseResolution") }}</h4>
            
            <label class="option">
              <input 
                v-model="selectedResolution" 
                type="radio" 
                value="keep"
              />
              <div class="option-content">
                <strong>{{ t("conflict.keepExisting") }}</strong>
                <span class="option-desc">{{ t("conflict.keepDesc") }}</span>
              </div>
            </label>

            <label class="option">
              <input 
                v-model="selectedResolution" 
                type="radio" 
                value="overwrite"
              />
              <div class="option-content">
                <strong>{{ t("conflict.overwrite") }}</strong>
                <span class="option-desc">{{ t("conflict.overwriteDesc") }}</span>
              </div>
            </label>

            <label class="option">
              <input 
                v-model="selectedResolution" 
                type="radio" 
                value="coexist"
              />
              <div class="option-content">
                <strong>{{ t("conflict.coexist") }}</strong>
                <span class="option-desc">{{ t("conflict.coexistDesc") }}</span>
              </div>
            </label>

            <div v-if="selectedResolution === 'coexist'" class="coexist-input">
              <label>{{ t("conflict.newName") }}</label>
              <input 
                v-model="coexistName" 
                type="text" 
                :placeholder="t('conflict.newNamePlaceholder')"
              />
            </div>
          </div>
        </div>

        <template #footer>
          <button class="ghost" @click="handleClose">
            {{ t("common.cancel") }}
          </button>
          <button
            class="primary"
            :disabled="!canSubmit"
            @click="handleSubmit"
          >
            {{ t("conflict.confirm") }}
          </button>
        </template>
  </BaseModal>
</template>

<style scoped>
.modal-content {
  /* Content-specific styles only */
}

.conflict-info {
  margin-bottom: 20px;
}

.conflict-desc {
  color: var(--color-warning);
  font-size: 14px;
}

.analysis-section {
  background: var(--color-card-bg);
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 20px;
}

.analysis-section h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  color: var(--color-muted);
}

.analysis-badges {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
  flex-wrap: wrap;
}

.analysis-badge {
  padding: 4px 10px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 600;
}

.analysis-badge.type-identical {
  background: var(--color-chip-bg);
  color: var(--color-muted);
}

.analysis-badge.type-patch {
  background: var(--color-success-bg);
  color: var(--color-success-text);
}

.analysis-badge.type-minor {
  background: #e0f2fe;
  color: #0369a1;
}

.analysis-badge.type-major {
  background: var(--color-warning-bg);
  color: var(--color-warning-text);
}

.analysis-badge.type-fork {
  background: var(--color-error-bg);
  color: var(--color-error-text);
}

.analysis-badge.severity-none {
  background: var(--color-chip-bg);
  color: var(--color-muted);
}

.analysis-badge.severity-minor {
  background: var(--color-success-bg);
  color: var(--color-success-text);
}

.analysis-badge.severity-major {
  background: var(--color-warning-bg);
  color: var(--color-warning-text);
}

.analysis-badge.severity-breaking {
  background: var(--color-error-bg);
  color: var(--color-error-text);
}

.analysis-badge.auto-resolvable {
  background: var(--color-success-bg);
  color: var(--color-success-text);
  border: 1px solid var(--color-success-border);
}

.suggestions {
  border-top: 1px solid var(--color-border);
  padding-top: 12px;
}

.suggestion-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 0;
  border-bottom: 1px solid var(--color-border);
}

.suggestion-item:last-child {
  border-bottom: none;
}

.suggestion-action {
  font-weight: 600;
  font-size: 13px;
  min-width: 120px;
}

.suggestion-desc {
  flex: 1;
  font-size: 13px;
  color: var(--color-text);
}

.suggestion-confidence {
  font-size: 12px;
  color: var(--color-muted);
  background: var(--color-bg);
  padding: 2px 8px;
  border-radius: 4px;
}

.comparison {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  gap: 16px;
  margin-bottom: 24px;
}

.skill-box {
  background: var(--color-card-bg);
  border: 1px solid var(--color-card-border);
  border-radius: 8px;
  padding: 16px;
}

.skill-box.existing {
  border-color: var(--color-info-border);
}

.skill-box.existing.no-data {
  border-color: var(--color-border);
  background: var(--color-bg);
  opacity: 0.7;
}

.skill-box.incoming {
  border-color: var(--color-warning-border);
}

.no-data-message {
  padding: 20px;
  text-align: center;
  color: var(--color-muted);
  font-style: italic;
}

.skill-box h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  color: var(--color-muted);
}

.skill-details {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.detail-row {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.detail-row .label {
  font-size: 11px;
  color: var(--color-muted);
  text-transform: uppercase;
}

.detail-row .value {
  font-size: 13px;
  color: var(--color-text);
}

.detail-row .value.path {
  font-family: monospace;
  font-size: 11px;
  word-break: break-all;
  color: var(--color-muted);
}

.vs-divider {
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  color: var(--color-muted);
  font-size: 12px;
}

.diff-preview {
  margin-bottom: 24px;
  padding: 16px;
  border-radius: 8px;
  background: var(--color-card-bg);
  border: 1px solid var(--color-card-border);
}

.diff-preview h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  color: var(--color-muted);
}

.diff-stats {
  display: flex;
  gap: 12px;
  margin-bottom: 12px;
  font-size: 12px;
  color: var(--color-muted);
}

.diff-content {
  margin: 0;
  padding: 12px;
  border-radius: 6px;
  background: var(--color-bg);
  overflow-x: auto;
  font-size: 12px;
  line-height: 1.5;
  font-family: monospace;
}

.resolution-options {
  border-top: 1px solid var(--color-border);
  padding-top: 20px;
}

.resolution-options h4 {
  margin: 0 0 16px 0;
  font-size: 14px;
}

.option {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 12px;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  margin-bottom: 8px;
  cursor: pointer;
  transition: all 0.2s;
}

.option:hover {
  background: var(--color-hover);
}

.option input[type="radio"] {
  margin-top: 2px;
}

.option-content {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.option-content strong {
  font-size: 14px;
}

.option-desc {
  font-size: 12px;
  color: var(--color-muted);
}

.coexist-input {
  margin-top: 12px;
  padding: 12px;
  background: var(--color-card-bg);
  border-radius: 8px;
}

.coexist-input label {
  display: block;
  font-size: 12px;
  margin-bottom: 8px;
  color: var(--color-muted);
}

.coexist-input input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  font-size: 14px;
  background: var(--color-bg);
  color: var(--color-text);
}

</style>
