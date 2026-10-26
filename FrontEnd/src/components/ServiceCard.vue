<script setup lang="ts">
import { ref } from 'vue'
import {
  Check,
  Copy,
  ExternalLink,
  GitBranch,
  Globe,
  GripVertical,
  Info,
  Pencil,
  Play,
  RotateCw,
  ScrollText,
  Square,
  Trash2,
} from '@lucide/vue'
import { useServiceItem } from '../composables/useServiceItem'
import { categoryIconFor } from '../utils/serviceIcons'
import { isServiceDrag, setServiceDragData } from '../utils/serviceDrag'
import ServiceHealthBadge from './ServiceHealthBadge.vue'
import ServiceVitals from './ServiceVitals.vue'
import type { Service, ServiceHealth, ServiceUsage, UsageHistory } from '../types'

interface Props {
  svc: Service
  status: string
  pidAlive: boolean
  gitBranch?: string
  health?: ServiceHealth
  statusClass: string
  categoryLabel: string
  usage?: ServiceUsage
  usageHistory?: UsageHistory
  dragging?: boolean
}

interface Emits {
  (e: 'start', id: string): void
  (e: 'stop', id: string): void
  (e: 'restart', id: string): void
  (e: 'logs', id: string): void
  (e: 'info', svc: Service): void
  (e: 'edit', svc: Service): void
  (e: 'delete', id: string): void
  (e: 'contextmenu', svc: Service, event: MouseEvent): void
  (e: 'git', svc: Service): void
  (e: 'drag-start', svc: Service): void
  (e: 'drag-end'): void
  (e: 'drop-service', targetId: string, after: boolean): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()
const dragOver = ref(false)
const dropAfter = ref(false)

const {
  canStart,
  canStop,
  canRestart,
  statusText,
  accentStyle,
  serviceUrl,
  copied,
  copyUrl,
  openInBrowser,
} = useServiceItem({
  service: () => props.svc,
  status: () => props.status,
  pidAlive: () => props.pidAlive,
})

function onDragStart(event: DragEvent) {
  setServiceDragData(event, props.svc.id)
  const card = (event.currentTarget as HTMLElement).closest('.card')
  if (card) event.dataTransfer?.setDragImage(card, 20, 20)
  emit('drag-start', props.svc)
}

function onDragEnd() {
  dragOver.value = false
  emit('drag-end')
}

function onDragOver(event: DragEvent) {
  if (props.dragging || !isServiceDrag(event)) return
  event.preventDefault()
  if (event.dataTransfer) event.dataTransfer.dropEffect = 'move'
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect()
  dropAfter.value = event.clientX > rect.left + rect.width / 2
  dragOver.value = true
}

function onDragLeave(event: DragEvent) {
  const current = event.currentTarget as HTMLElement
  const next = event.relatedTarget as Node | null
  if (!next || !current.contains(next)) {
    dragOver.value = false
  }
}

function onDrop(event: DragEvent) {
  if (props.dragging || !isServiceDrag(event)) return
  event.preventDefault()
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect()
  const after = event.clientX > rect.left + rect.width / 2
  dragOver.value = false
  emit('drop-service', props.svc.id, after)
}
</script>

<template>
  <article
    class="card"
    :class="{
      'is-dragging': dragging,
      'is-drag-over': dragOver,
      'is-drop-after': dragOver && dropAfter,
    }"
    :style="accentStyle"
    @contextmenu.prevent="$emit('contextmenu', svc, $event)"
    @dragover="onDragOver"
    @dragleave="onDragLeave"
    @drop="onDrop"
  >
    <span class="card-accent" aria-hidden="true"></span>

    <div class="card-header">
      <div class="service-identity">
        <button
          class="drag-handle"
          type="button"
          draggable="true"
          title="Arrastrar para ordenar o mover a otro workspace"
          :aria-label="`Arrastrar ${svc.name}`"
          @click.stop
          @dragstart="onDragStart"
          @dragend="onDragEnd"
        >
          <GripVertical :size="14" />
        </button>
        <div class="service-avatar">
          <component :is="categoryIconFor(svc.category)" :size="17" />
        </div>
        <div class="service-info">
          <h3>{{ svc.name }}</h3>
          <div class="meta">
            <span class="category-badge">{{ categoryLabel }}</span>
            <span class="preset">{{ svc.preset }}</span>
            <button
              v-if="gitBranch"
              class="git-branch"
              type="button"
              :title="`Rama Git: ${gitBranch}. Abrir operaciones Git`"
              @click.stop="$emit('git', svc)"
            >
              <GitBranch :size="11" />
              {{ gitBranch }}
            </button>
          </div>
        </div>
      </div>

      <div class="card-status">
        <ServiceHealthBadge :health="health" />
        <span class="status-pill" :class="statusClass">
          <span class="status-dot"></span>
          {{ statusText }}
        </span>
      </div>
    </div>

    <div class="card-body">
      <div class="url-row">
        <Globe :size="14" class="url-icon" />
        <a v-if="serviceUrl" :href="serviceUrl" target="_blank" rel="noreferrer" class="url-link" :title="serviceUrl">
          {{ serviceUrl }}
        </a>
        <span v-else class="url-empty">Sin URL configurada</span>
        <div class="url-actions">
          <button
            class="card-icon-btn"
            type="button"
            :title="copied ? 'URL copiada' : 'Copiar URL'"
            :aria-label="copied ? 'URL copiada' : 'Copiar URL'"
            :disabled="!serviceUrl"
            @click="copyUrl"
          >
            <Check v-if="copied" :size="13" />
            <Copy v-else :size="13" />
          </button>
          <button
            class="card-icon-btn"
            type="button"
            title="Abrir en el navegador"
            aria-label="Abrir en el navegador"
            :disabled="!serviceUrl"
            @click="openInBrowser"
          >
            <ExternalLink :size="13" />
          </button>
        </div>
      </div>

      <ServiceVitals
        :cpu-percent="usage?.cpuPercent"
        :memory-bytes="usage?.memoryBytes"
        :process-count="usage?.processCount"
        :history="usageHistory"
        :running="pidAlive || status === 'running' || status === 'external'"
      />
    </div>

    <div class="card-actions">
      <button
        v-if="canStart"
        class="btn btn-success btn-icon"
        type="button"
        title="Iniciar"
        aria-label="Iniciar"
        @click="$emit('start', svc.id)"
      >
        <Play :size="14" />
      </button>
      <button
        v-else
        class="btn btn-danger btn-icon"
        type="button"
        title="Detener"
        aria-label="Detener"
        :disabled="!canStop"
        @click="$emit('stop', svc.id)"
      >
        <Square :size="14" />
      </button>

      <div class="secondary-actions">
        <button
          v-if="canRestart"
          class="btn btn-secondary btn-icon"
          type="button"
          title="Reiniciar"
          aria-label="Reiniciar"
          @click="$emit('restart', svc.id)"
        >
          <RotateCw :size="14" />
        </button>

        <button
          class="btn btn-secondary btn-icon"
          type="button"
          title="Ver logs"
          aria-label="Ver logs"
          @click="$emit('logs', svc.id)"
        >
          <ScrollText :size="14" />
        </button>

        <button
          class="btn btn-secondary btn-icon"
          type="button"
          :title="gitBranch ? 'Operaciones Git' : 'No es un repositorio Git'"
          :aria-label="gitBranch ? 'Abrir operaciones Git' : 'No es un repositorio Git'"
          :disabled="!gitBranch"
          @click="$emit('git', svc)"
        >
          <GitBranch :size="14" />
        </button>

        <button
          class="btn btn-ghost btn-icon"
          type="button"
          title="Información del servicio"
          aria-label="Información del servicio"
          @click="$emit('info', svc)"
        >
          <Info :size="14" />
        </button>

        <button
          class="btn btn-ghost btn-icon"
          type="button"
          title="Editar servicio"
          aria-label="Editar servicio"
          @click="$emit('edit', svc)"
        >
          <Pencil :size="14" />
        </button>

        <button
          class="btn btn-ghost btn-icon btn-delete"
          type="button"
          title="Eliminar servicio"
          aria-label="Eliminar servicio"
          @click="$emit('delete', svc.id)"
        >
          <Trash2 :size="14" />
        </button>
      </div>
    </div>
  </article>
