<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import BaseModal from "./BaseModal.vue";

const { t } = useI18n();

defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "confirm", path: string, name: string): void;
}>();

const projectPath = ref("");
const projectName = ref("");

async function handleSelectFolder() {
  try {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const selected = await open({
      directory: true,
      multiple: false,
      title: t("projects.selectFolder")
    });

    if (selected && typeof selected === "string") {
      projectPath.value = selected;
      // Extract folder name from path
      const parts = selected.split(/[\\/]/).filter(Boolean);
      projectName.value = parts[parts.length - 1] || t("projects.untitled");
    }
  } catch (err) {
    console.error("Failed to select folder:", err);
  }
}

function handleConfirm() {
  if (projectPath.value.trim() && projectName.value.trim()) {
    emit("confirm", projectPath.value.trim(), projectName.value.trim());
    projectPath.value = "";
    projectName.value = "";
  }
}

function handleClose() {
  emit("close");
  projectPath.value = "";
  projectName.value = "";
}
</script>

<template>
  <BaseModal :show="visible" :title="t('projects.addTitle')" @close="handleClose">
    <div class="form-group">
      <label class="form-label">{{ t("projects.projectPath") }}</label>
      <div class="input-with-button">
        <input
          v-model="projectPath"
          class="input"
          :placeholder="t('projects.pathPlaceholder')"
          readonly
        />
        <button class="icon-button" @click="handleSelectFolder" type="button">
          {{ t("projects.selectFolderButton") }}
        </button>
      </div>
    </div>

    <div class="form-group">
      <label class="form-label">{{ t("projects.projectName") }}</label>
      <input
        v-model="projectName"
        class="input"
        :placeholder="t('projects.namePlaceholder')"
      />
    </div>

    <div class="hint">
      {{ t("projects.addHint") }}
    </div>

    <template #footer>
      <button class="ghost" @click="handleClose">{{ t("projects.cancel") }}</button>
      <button
        class="primary"
        :disabled="!projectPath.trim() || !projectName.trim()"
        @click="handleConfirm"
      >
        {{ t("projects.add") }}
      </button>
    </template>
  </BaseModal>
</template>

<style scoped>
.form-group {
  margin-bottom: 20px;
}

.form-label {
  display: block;
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text);
  margin-bottom: 8px;
}

.input {
  width: 100%;
  padding: 10px 14px;
  background: var(--color-input-bg);
  border: 1px solid var(--color-input-border);
  border-radius: 6px;
  color: var(--color-text);
  font-size: 14px;
  transition: border-color 0.2s ease;
}

.input:focus {
  outline: none;
  border-color: var(--color-input-focus);
}

.input-with-button {
  display: flex;
  gap: 8px;
}

.input-with-button .input {
  flex: 1;
}

.icon-button {
  padding: 10px 16px;
  background: var(--color-primary-bg);
  border: 1px solid var(--color-primary-bg);
  border-radius: 6px;
  color: var(--color-primary-text);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.icon-button:hover {
  background: var(--color-primary-hover);
  border-color: var(--color-primary-hover);
  color: var(--color-primary-text);
}

.icon-button:active {
  background: var(--color-primary-active);
  border-color: var(--color-primary-active);
  color: var(--color-primary-text);
}
</style>
