<script setup lang="ts">
import { ref } from 'vue'
import {
  Check,
  Copy,
  ExternalLink,
  Folder,
  FolderInput,
  GitBranch,
  Globe,
  GripVertical,
  Info,
  Pencil,
  Play,
  RotateCw,
  ScrollText,
  Square,
  Terminal,
  Trash2,
} from '@lucide/vue'
import { useServiceItem } from '../composables/useServiceItem'
import { categoryIconFor } from '../utils/serviceIcons'
import { isServiceDrag, setServiceDragData } from '../utils/serviceDrag'
import ServiceHealthBadge from './ServiceHealthBadge.vue'
import type { Service, ServiceHealth } from '../types'

interface Props {
  svc: Service
  status: string
  pidAlive: boolean
  gitBranch?: string
  health?: ServiceHealth
  statusClass: string
  categoryLabel: string
  dragging?: boolean
}

interface Emits {
  (e: 'start', id: string): void
  (e: 'stop', id: string): void
  (e: 'restart', id: string): void
  (e: 'logs', id: string): void
  (e: 'info', svc: Service): void
  (e: 'move', svc: Service): void
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

function formatPath(path: string): string {
  if (!path) return 'Sin carpeta'
  if (path.length <= 56) return path
  return '…' + path.slice(-53)
}

function onDragStart(event: DragEvent) {
  setServiceDragData(event, props.svc.id)
  const row = (event.currentTarget as HTMLElement).closest('.service-row')
  if (row) event.dataTransfer?.setDragImage(row, 20, 20)
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
  dropAfter.value = event.clientY > rect.top + rect.height / 2
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
  const after = event.clientY > rect.top + rect.height / 2
  dragOver.value = false
  emit('drop-service', props.svc.id, after)
}
</script>

<template>
  <article
    class="service-row"
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
    <span class="row-accent" aria-hidden="true"></span>

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

    <div class="row-main">
      <div class="row-title">
        <h3>{{ svc.name }}</h3>
        <span class="category-badge">
          <component :is="categoryIconFor(svc.category)" :size="11" />
          {{ categoryLabel }}
        </span>
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
        <ServiceHealthBadge :health="health" />
        <span class="status-pill" :class="statusClass">
          <span class="status-dot"></span>
          {{ statusText }}
        </span>
      </div>

      <div class="row-details">
        <span class="detail" :title="svc.cwd">
          <Folder :size="12" />
          <span class="mono">{{ formatPath(svc.cwd) }}</span>
        </span>
        <span class="detail" :title="`${svc.command} ${svc.args?.join(' ')}`">
          <Terminal :size="12" />
          <code class="mono">{{ svc.command }} {{ svc.args?.join(' ') }}</code>
        </span>
      </div>

      <div class="row-url">
        <Globe :size="12" />
        <a v-if="serviceUrl" :href="serviceUrl" target="_blank" rel="noreferrer" :title="serviceUrl">
          {{ serviceUrl }}
        </a>
        <span v-else class="url-empty">Sin URL</span>
        <button
          class="row-icon-btn"
          type="button"
          :title="copied ? 'URL copiada' : 'Copiar URL'"
          :aria-label="copied ? 'URL copiada' : 'Copiar URL'"
          @click="copyUrl"
        >
          <Check v-if="copied" :size="12" />
          <Copy v-else :size="12" />
        </button>
        <button
          class="row-icon-btn"
          type="button"
          title="Abrir en navegador"
          aria-label="Abrir en navegador"
          @click="openInBrowser"
        >
          <ExternalLink :size="12" />
        </button>
      </div>
    </div>

    <div class="row-actions">
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
        title="Mover a otro workspace"
        aria-label="Mover a otro workspace"
        @click="$emit('move', svc)"
      >
        <FolderInput :size="14" />
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
        class="btn btn-ghost btn-icon row-delete"
        type="button"
        title="Eliminar servicio"
        aria-label="Eliminar servicio"
        @click="$emit('delete', svc.id)"
      >
        <Trash2 :size="14" />
      </button>
    </div>
  </article>
</template>

<style scoped>
.service-row {
  position: relative;
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  gap: 12px;
  align-items: center;
  min-width: 0;
  padding: 12px 14px 12px 18px;
  overflow: hidden;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface);
  box-shadow: var(--shadow-sm);
  transition:
    border-color var(--transition),
    box-shadow var(--transition),
    background var(--transition);
}

.service-row:hover {
  border-color: var(--border-strong);
  background: var(--surface-2);
  box-shadow: var(--shadow-md);
}

.service-row.is-dragging {
  opacity: 0.5;
}

.service-row.is-drag-over {
  border-color: var(--primary);
}

.service-row.is-drag-over::after {
  content: '';
  position: absolute;
  top: 0;
  right: 10px;
  left: 10px;
  height: 3px;
  border-radius: 999px;
  background: var(--primary);
}

.service-row.is-drop-after::after {
  top: auto;
  bottom: 0;
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

.row-accent {
  position: absolute;
  top: 10px;
  bottom: 10px;
  left: 0;
  width: 3px;
  border-radius: 999px;
  background: var(--service-accent, var(--primary));
}

.row-main {
  display: flex;
  flex-direction: column;
  min-width: 0;
  gap: 6px;
}

.row-title {
  display: flex;
  align-items: center;
  gap: 7px;
  flex-wrap: wrap;
  min-width: 0;
}

.row-title h3 {
  margin: 0;
  overflow: hidden;
  color: var(--text);
  font-size: 14px;
  font-weight: 700;
  letter-spacing: -0.01em;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.category-badge,
.preset {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 7px;
  border-radius: 999px;
  font-size: 10px;
  font-weight: 600;
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
  padding: 2px 7px;
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

.row-details {
  display: flex;
  align-items: center;
  gap: 14px;
  flex-wrap: wrap;
  min-width: 0;
}

.detail {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  min-width: 0;
  color: var(--text-muted);
  font-size: 11px;
}

.detail > svg {
  flex-shrink: 0;
  color: var(--text-subtle);
}

.mono {
  overflow: hidden;
  font-family: var(--mono);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.detail code {
  padding: 0;
  border: 0;
  background: transparent;
  font-size: inherit;
}

.row-url {
  display: flex;
  align-items: center;
  gap: 5px;
  min-width: 0;
  color: var(--text-subtle);
}

.row-url > svg {
  flex-shrink: 0;
}

.row-url a {
  overflow: hidden;
  color: var(--primary);
  font-size: 11px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.url-empty {
  color: var(--text-subtle);
  font-size: 11px;
}

.row-icon-btn {
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

.row-icon-btn:hover {
  border-color: var(--border);
  background: var(--surface);
  color: var(--text);
}

.row-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 4px;
  flex-wrap: wrap;
}

.row-delete {
  color: var(--danger);
}

.row-delete:hover:not(:disabled) {
  background: var(--danger-soft);
  color: var(--danger);
}

@media (max-width: 900px) {
  .service-row {
    grid-template-columns: auto minmax(0, 1fr);
    gap: 10px;
  }

  .row-actions {
    grid-column: 1 / -1;
    justify-content: flex-start;
    padding-top: 8px;
    border-top: 1px solid var(--border);
  }
}

@media (max-width: 600px) {
  .row-details {
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
  }
}
</style>
