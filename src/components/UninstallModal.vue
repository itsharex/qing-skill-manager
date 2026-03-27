<script setup lang="ts">
import { useI18n } from "vue-i18n";
import BaseModal from "./BaseModal.vue";

defineProps<{
  visible: boolean;
  targetName: string;
  mode: "ide" | "local";
}>();

defineEmits<{
  (e: "confirm"): void;
  (e: "cancel"): void;
}>();

const { t } = useI18n();
</script>

<template>
  <BaseModal
    :show="visible"
    :title="mode === 'local' ? t('uninstallModal.deleteTitle') : t('uninstallModal.title')"
    size="small"
    @close="$emit('cancel')"
  >
    <div class="hint">
      {{ mode === "local" ? t("uninstallModal.deleteHint") : t("uninstallModal.hint") }}
    </div>
    <div class="card-link">{{ targetName }}</div>

    <template #footer>
      <button class="ghost" @click="$emit('cancel')">{{ t("uninstallModal.cancel") }}</button>
      <button class="primary" @click="$emit('confirm')">
        {{ mode === "local" ? t("uninstallModal.deleteConfirm") : t("uninstallModal.confirm") }}
      </button>
    </template>
  </BaseModal>
</template>
