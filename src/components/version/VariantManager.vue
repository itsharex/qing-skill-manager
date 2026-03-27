<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import type { SkillPackage, SkillVariant, SkillVersion } from "../../composables/types";

defineProps<{
  skillPackage: SkillPackage;
  sortedVersions: SkillVersion[];
}>();

const emit = defineEmits<{
  (e: "createVariant", versionId: string, name: string, description?: string): void;
  (e: "updateVariant", variantId: string, newName?: string, newVersionId?: string, newDescription?: string): void;
  (e: "deleteVariant", variantId: string): void;
}>();

const { t } = useI18n();

const variantName = ref("");
const variantDescription = ref("");
const variantVersionId = ref<string>("");
const editingVariantId = ref<string | null>(null);
const editingVariantName = ref("");
const editingVariantDescription = ref("");
const editingVariantVersionId = ref("");

function handleCreateVariant() {
  if (!variantName.value.trim() || !variantVersionId.value) return;
  emit("createVariant", variantVersionId.value, variantName.value.trim(), variantDescription.value.trim() || undefined);
  variantName.value = "";
  variantDescription.value = "";
  variantVersionId.value = "";
}

function handleDeleteVariant(variantId: string) {
  emit("deleteVariant", variantId);
}

function openVariantEditor(variant: SkillVariant) {
  editingVariantId.value = variant.id;
  editingVariantName.value = variant.name;
  editingVariantDescription.value = variant.description || "";
  editingVariantVersionId.value = variant.currentVersion;
}

function resetVariantEditor() {
  editingVariantId.value = null;
  editingVariantName.value = "";
  editingVariantDescription.value = "";
  editingVariantVersionId.value = "";
}

function saveVariantEditor() {
  if (!editingVariantId.value) return;
  emit(
    "updateVariant",
    editingVariantId.value,
    editingVariantName.value.trim() || undefined,
    editingVariantVersionId.value || undefined,
    editingVariantDescription.value.trim() || undefined
  );
  resetVariantEditor();
}

function reset() {
  variantName.value = "";
  variantDescription.value = "";
  variantVersionId.value = "";
  resetVariantEditor();
}

defineExpose({ reset });
</script>

<template>
  <section class="variants-section section-card">
    <div class="section-header">
      <div>
        <h4>{{ t("version.variants") }}</h4>
        <p class="section-help">{{ t("version.variantsHelp") }}</p>
      </div>
    </div>
    <div class="variant-create-row">
      <select v-model="variantVersionId" class="version-select">
        <option value="">{{ t("version.selectVersion") }}</option>
        <option v-for="version in sortedVersions" :key="version.id" :value="version.id">
          {{ version.displayName }}
        </option>
      </select>
      <input v-model="variantName" class="input compact-input" :placeholder="t('version.variantNamePlaceholder')" />
      <input v-model="variantDescription" class="input compact-input" :placeholder="t('version.variantDescriptionPlaceholder')" />
      <button class="primary" :disabled="!variantName.trim() || !variantVersionId" @click="handleCreateVariant">
        {{ t("version.createVariant") }}
      </button>
    </div>

    <div v-if="skillPackage.variants.length === 0" class="empty-variants">
      {{ t("version.noVariants") }}
    </div>

    <div v-else class="variant-list">
      <div v-for="variant in skillPackage.variants" :key="variant.id" class="variant-item">
        <div class="variant-main">
          <div class="variant-name">{{ variant.name }}</div>
          <div class="variant-meta">{{ variant.description || t("version.noVariantDescription") }}</div>
          <div class="variant-meta code">{{ variant.currentVersion }}</div>
        </div>
        <div class="variant-actions">
          <button class="ghost" @click="openVariantEditor(variant)">
            {{ t("version.editVariant") }}
          </button>
          <button class="ghost danger btn-sm" @click="handleDeleteVariant(variant.id)">
            {{ t("version.deleteVariant") }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="editingVariantId" class="variant-editor">
      <h5>{{ t("version.editVariantTitle") }}</h5>
      <div class="variant-create-row">
        <select v-model="editingVariantVersionId" class="version-select">
          <option value="">{{ t("version.selectVersion") }}</option>
          <option v-for="version in sortedVersions" :key="version.id" :value="version.id">
            {{ version.displayName }}
          </option>
        </select>
        <input v-model="editingVariantName" class="input compact-input" :placeholder="t('version.variantNamePlaceholder')" />
        <input v-model="editingVariantDescription" class="input compact-input" :placeholder="t('version.variantDescriptionPlaceholder')" />
      </div>
      <div class="variant-editor-actions">
        <button class="ghost" @click="resetVariantEditor">
          {{ t("common.cancel") }}
        </button>
        <button class="primary" :disabled="!editingVariantName.trim() || !editingVariantVersionId" @click="saveVariantEditor">
          {{ t("version.saveVariant") }}
        </button>
      </div>
    </div>
  </section>
</template>

<style scoped>
.section-card {
  background: var(--color-card-bg);
  border: 1px solid var(--color-card-border);
  border-radius: 12px;
  padding: 18px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: flex-start;
  margin-bottom: 14px;
}

.section-header h4 {
  margin: 0;
  font-size: 16px;
}

.section-help {
  margin: 8px 0 0 0;
  font-size: 13px;
  color: var(--color-muted);
}

.variants-section {
  margin-top: 24px;
}

.variants-section h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
}

