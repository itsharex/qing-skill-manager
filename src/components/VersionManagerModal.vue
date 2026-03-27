<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import type { DeleteStrategy, ProjectConfig, ProjectSkill, SkillPackage, SkillVersion } from "../composables/types";
import VersionCreateForm from "./version/VersionCreateForm.vue";
import VersionList from "./version/VersionList.vue";
import VariantManager from "./version/VariantManager.vue";

const props = defineProps<{
  show: boolean;
  skillPackage: SkillPackage | null;
  currentSkillPath?: string;
  selectedSourcePath?: string;
  projects?: ProjectConfig[];
  projectSkills?: ProjectSkill[];
  selectedProjectId?: string | null;
  projectSkillsLoading?: boolean;
  loading: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "rename", versionId: string, newName: string): void;
  (e: "delete", versionId: string, strategy: DeleteStrategy, force: boolean): void;
  (e: "setDefault", versionId: string): void;
  (e: "compare", fromVersionId: string, toVersionId: string): void;
  (e: "createVersion", version: string, displayName: string, sourcePath: string, parentVersion?: string): void;
  (e: "pickSourcePath"): void;
  (e: "pickProject", projectId: string): void;
  (e: "createVariant", versionId: string, name: string, description?: string): void;
  (e: "updateVariant", variantId: string, newName?: string, newVersionId?: string, newDescription?: string): void;
  (e: "deleteVariant", variantId: string): void;
}>();

const { t } = useI18n();

const showRenameModal = ref(false);
const renamingVersion = ref<SkillVersion | null>(null);
const newVersionName = ref("");

const showDeleteModal = ref(false);
const deletingVersion = ref<SkillVersion | null>(null);
const forceDelete = ref(false);
const deleteStrategy = ref<DeleteStrategy>("soft");

const createFormRef = ref<InstanceType<typeof VersionCreateForm> | null>(null);
const variantManagerRef = ref<InstanceType<typeof VariantManager> | null>(null);

const sortedVersions = computed(() => {
  if (!props.skillPackage) return [];
  return [...props.skillPackage.versions].sort((a, b) => b.createdAt - a.createdAt);
});

const defaultVersion = computed(() => {
  if (!props.skillPackage) return null;
  return props.skillPackage.versions.find(v => v.id === props.skillPackage?.defaultVersion) || null;
});

const otherVersions = computed(() => {
  if (!props.skillPackage) return [];
  return sortedVersions.value.filter((version) => version.id !== props.skillPackage?.defaultVersion);
});

function handleClose() {
  showRenameModal.value = false;
  showDeleteModal.value = false;
  renamingVersion.value = null;
  deletingVersion.value = null;
  newVersionName.value = "";
  forceDelete.value = false;
  deleteStrategy.value = "soft";
  createFormRef.value?.reset();
  variantManagerRef.value?.reset();
  emit("close");
}

function openRenameModal(version: SkillVersion) {
  renamingVersion.value = version;
  newVersionName.value = version.displayName;
  showRenameModal.value = true;
}

function confirmRename() {
  if (renamingVersion.value && newVersionName.value.trim()) {
    emit("rename", renamingVersion.value.id, newVersionName.value.trim());
    showRenameModal.value = false;
    newVersionName.value = "";
    renamingVersion.value = null;
  }
}

function openDeleteModal(version: SkillVersion) {
  deletingVersion.value = version;
  forceDelete.value = false;
  deleteStrategy.value = "soft";
  showDeleteModal.value = true;
}

function confirmDelete() {
  if (deletingVersion.value) {
    emit("delete", deletingVersion.value.id, deleteStrategy.value, forceDelete.value);
    showDeleteModal.value = false;
    deletingVersion.value = null;
    forceDelete.value = false;
    deleteStrategy.value = "soft";
  }
}

function handleCompare(fromVersionId: string, toVersionId: string) {
  emit("compare", fromVersionId, toVersionId);
}
</script>

