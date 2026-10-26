<script setup lang="ts">
import { AlertTriangle, PanelLeftClose, PanelLeftOpen, Plus, Settings2 } from '@lucide/vue'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { isServiceDrag } from '../utils/serviceDrag'
import { formatBytes, formatCpuPercent } from '../utils/metricsFormat'
import WorkspaceUsageMeter from './WorkspaceUsageMeter.vue'
import type { Workspace, WorkspaceStats, WorkspaceUsage } from '../types'

interface Props {
  workspaces: Workspace[]
  activeWorkspaceId: string
  stats: Record<string, WorkspaceStats>
  usage: Record<string, WorkspaceUsage>
  dragging?: boolean
}

interface Emits {
  (e: 'switch', id: string): void
  (e: 'manage'): void
  (e: 'drop-service', workspaceId: string): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const STORAGE_KEY_COLLAPSED = 'cardumen-deck-sidebar-collapsed-v1'
const STORAGE_KEY_WIDTH = 'cardumen-deck-sidebar-width'
const DEFAULT_WIDTH = 240
const MIN_WIDTH = 180
const MAX_WIDTH = 420
const COLLAPSED_WIDTH = 72

const viewportWidth = ref(window.innerWidth)
const maxWidth = computed(() =>
  Math.max(MIN_WIDTH, Math.min(MAX_WIDTH, Math.floor(viewportWidth.value * 0.45))),
)

function clampWidth(value: number): number {
  return Math.min(maxWidth.value, Math.max(MIN_WIDTH, Math.round(value)))
}

function readStoredWidth(): number {
  const stored = Number(localStorage.getItem(STORAGE_KEY_WIDTH))
  return Number.isFinite(stored) && stored > 0 ? clampWidth(stored) : DEFAULT_WIDTH
}

const collapsed = ref(localStorage.getItem(STORAGE_KEY_COLLAPSED) === '1')
const width = ref(readStoredWidth())
const resizing = ref(false)
let stopResize: (() => void) | undefined

const sidebarStyle = computed(() => ({
  width: `${collapsed.value ? COLLAPSED_WIDTH : width.value}px`,
}))

function persistWidth() {
  localStorage.setItem(STORAGE_KEY_WIDTH, String(width.value))
}

function toggleCollapsed() {
  collapsed.value = !collapsed.value
  localStorage.setItem(STORAGE_KEY_COLLAPSED, collapsed.value ? '1' : '0')
}

function startResize(event: PointerEvent) {
  if (collapsed.value || resizing.value) return

  event.preventDefault()
  event.stopPropagation()

  const handle = event.currentTarget as HTMLElement
  const startX = event.clientX
  const startWidth = width.value

  handle.focus({ preventScroll: true })
  resizing.value = true
  document.body.classList.add('sidebar-resizing')

  const onMove = (moveEvent: PointerEvent) => {
    width.value = clampWidth(startWidth + moveEvent.clientX - startX)
  }

  const finishResize = () => {
    window.removeEventListener('pointermove', onMove)
    window.removeEventListener('pointerup', finishResize)
    window.removeEventListener('pointercancel', finishResize)
    window.removeEventListener('blur', finishResize)

    stopResize = undefined
    resizing.value = false
    document.body.classList.remove('sidebar-resizing')
    persistWidth()
  }

  stopResize = finishResize
  window.addEventListener('pointermove', onMove)
  window.addEventListener('pointerup', finishResize)
  window.addEventListener('pointercancel', finishResize)
  window.addEventListener('blur', finishResize)
}

function resetWidth() {
  width.value = clampWidth(DEFAULT_WIDTH)
  persistWidth()
}

function onResizeKeydown(event: KeyboardEvent) {
  if (collapsed.value) return

  const step = event.shiftKey ? 32 : 8
  let nextWidth = width.value

  if (event.key === 'ArrowLeft') nextWidth -= step
  else if (event.key === 'ArrowRight') nextWidth += step
  else if (event.key === 'Home') nextWidth = MIN_WIDTH
  else if (event.key === 'End') nextWidth = maxWidth.value
  else return

  event.preventDefault()
  width.value = clampWidth(nextWidth)
  persistWidth()
}

function onWindowResize() {
  viewportWidth.value = window.innerWidth
  const clamped = clampWidth(width.value)
  if (clamped !== width.value) {
    width.value = clamped
    persistWidth()
  }
}

onMounted(() => {
  window.addEventListener('resize', onWindowResize)
})

onBeforeUnmount(() => {
  window.removeEventListener('resize', onWindowResize)
  stopResize?.()
  document.body.classList.remove('sidebar-resizing')
})

function statsFor(id: string): WorkspaceStats {
  return props.stats[id] ?? { total: 0, running: 0, errors: 0 }
}

function usageFor(id: string): WorkspaceUsage {
  return props.usage[id] ?? { cpuPercent: 0, memoryBytes: 0, processCount: 0 }
}

function workspaceTitle(workspace: Workspace): string {
  const stats = statsFor(workspace.id)
  const usage = usageFor(workspace.id)
  return `${workspace.name} · ${stats.total} servicios · ${stats.running} activos · CPU ${formatCpuPercent(usage.cpuPercent)} · RAM ${formatBytes(usage.memoryBytes)}`
}

function initial(name: string): string {
  return name.trim().charAt(0).toUpperCase() || 'W'
}

function switchWorkspace(id: string) {
  if (id !== props.activeWorkspaceId) emit('switch', id)
}

const dragOverWorkspaceId = ref<string | null>(null)

watch(
  () => props.dragging,
  (dragging) => {
    if (!dragging) dragOverWorkspaceId.value = null
  },
)

function canDropInWorkspace(id: string): boolean {
  return Boolean(props.dragging) && id !== props.activeWorkspaceId
}

function onWorkspaceDragOver(id: string, event: DragEvent) {
  if (!canDropInWorkspace(id) || !isServiceDrag(event)) return
  event.preventDefault()
  if (event.dataTransfer) event.dataTransfer.dropEffect = 'move'
  dragOverWorkspaceId.value = id
}

function onWorkspaceDragLeave(id: string, event: DragEvent) {
  if (dragOverWorkspaceId.value !== id) return
  const current = event.currentTarget as HTMLElement
  const next = event.relatedTarget as Node | null
  if (!next || !current.contains(next)) {
    dragOverWorkspaceId.value = null
  }
}

function onWorkspaceDrop(id: string, event: DragEvent) {
  if (!canDropInWorkspace(id) || !isServiceDrag(event)) return
  event.preventDefault()
  dragOverWorkspaceId.value = null
  emit('drop-service', id)
}
</script>

<template>
  <aside class="sidebar" :class="{ collapsed, resizing }" :style="sidebarStyle">
    <div
      v-if="!collapsed"
      class="sidebar-resizer"
      role="separator"
      aria-orientation="vertical"
      aria-label="Redimensionar ancho del sidebar"
      :aria-valuemin="MIN_WIDTH"
      :aria-valuemax="maxWidth"
      :aria-valuenow="width"
      tabindex="0"
      @pointerdown="startResize"
      @dblclick="resetWidth"
      @keydown="onResizeKeydown"
    >
      <span class="resizer-grip"></span>
    </div>

