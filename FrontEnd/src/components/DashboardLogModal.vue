<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { X } from '@lucide/vue'
import DashboardLogPanel from './DashboardLogPanel.vue'

interface Emits {
  (e: 'close'): void
}

defineEmits<Emits>()

const closeButton = ref<HTMLButtonElement | null>(null)

onMounted(() => {
  closeButton.value?.focus()
})
</script>

<template>
  <div class="log-modal-overlay" @click="$emit('close')">
    <div
      class="log-modal-panel"
      role="dialog"
      aria-modal="true"
      aria-label="Registro del dashboard"
      @click.stop
    >
      <button
        ref="closeButton"
        class="log-modal-close"
        type="button"
        title="Cerrar"
        aria-label="Cerrar registro del dashboard"
        @click="$emit('close')"
      >
        <X :size="17" />
      </button>

      <DashboardLogPanel />
    </div>
  </div>
</template>

<style scoped>
.log-modal-overlay {
  position: fixed;
  inset: 0;
  z-index: 1150;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  overflow: auto;
  background: var(--overlay);
  backdrop-filter: blur(6px);
}

.log-modal-panel {
  position: relative;
  width: 100%;
  max-width: min(1100px, calc(100vw - 48px));
  max-height: calc(100vh - 48px);
  overflow: hidden;
  padding: 20px;
  border: 1px solid var(--border);
  border-radius: var(--radius-xl);
  background: var(--surface);
  box-shadow: var(--shadow-lg);
  animation: logModalIn 180ms ease-out;
}

.log-modal-close {
  position: absolute;
  top: 16px;
  right: 16px;
  z-index: 1;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  padding: 0;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface);
  color: var(--text-muted);
  cursor: pointer;
}

.log-modal-close:hover {
  border-color: var(--border-strong);
  background: var(--surface-2);
  color: var(--text);
}

.log-modal-close:focus-visible {
  outline: none;
  box-shadow: var(--focus-ring);
}

:deep(.panel-head) {
  padding-right: 42px;
}

:deep(.log-list) {
  max-height: 55vh;
}

@media (max-width: 620px) {
  .log-modal-overlay {
    padding: 12px;
  }

  .log-modal-panel {
    max-width: calc(100vw - 24px);
    max-height: calc(100vh - 24px);
    padding: 14px;
  }

  :deep(.log-list) {
    max-height: 50vh;
  }
}

@media (prefers-reduced-motion: reduce) {
  .log-modal-panel {
    animation: none;
  }
}

@keyframes logModalIn {
  from {
    opacity: 0;
    transform: translateY(8px) scale(0.985);
  }
  to {
    opacity: 1;
    transform: none;
  }
}
</style>
