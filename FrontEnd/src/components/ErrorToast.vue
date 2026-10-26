<script setup lang="ts">
import { AlertTriangle, ScrollText, X } from '@lucide/vue'
import type { AppNotification } from '../types'

interface Props {
  items: AppNotification[]
}

interface Emits {
  (e: 'dismiss', id: string): void
  (e: 'open-log', item: AppNotification): void
}

defineProps<Props>()
defineEmits<Emits>()

function formatTime(timestamp: number): string {
  return new Intl.DateTimeFormat('es', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  }).format(new Date(timestamp))
}
</script>

<template>
  <Teleport to="body">
    <div class="toast-stack" aria-live="assertive" aria-label="Errores recientes">
      <article v-for="item in items" :key="item.id" class="error-toast" :class="`is-${item.level}`">
        <div class="toast-icon">
          <AlertTriangle :size="17" />
        </div>
        <div class="toast-content">
          <div class="toast-head">
            <strong>{{ item.title }}</strong>
            <span>{{ formatTime(item.createdAt) }}</span>
          </div>
          <p>{{ item.message }}</p>
          <div class="toast-actions">
            <button class="toast-log" type="button" @click="$emit('open-log', item)">
              <ScrollText :size="13" />
              Ver registro
            </button>
          </div>
        </div>
        <button
          class="toast-close"
          type="button"
          title="Cerrar aviso"
          aria-label="Cerrar aviso"
          @click="$emit('dismiss', item.id)"
        >
          <X :size="14" />
        </button>
      </article>
    </div>
  </Teleport>
</template>

<style scoped>
.toast-stack {
  position: fixed;
  right: 20px;
  bottom: 20px;
  z-index: 1200;
  display: flex;
  flex-direction: column;
  gap: 10px;
  width: min(380px, calc(100vw - 24px));
  pointer-events: none;
}

.error-toast {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 12px;
  border: 1px solid var(--danger);
  border-left-width: 4px;
  border-radius: var(--radius-md);
  background: var(--surface);
  box-shadow: var(--shadow-lg);
  pointer-events: auto;
  animation: toastIn 180ms ease-out;
}

.error-toast.is-warn {
  border-color: var(--warning);
}

.toast-icon {
  display: grid;
  place-items: center;
  width: 32px;
  height: 32px;
  flex-shrink: 0;
  border-radius: 9px;
  background: var(--danger-soft);
  color: var(--danger);
}

.is-warn .toast-icon {
  background: var(--warning-soft);
  color: var(--warning);
}

.toast-content {
  min-width: 0;
  flex: 1;
}

.toast-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
}

.toast-head strong {
  overflow: hidden;
  color: var(--text);
  font-size: 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.toast-head span {
  flex-shrink: 0;
  color: var(--text-subtle);
  font-family: var(--mono);
  font-size: 10px;
}

.toast-content p {
  margin: 4px 0 0;
  overflow-wrap: anywhere;
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.45;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.toast-actions {
  display: flex;
  margin-top: 7px;
}

.toast-log {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 0;
  border: 0;
  background: transparent;
  color: var(--primary);
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
}

.toast-log:hover {
  text-decoration: underline;
}

.toast-close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  flex-shrink: 0;
  padding: 0;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  color: var(--text-subtle);
  cursor: pointer;
}

.toast-close:hover {
  border-color: var(--border);
  background: var(--surface-2);
  color: var(--text);
}

@keyframes toastIn {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
  to {
    opacity: 1;
    transform: none;
  }
}

@media (max-width: 520px) {
  .toast-stack {
    right: 12px;
    bottom: 12px;
  }
}
</style>