    <div class="sidebar-top">
      <div v-if="!collapsed" class="sidebar-heading">
        <span>Workspaces</span>
        <span class="workspace-total">{{ workspaces.length }}</span>
      </div>
      <button
        class="icon-btn collapse-btn"
        type="button"
        :aria-label="collapsed ? 'Expandir sidebar' : 'Colapsar sidebar'"
        :title="collapsed ? 'Expandir sidebar' : 'Colapsar sidebar'"
        @click="toggleCollapsed"
      >
        <PanelLeftOpen v-if="collapsed" :size="17" />
        <PanelLeftClose v-else :size="17" />
      </button>
    </div>

    <nav class="workspace-nav" aria-label="Workspaces">
      <button
        v-for="workspace in workspaces"
        :key="workspace.id"
        class="workspace-item"
        :class="{
          active: workspace.id === activeWorkspaceId,
          'can-drop': canDropInWorkspace(workspace.id),
          'is-drop-target': dragOverWorkspaceId === workspace.id,
        }"
        type="button"
        :title="workspaceTitle(workspace)"
        :aria-current="workspace.id === activeWorkspaceId ? 'page' : undefined"
        @click="switchWorkspace(workspace.id)"
        @dragover="onWorkspaceDragOver(workspace.id, $event)"
        @dragleave="onWorkspaceDragLeave(workspace.id, $event)"
        @drop="onWorkspaceDrop(workspace.id, $event)"
      >
        <template v-if="collapsed">
          <span class="workspace-avatar" :style="{ background: workspace.color || 'var(--primary)' }">
            {{ initial(workspace.name) }}
            <span
              v-if="statsFor(workspace.id).errors > 0"
              class="avatar-dot error"
              :title="`${statsFor(workspace.id).errors} error(es)`"
            ></span>
            <span
              v-else-if="statsFor(workspace.id).running > 0"
              class="avatar-dot running"
              :title="`${statsFor(workspace.id).running} activo(s)`"
            ></span>
          </span>
        </template>

        <template v-else>
          <span
            class="workspace-dot"
            :style="{ background: workspace.color || 'var(--primary)' }"
          ></span>
          <span class="workspace-content">
            <span class="workspace-name">{{ workspace.name }}</span>
            <span class="workspace-meta">
              {{
                dragOverWorkspaceId === workspace.id
                  ? 'Soltar aquí'
                  : `${statsFor(workspace.id).total} servicios`
              }}
            </span>
            <WorkspaceUsageMeter
              :cpu-percent="usageFor(workspace.id).cpuPercent"
              :memory-bytes="usageFor(workspace.id).memoryBytes"
              :process-count="usageFor(workspace.id).processCount"
              :label="workspace.name"
            />
          </span>
          <span class="workspace-indicators">
            <span v-if="statsFor(workspace.id).running > 0" class="indicator running">
              {{ statsFor(workspace.id).running }}
            </span>
            <span v-if="statsFor(workspace.id).errors > 0" class="indicator errors">
              <AlertTriangle :size="11" />
              {{ statsFor(workspace.id).errors }}
            </span>
          </span>
        </template>
      </button>
    </nav>

    <div class="sidebar-footer">
      <button class="sidebar-action" type="button" :title="collapsed ? 'Nuevo workspace' : undefined" @click="$emit('manage')">
        <Plus :size="16" />
        <span v-if="!collapsed">Nuevo workspace</span>
      </button>
      <button class="sidebar-action" type="button" title="Gestionar workspaces" @click="$emit('manage')">
        <Settings2 :size="16" />
        <span v-if="!collapsed">Gestionar workspaces</span>
      </button>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  position: relative;
  display: flex;
  flex-direction: column;
  width: 240px;
  flex-shrink: 0;
  min-height: 0;
  border-right: 1px solid var(--border);
  background: var(--surface-glass);
  backdrop-filter: blur(14px);
  transition: width var(--transition);
}