</template>

<style scoped>
.card {
  position: relative;
  display: flex;
  flex-direction: column;
  min-width: 0;
  overflow: hidden;
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  background: var(--surface);
  box-shadow: var(--shadow-sm);
  transition:
    transform var(--transition),
    box-shadow var(--transition),
    border-color var(--transition);
}

.card:hover {
  border-color: var(--border-strong);
  box-shadow: var(--shadow-md);
  transform: translateY(-2px);
}

.card.is-dragging {
  opacity: 0.5;
  transform: scale(0.985);
}

.card.is-dragging:hover {
  transform: scale(0.985);
}

.card.is-drag-over {
  border-color: var(--primary);
}

.card.is-drag-over::after {
  content: '';
  position: absolute;
  top: 10px;
  bottom: 10px;
  left: 0;
  width: 3px;
  border-radius: 999px;
  background: var(--primary);
}

.card.is-drop-after::after {
  right: 0;
  left: auto;
}

.drag-handle {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 28px;
  flex-shrink: 0;
  padding: 0;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--text-subtle);
  cursor: grab;
}

.drag-handle:hover {
  background: var(--surface-2);
  color: var(--text);
}

.drag-handle:active {
  cursor: grabbing;
}

.drag-handle:focus-visible {
  outline: none;
  box-shadow: var(--focus-ring);
}