.variant-create-row {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
  margin-bottom: 16px;
}

.compact-input {
  flex: 1 1 180px;
}

.empty-variants {
  color: var(--color-muted);
  font-size: 13px;
  padding: 12px 0;
}

.variant-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.variant-item {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  align-items: flex-start;
  padding: 12px;
  border-radius: 8px;
  background: var(--color-card-bg);
  border: 1px solid var(--color-card-border);
}

.variant-main {
  flex: 1;
}

.variant-name {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 4px;
}

.variant-meta {
  font-size: 12px;
  color: var(--color-muted);
}

.variant-meta.code {
  font-family: monospace;
  margin-top: 4px;
}

.variant-actions {
  display: flex;
  gap: 8px;
}

.variant-editor {
  margin-top: 16px;
  padding: 16px;
  border-radius: 8px;
  border: 1px solid var(--color-border);
  background: var(--color-bg);
}

.variant-editor .version-select {
  flex: 1 1 220px;
}

.variant-editor h5 {
  margin: 0 0 12px 0;
  font-size: 13px;
}

.variant-editor-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 12px;
}

.variant-create-row .version-select {
  width: 100%;
}

.version-select {
  appearance: none;
  -webkit-appearance: none;
  -moz-appearance: none;
  padding: 12px 42px 12px 14px;
  border: 1px solid var(--color-card-border);
  border-radius: 10px;
  background-color: var(--color-bg);
  background-image:
    linear-gradient(45deg, transparent 50%, var(--color-muted) 50%),
    linear-gradient(135deg, var(--color-muted) 50%, transparent 50%),
    linear-gradient(to right, transparent, transparent);
  background-position:
    calc(100% - 18px) calc(50% - 3px),
    calc(100% - 12px) calc(50% - 3px),
    100% 0;
  background-size: 6px 6px, 6px 6px, 2.5em 2.5em;
  background-repeat: no-repeat;
  color: var(--color-text);
  font-size: 14px;
  min-width: 180px;
  box-shadow: 0 1px 0 rgba(255, 255, 255, 0.03), inset 0 1px 2px rgba(0, 0, 0, 0.06);
  transition: border-color 0.18s ease, box-shadow 0.18s ease, transform 0.18s ease, background-color 0.18s ease;
}

.version-select:hover {
  border-color: color-mix(in srgb, var(--color-primary-bg) 45%, var(--color-card-border));
  background-color: color-mix(in srgb, var(--color-bg) 88%, var(--color-card-bg) 12%);
}

.version-select:focus {
  outline: none;
  border-color: var(--color-primary-bg);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-primary-bg) 18%, transparent), inset 0 1px 2px rgba(0, 0, 0, 0.04);
}

.version-select:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  background-color: color-mix(in srgb, var(--color-bg) 80%, var(--color-card-bg) 20%);
}

.version-select option {
  color: var(--color-text);
  background: var(--color-bg);
}

button.danger {
  background: var(--color-error-bg);
  color: var(--color-error-text);
  border-color: var(--color-error-border);
}

button.danger:hover {
  background: var(--color-error-border);
}
</style>