.sidebar.resizing {
  transition: none;
}

.sidebar.collapsed {
  width: 72px;
}

.sidebar-resizer {
  position: absolute;
  top: 0;
  right: -3px;
  bottom: 0;
  z-index: 10;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 7px;
  background: transparent;
  cursor: col-resize;
  touch-action: none;
}

.resizer-grip {
  width: 2px;
  height: 38px;
  border-radius: 999px;
  background: transparent;
  transition:
    background var(--transition),
    height var(--transition);
}

.sidebar-resizer:hover .resizer-grip,
.sidebar.resizing .resizer-grip,
.sidebar-resizer:focus-visible .resizer-grip {
  height: 56px;
  background: var(--primary);
}

.sidebar-resizer:focus-visible {
  outline: none;
  border-radius: 999px;
  box-shadow: inset 0 0 0 2px var(--primary);
}

.sidebar-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 14px 12px 10px;
}

.sidebar-heading {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--text-subtle);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.workspace-total {
  min-width: 17px;
  padding: 1px 5px;
  border-radius: 999px;
  background: var(--surface-2);
  color: var(--text-subtle);
  font-size: 9px;
  text-align: center;
}

.collapse-btn {
  flex-shrink: 0;
}

.collapsed .sidebar-top {
  justify-content: center;
  padding-inline: 8px;
}

