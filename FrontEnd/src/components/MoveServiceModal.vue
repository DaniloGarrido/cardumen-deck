<script setup lang="ts">
import { computed, ref } from 'vue'
import { AlertTriangle, Check, FolderInput, X } from '@lucide/vue'
import { logDashboardError } from '../composables/useDashboardLog'
import { useServices } from '../composables/useServices'
import type { Service, Workspace, WorkspaceStats } from '../types'

interface Props {
  service: Service
  workspaces: Workspace[]
  stats: Record<string, WorkspaceStats>
}

interface Emits {
  (e: 'close'): void
  (e: 'moved'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const { move, getRuntime } = useServices()
const destinations = computed(() =>
  props.workspaces.filter((workspace) => workspace.id !== props.service.workspaceId),
)
const selectedId = ref(destinations.value[0]?.id ?? '')
const busy = ref(false)
const error = ref('')
const isRunning = computed(() => Boolean(getRuntime(props.service.id)?.pidAlive))
const currentWorkspaceName = computed(
  () => props.workspaces.find((workspace) => workspace.id === props.service.workspaceId)?.name ?? 'Sin workspace',
)

async function submit() {
  if (!selectedId.value || busy.value) return

  busy.value = true
  error.value = ''
  try {
    await move(props.service.id, selectedId.value)
    emit('moved')
  } catch (cause) {
    error.value = String(cause)
    void logDashboardError('service.move', cause, {
      title: `No se pudo mover ${props.service.name}`,
      notify: true,
      serviceId: props.service.id,
      serviceName: props.service.name,
      context: {
        fromWorkspaceId: props.service.workspaceId,
        toWorkspaceId: selectedId.value,
      },
    })
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <div class="move-overlay" @click="$emit('close')">
    <div
      class="move-panel"
      role="dialog"
      aria-modal="true"
      aria-labelledby="move-service-title"
      @click.stop
    >
      <header class="move-header">
        <div class="move-title">
          <div class="move-icon"><FolderInput :size="19" /></div>
          <div>
            <h3 id="move-service-title">Mover servicio</h3>
            <p>{{ service.name }}</p>
          </div>
        </div>
        <button class="move-close" type="button" title="Cerrar" aria-label="Cerrar" @click="$emit('close')">
          <X :size="17" />
        </button>
      </header>

      <form class="move-body" @submit.prevent="submit">
        <div class="current-workspace">
          <span>Workspace actual</span>
          <strong>{{ currentWorkspaceName }}</strong>
        </div>

        <div class="move-note">
          <FolderInput :size="14" />
          <span v-if="isRunning">El servicio seguirá ejecutándose en segundo plano.</span>
          <span v-else>El servicio conservará su configuración y logs.</span>
        </div>

        <div class="destination-heading">Mover a</div>

        <div v-if="destinations.length" class="destination-list">
          <label
            v-for="workspace in destinations"
            :key="workspace.id"
            class="destination"
            :class="{ 'is-selected': selectedId === workspace.id }"
          >
            <input v-model="selectedId" type="radio" :value="workspace.id" :disabled="busy" />
            <span class="destination-dot" :style="{ background: workspace.color || 'var(--primary)' }"></span>
            <span class="destination-content">
              <strong>{{ workspace.name }}</strong>
              <span>{{ stats[workspace.id]?.total ?? 0 }} servicios</span>
            </span>
            <span class="destination-check">
              <Check v-if="selectedId === workspace.id" :size="14" />
            </span>
          </label>
        </div>

        <div v-else class="move-empty">
          Necesitas al menos otro workspace para mover este servicio.
        </div>

        <p v-if="error" class="move-error" role="alert">
          <AlertTriangle :size="14" />
          {{ error }}
        </p>

        <footer class="move-actions">
          <button class="btn btn-ghost" type="button" @click="$emit('close')">
            Cancelar
          </button>
          <button class="btn btn-primary" type="submit" :disabled="!selectedId || busy">
            <FolderInput :size="15" />
            {{ busy ? 'Moviendo...' : 'Mover servicio' }}
          </button>
        </footer>
      </form>
    </div>
  </div>
</template>

<style scoped>
.move-overlay {
  position: fixed;
  inset: 0;
  z-index: 1150;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  background: var(--overlay);
  backdrop-filter: blur(6px);
}

.move-panel {
  width: 100%;
  max-width: 480px;
  max-height: 90vh;
  overflow: auto;
  border: 1px solid var(--border);
  border-radius: var(--radius-xl);
  background: var(--surface);
  box-shadow: var(--shadow-lg);
  animation: moveIn 180ms ease-out;
}

.move-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 18px 20px;
  border-bottom: 1px solid var(--border);
}

.move-title {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.move-icon {
  display: grid;
  place-items: center;
  width: 40px;
  height: 40px;
  flex-shrink: 0;
  border-radius: 12px;
  background: var(--primary-soft);
  color: var(--primary);
}

.move-title h3 {
  margin: 0;
  color: var(--text);
  font-size: 16px;
  letter-spacing: -0.01em;
}

.move-title p {
  margin: 3px 0 0;
  overflow: hidden;
  color: var(--text-muted);
  font-size: 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.move-close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  flex-shrink: 0;
  padding: 0;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface);
  color: var(--text-muted);
  cursor: pointer;
}

.move-close:hover {
  border-color: var(--border-strong);
  background: var(--surface-2);
  color: var(--text);
}

.move-body {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 18px 20px 20px;
}

.current-workspace {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 9px 11px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  color: var(--text-muted);
  font-size: 11px;
}

.current-workspace strong {
  color: var(--text);
  font-size: 12px;
}

.move-note {
  display: flex;
  align-items: flex-start;
  gap: 6px;
  color: var(--text-subtle);
  font-size: 11px;
  line-height: 1.4;
}

.move-note svg {
  flex-shrink: 0;
  margin-top: 1px;
}

.destination-heading {
  color: var(--text-muted);
  font-size: 10px;
  font-weight: 800;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.destination-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.destination {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface);
  cursor: pointer;
  transition:
    border-color var(--transition),
    background var(--transition);
}

.destination:hover {
  border-color: var(--border-strong);
  background: var(--surface-2);
}

.destination.is-selected {
  border-color: var(--primary);
  background: var(--primary-soft);
}

.destination input {
  position: absolute;
  opacity: 0;
  pointer-events: none;
}

.destination-dot {
  width: 10px;
  height: 10px;
  flex-shrink: 0;
  border-radius: 999px;
}

.destination-content {
  display: flex;
  flex-direction: column;
  min-width: 0;
  flex: 1;
  gap: 2px;
}

.destination-content strong {
  overflow: hidden;
  color: var(--text);
  font-size: 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.destination-content span {
  color: var(--text-subtle);
  font-size: 10px;
}

.destination-check {
  display: grid;
  place-items: center;
  width: 20px;
  height: 20px;
  flex-shrink: 0;
  color: var(--primary);
}

.move-empty {
  padding: 16px;
  border: 1px dashed var(--border);
  border-radius: var(--radius-sm);
  color: var(--text-subtle);
  font-size: 12px;
  text-align: center;
}

.move-error {
  display: flex;
  align-items: flex-start;
  gap: 6px;
  margin: 0;
  padding: 9px 10px;
  border: 1px solid var(--danger);
  border-radius: var(--radius-sm);
  background: var(--danger-soft);
  color: var(--danger);
  font-size: 11px;
}

.move-error svg {
  flex-shrink: 0;
}

.move-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding-top: 4px;
}

@media (max-width: 520px) {
  .move-overlay {
    padding: 12px;
  }

  .move-header {
    padding: 16px;
  }

  .move-body {
    padding: 16px;
  }

  .move-actions {
    flex-direction: column-reverse;
  }

  .move-actions .btn {
    justify-content: center;
    width: 100%;
  }
}

@keyframes moveIn {
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