.card-accent {
  position: absolute;
  inset: 0 0 auto;
  height: 3px;
  background: var(--service-accent, var(--primary));
}

.card-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  padding: 18px 16px 14px;
}

.card-status {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.service-identity {
  display: flex;
  align-items: center;
  gap: 11px;
  min-width: 0;
}

.service-avatar {
  display: grid;
  place-items: center;
  width: 38px;
  height: 38px;
  flex-shrink: 0;
  border: 1px solid var(--border);
  border-radius: 12px;
  background: var(--surface-2);
  color: var(--service-accent, var(--primary));
}

.service-info {
  min-width: 0;
}

.service-info h3 {
  margin: 0;
  overflow: hidden;
  color: var(--text);
  font-size: 15px;
  font-weight: 700;
  letter-spacing: -0.01em;
  line-height: 1.3;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.meta {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
  margin-top: 7px;
}

.category-badge,
.preset {
  display: inline-flex;
  align-items: center;
  padding: 3px 8px;
  border-radius: 999px;
  font-size: 10px;
  font-weight: 600;
  line-height: 1.3;
  white-space: nowrap;
}

.category-badge {
  background: var(--primary-soft);
  color: var(--primary);
}

.preset {
  background: var(--surface-2);
  color: var(--text-muted);
}

.git-branch {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  max-width: 150px;
  padding: 3px 8px;
  overflow: hidden;
  border: 0;
  border-radius: 999px;
  background: var(--info-soft);
  color: var(--info);
  font-family: var(--mono);
  font-size: 10px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
  cursor: pointer;
}

.git-branch:hover,
.git-branch:focus-visible {
  outline: none;
  filter: brightness(1.08);
  box-shadow: 0 0 0 1px var(--info);
}

.card-body {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 0 16px 14px;
}

.url-row {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  padding: 9px 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
}

.url-icon {
  flex-shrink: 0;
  color: var(--text-subtle);
}

.url-link {
  min-width: 0;
  flex: 1;
  overflow: hidden;
  color: var(--primary);
  font-size: 12px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.url-empty {
  min-width: 0;
  flex: 1;
  color: var(--text-subtle);
  font-size: 12px;
}

.url-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.card-icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  flex-shrink: 0;
  padding: 0;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  color: var(--text-subtle);
  cursor: pointer;
}

.card-icon-btn:hover:not(:disabled) {
  border-color: var(--border);
  background: var(--surface);
  color: var(--text);
}

.card-icon-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.card-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  margin-top: auto;
  padding: 12px 16px 14px;
  border-top: 1px solid var(--border);
}

.secondary-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-left: auto;
}

.btn-delete {
  color: var(--danger);
}

.btn-delete:hover:not(:disabled) {
  background: var(--danger-soft);
  color: var(--danger);
}

@media (max-width: 380px) {
  .card-header {
    gap: 8px;
  }

  .service-avatar {
    display: none;
  }
}
</style>