.workspace-nav {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 4px;
  min-height: 0;
  padding: 4px 8px;
  overflow: auto;
}

.workspace-item {
  position: relative;
  display: flex;
  align-items: center;
  gap: 9px;
  width: 100%;
  min-height: 58px;
  padding: 8px 9px;
  border: 0;
  border-radius: 10px;
  background: transparent;
  color: var(--text);
  text-align: left;
  cursor: pointer;
  transition:
    background var(--transition),
    color var(--transition);
}

.workspace-item:hover {
  background: var(--surface-2);
}

.workspace-item.active {
  background: var(--primary-soft);
}

.workspace-item.can-drop {
  box-shadow: inset 0 0 0 1px var(--border-strong);
}

.workspace-item.can-drop:hover {
  box-shadow: inset 0 0 0 1px var(--primary);
}

.workspace-item.is-drop-target {
  background: var(--primary-soft);
  box-shadow: inset 0 0 0 1px var(--primary);
}

.workspace-item.is-drop-target .workspace-name,
.workspace-item.is-drop-target .workspace-meta {
  color: var(--primary);
}

.workspace-item.active::before {
  content: '';
  position: absolute;
  top: 8px;
  bottom: 8px;
  left: 0;
  width: 3px;
  border-radius: 999px;
  background: var(--primary);
}

.workspace-dot {
  width: 8px;
  height: 8px;
  flex-shrink: 0;
  border-radius: 50%;
}

.workspace-content {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-width: 0;
  gap: 1px;
}

.workspace-name {
  overflow: hidden;
  color: var(--text);
  font-size: 12px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.workspace-item.active .workspace-name {
  color: var(--primary);
}

.workspace-meta {
  color: var(--text-subtle);
  font-size: 10px;
}

.workspace-indicators {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.indicator {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  padding: 1px 5px;
  border-radius: 999px;
  font-size: 9px;
  font-weight: 700;
}

.indicator.running {
  background: var(--success-soft);
  color: var(--success);
}

.indicator.errors {
  background: var(--danger-soft);
  color: var(--danger);
}

.collapsed .workspace-nav {
  align-items: center;
  padding-inline: 8px;
}

.collapsed .workspace-item {
  justify-content: center;
  width: 44px;
  min-height: 44px;
  padding: 5px;
}

.collapsed .workspace-item.active::before {
  top: 10px;
  bottom: 10px;
  left: -6px;
}

.workspace-avatar {
  position: relative;
  display: grid;
  place-items: center;
  width: 34px;
  height: 34px;
  border-radius: 10px;
  color: #fff;
  font-size: 12px;
  font-weight: 700;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.25);
}

.avatar-dot {
  position: absolute;
  top: -2px;
  right: -2px;
  width: 9px;
  height: 9px;
  border: 2px solid var(--surface);
  border-radius: 50%;
}

.avatar-dot.running {
  background: var(--success);
}

.avatar-dot.error {
  background: var(--danger);
}

.sidebar-footer {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 10px 8px;
  border-top: 1px solid var(--border);
}

.sidebar-action {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  min-height: 34px;
  padding: 7px 9px;
  border: 0;
  border-radius: 8px;
  background: transparent;
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 600;
  text-align: left;
  cursor: pointer;
  transition:
    background var(--transition),
    color var(--transition);
}

.sidebar-action:hover {
  background: var(--surface-2);
  color: var(--text);
}

.collapsed .sidebar-action {
  justify-content: center;
  padding-inline: 5px;
}
</style>