<template>
  <Teleport to="body">
    <div v-if="show" class="modal-overlay" @click.self="handleClose">
      <div class="modal">
        <div class="modal-header">
          <h3>{{ skillPackage?.name ? t("version.title", { name: skillPackage.name }) : t("version.loading") }}</h3>
          <button class="close-btn" @click="handleClose">&times;</button>
        </div>

        <div class="modal-content">
          <div v-if="loading" class="loading-state">
            {{ t("version.loading") }}
          </div>

          <div v-else-if="!skillPackage || skillPackage.versions.length === 0" class="empty-state">
            {{ t("version.noVersions") }}
          </div>

          <div v-else>
            <section class="hero-section">
              <div class="hero-card">
                <div class="hero-text">
                  <div class="hero-label">{{ t("version.currentVersionSection") }}</div>
                  <div class="hero-title">{{ defaultVersion?.displayName || "-" }}</div>
                  <div class="hero-meta">
                    <span>{{ t("version.defaultVersionId") }}: {{ defaultVersion?.version || "-" }}</span>
                    <span>
                      {{ skillPackage.defaultVersionSource === 'explicit' ? t('version.defaultVersionSourceExplicit') : t('version.defaultVersionSourceStrategy') }}
                    </span>
                    <span>{{ t("version.totalVersions") }}: {{ skillPackage.versions.length }}</span>
                  </div>
                  <p class="hero-help">{{ t("version.currentVersionHelp") }}</p>
                </div>
              </div>
            </section>

            <VersionCreateForm
              ref="createFormRef"
              :skill-package="skillPackage"
              :sorted-versions="sortedVersions"
              :projects="projects"
              :project-skills="projectSkills"
              :project-skills-loading="projectSkillsLoading"
              :selected-source-path="selectedSourcePath"
              :selected-project-id="selectedProjectId"
              @create-version="(version, displayName, sourcePath, parentVersion) => emit('createVersion', version, displayName, sourcePath, parentVersion)"
              @pick-source-path="emit('pickSourcePath')"
              @pick-project="(projectId) => emit('pickProject', projectId)"
            />

            <VersionList
              :skill-package="skillPackage"
              :default-version="defaultVersion"
              :other-versions="otherVersions"
              @set-default="(versionId) => emit('setDefault', versionId)"
              @compare="handleCompare"
              @open-rename="openRenameModal"
              @open-delete="openDeleteModal"
            />

            <VariantManager
              ref="variantManagerRef"
              :skill-package="skillPackage"
              :sorted-versions="sortedVersions"
              @create-variant="(versionId, name, description) => emit('createVariant', versionId, name, description)"
              @update-variant="(variantId, newName, newVersionId, newDescription) => emit('updateVariant', variantId, newName, newVersionId, newDescription)"
              @delete-variant="(variantId) => emit('deleteVariant', variantId)"
            />
          </div>
        </div>

        <div class="modal-footer">
          <button class="ghost" @click="handleClose">
            {{ t("common.cancel") }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="showRenameModal" class="submodal-overlay" @click.self="showRenameModal = false">
      <div class="submodal">
        <h4>{{ t("version.renameTitle") }}</h4>
        <input
          v-model="newVersionName"
          type="text"
          class="input"
          :placeholder="t('version.renamePlaceholder')"
          @keyup.enter="confirmRename"
        />
        <div class="submodal-actions">
          <button class="ghost" @click="showRenameModal = false">
            {{ t("common.cancel") }}
          </button>
          <button
            class="primary"
            :disabled="!newVersionName.trim()"
            @click="confirmRename"
          >
            {{ t("version.confirmRename") }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="showDeleteModal" class="submodal-overlay" @click.self="showDeleteModal = false">
      <div class="submodal">
        <h4>{{ t("version.deleteTitle") }}</h4>
        <p class="delete-warning">{{ t("version.deleteWarning", { name: deletingVersion?.displayName }) }}</p>
        <div class="delete-strategy-group">
          <label class="strategy-option">
            <input v-model="deleteStrategy" type="radio" value="soft" />
            <div>
              <strong>{{ t("version.deleteStrategySoft") }}</strong>
              <div class="strategy-desc">{{ t("version.deleteStrategySoftDesc") }}</div>
            </div>
          </label>
          <label class="strategy-option">
            <input v-model="deleteStrategy" type="radio" value="archive" />
            <div>
              <strong>{{ t("version.deleteStrategyArchive") }}</strong>
              <div class="strategy-desc">{{ t("version.deleteStrategyArchiveDesc") }}</div>
            </div>
          </label>
          <label class="strategy-option">
            <input v-model="deleteStrategy" type="radio" value="hard" />
            <div>
              <strong>{{ t("version.deleteStrategyHard") }}</strong>
              <div class="strategy-desc">{{ t("version.deleteStrategyHardDesc") }}</div>
            </div>
          </label>
        </div>
        <label class="checkbox">
          <input v-model="forceDelete" type="checkbox" />
          {{ t("version.forceDelete") }}
        </label>
        <div class="submodal-actions">
          <button class="ghost" @click="showDeleteModal = false">
            {{ t("common.cancel") }}
          </button>
          <button class="primary danger" @click="confirmDelete">
            {{ t("version.confirmDelete") }}
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
  z-index: 1100;
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

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 16px 20px;
  border-top: 1px solid var(--color-border);
}

.loading-state,
.empty-state {
  text-align: center;
  padding: 40px;
  color: var(--color-muted);
}

.hero-section {
  margin-bottom: 20px;
}

.hero-card {
  background: var(--color-card-bg);
  border: 1px solid var(--color-card-border);
  border-radius: 12px;
  padding: 18px;
}

.hero-label {
  font-size: 12px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--color-muted);
  margin-bottom: 8px;
}

.hero-title {
  font-size: 22px;
  font-weight: 700;
  margin-bottom: 10px;
}

.hero-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  font-size: 12px;
  color: var(--color-muted);
}

.hero-help {
  margin: 8px 0 0 0;
  font-size: 13px;
  color: var(--color-muted);
}

.delete-strategy-group {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin: 16px 0;
}

.strategy-option {
  display: flex;
  gap: 10px;
  align-items: flex-start;
  padding: 10px;
  border-radius: 8px;
  border: 1px solid var(--color-border);
  background: var(--color-card-bg);
}

.strategy-desc {
  font-size: 12px;
  color: var(--color-muted);
  margin-top: 4px;
}

.submodal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1200;
  padding: 20px;
}

.submodal {
  background: var(--color-bg);
  border-radius: 12px;
  padding: 24px;
  max-width: 400px;
  width: 100%;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.4);
}

.submodal h4 {
  margin: 0 0 16px 0;
  font-size: 16px;
}

.submodal .input {
  width: 100%;
  padding: 10px 14px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  font-size: 14px;
  margin-bottom: 16px;
  background: var(--color-bg);
  color: var(--color-text);
}

.delete-warning {
  color: var(--color-warning-text);
  font-size: 14px;
  margin-bottom: 16px;
}

.submodal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.submodal-actions button {
  padding: 8px 16px;
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
