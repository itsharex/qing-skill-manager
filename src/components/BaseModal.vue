<script setup lang="ts">
import { onUnmounted, watch } from "vue";

const props = defineProps<{
  show: boolean;
  title?: string;
  size?: "small" | "medium" | "large";
}>();

const emit = defineEmits<{
  close: [];
}>();

function handleKeydown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    emit("close");
  }
}

watch(
  () => props.show,
  (val) => {
    if (val) {
      document.addEventListener("keydown", handleKeydown);
    } else {
      document.removeEventListener("keydown", handleKeydown);
    }
  }
);

onUnmounted(() => {
  document.removeEventListener("keydown", handleKeydown);
});
</script>

<template>
  <Teleport to="body">
    <Transition name="modal-fade">
      <div
        v-if="show"
        class="base-modal-overlay"
        role="dialog"
        aria-modal="true"
        @click.self="$emit('close')"
      >
        <div
          class="base-modal"
          :class="size ? `base-modal--${size}` : ''"
        >
          <div v-if="title || $slots.header" class="base-modal-header">
            <h2 class="base-modal-title">
              <slot name="header">{{ title }}</slot>
            </h2>
            <button
              class="base-modal-close"
              aria-label="Close"
              @click="$emit('close')"
            >
              &times;
            </button>
          </div>
          <div class="base-modal-body">
            <slot />
          </div>
          <div v-if="$slots.footer" class="base-modal-footer">
            <slot name="footer" />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.base-modal-overlay {
  position: fixed;
  inset: 0;
  background: var(--color-overlay-bg);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.base-modal {
  background: var(--color-panel-bg);
  border-radius: 12px;
  width: 100%;
  max-width: 500px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12);
}

.base-modal--small {
  max-width: 420px;
}

.base-modal--medium {
  max-width: 600px;
}

.base-modal--large {
  max-width: 900px;
}

.base-modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid var(--color-panel-border);
}

.base-modal-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text);
  margin: 0;
}

.base-modal-close {
  background: transparent;
  border: none;
  font-size: 28px;
  color: var(--color-muted);
  cursor: pointer;
  padding: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  transition: background 0.2s ease;
}

.base-modal-close:hover {
  background: var(--color-tabs-bg);
  color: var(--color-text);
}

.base-modal-body {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 24px;
}

.base-modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 16px 24px;
  border-top: 1px solid var(--color-panel-border);
}

.base-modal-footer :deep(button:disabled) {
  opacity: 0.5;
  cursor: not-allowed;
}

.modal-fade-enter-active,
.modal-fade-leave-active {
  transition: opacity 0.2s ease;
}

.modal-fade-enter-from,
.modal-fade-leave-to {
  opacity: 0;
}
</style>
