<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch, type Component } from 'vue'
import {
  Activity,
  Check,
  Code2,
  Copy,
  ExternalLink,
  FolderInput,
  FolderOpen,
  GitBranch,
  Globe,
  Info,
  Pencil,
  Play,
  RotateCw,
  ScrollText,
  Square,
  Terminal,
  Trash2,
} from '@lucide/vue'
import { logDashboardError } from '../composables/useDashboardLog'
import { useLogs } from '../composables/useServices'
import { useServiceItem } from '../composables/useServiceItem'
import { useServiceTools } from '../composables/useServiceTools'
import { writeToClipboard } from '../utils/clipboard'
import type { Service } from '../types'

type MenuAction =
  | 'start'
  | 'stop'
  | 'restart'
  | 'logs'
  | 'info'
  | 'move'
  | 'edit'
  | 'git'
  | 'diagnose'
  | 'copy-selection'
  | 'open-url'
  | 'copy-url'
  | 'open-folder'
  | 'copy-command'
  | 'copy-path'
  | 'open-terminal'
  | 'open-editor'
  | 'delete'

interface MenuItem {
  key: string
  label: string
  icon: Component
  action: MenuAction
  disabled?: boolean
  danger?: boolean
  separatorBefore?: boolean
}

interface Props {
  service: Service
  status: string
  pidAlive: boolean
  canMove: boolean
  selectedText?: string
  x: number
  y: number
}

interface Emits {
  (e: 'close'): void
  (e: 'start', id: string): void
  (e: 'stop', id: string): void
  (e: 'restart', id: string): void
  (e: 'logs', id: string): void
  (e: 'info', svc: Service): void
  (e: 'move', svc: Service): void
  (e: 'edit', svc: Service): void
  (e: 'git', svc: Service): void
  (e: 'diagnose', svc: Service): void
  (e: 'delete', id: string): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const menuRef = ref<HTMLElement | null>(null)
const position = ref({ left: props.x, top: props.y })
const copiedKey = ref<string | null>(null)
let copyTimer: number | undefined
let contextMenuListenerTimer: number | undefined

const { canStart, canStop, canRestart, statusText, serviceUrl, openInBrowser } = useServiceItem({
  service: () => props.service,
  status: () => props.status,
  pidAlive: () => props.pidAlive,
})
const { openServiceFolder } = useLogs()
const { openTerminal, openEditor } = useServiceTools()

const menuStyle = computed(() => ({
  left: `${position.value.left}px`,
  top: `${position.value.top}px`,
  '--service-accent': props.service.color || 'var(--primary)',
}))

const items = computed<MenuItem[]>(() => {
  const result: MenuItem[] = []

  if (props.selectedText) {
    result.push({
      key: 'copy-selection',
      label: 'Copiar selección',
      icon: Copy,
      action: 'copy-selection',
    })
  }

  const separatorBeforeServiceActions = result.length > 0

  if (canStart.value) {
    result.push({
      key: 'start',
      label: 'Iniciar',
      icon: Play,
      action: 'start',
      separatorBefore: separatorBeforeServiceActions,
    })
  } else {
    result.push({
      key: 'stop',
      label: 'Detener',
      icon: Square,
      action: 'stop',
      disabled: !canStop.value,
      separatorBefore: separatorBeforeServiceActions,
    })
  }

  if (canRestart.value) {
    result.push({ key: 'restart', label: 'Reiniciar', icon: RotateCw, action: 'restart' })
  }

  result.push({ key: 'logs', label: 'Ver logs', icon: ScrollText, action: 'logs' })

  result.push(
    { key: 'info', label: 'Información', icon: Info, action: 'info', separatorBefore: true },
    {
      key: 'move',
      label: 'Mover a workspace',
      icon: FolderInput,
      action: 'move',
      disabled: !props.canMove,
    },
    { key: 'edit', label: 'Editar', icon: Pencil, action: 'edit' },
    {
      key: 'git',
      label: 'Operaciones Git',
      icon: GitBranch,
      action: 'git',
      disabled: !props.service.cwd,
    },
  )

  result.push(
    {
      key: 'diagnose',
      label: 'Diagnosticar',
      icon: Activity,
      action: 'diagnose',
      disabled: !props.service.cwd,
      separatorBefore: true,
    },
    {
      key: 'terminal',
      label: 'Abrir terminal',
      icon: Terminal,
      action: 'open-terminal',
      disabled: !props.service.cwd,
    },
    {
      key: 'editor',
      label: 'Abrir en VS Code',
      icon: Code2,
      action: 'open-editor',
      disabled: !props.service.cwd,
    },
  )

  result.push(
    {
      key: 'open-url',
      label: 'Abrir URL',
      icon: Globe,
      action: 'open-url',
      disabled: !serviceUrl.value,
      separatorBefore: true,
    },
    {
      key: 'copy-url',
      label: 'Copiar URL',
      icon: Copy,
      action: 'copy-url',
      disabled: !serviceUrl.value,
    },
    {
      key: 'open-folder',
      label: 'Abrir carpeta',
      icon: FolderOpen,
      action: 'open-folder',
      disabled: !props.service.cwd,
    },
    {
      key: 'copy-command',
      label: 'Copiar comando',
      icon: Copy,
      action: 'copy-command',
      disabled: !props.service.command,
    },
    {
      key: 'copy-path',
      label: 'Copiar ruta',
      icon: Copy,
      action: 'copy-path',
      disabled: !props.service.cwd,
    },
  )

  result.push({
    key: 'delete',
    label: 'Eliminar',
    icon: Trash2,
    action: 'delete',
    danger: true,
    separatorBefore: true,
  })

  return result
})

const command = computed(() =>
  [props.service.command, ...(props.service.args ?? [])].filter(Boolean).join(' '),
)

function close() {
  emit('close')
}

function focusFirstItem() {
  const first = menuRef.value?.querySelector<HTMLButtonElement>(
    '[role="menuitem"]:not(:disabled)',
  )
  first?.focus()
}

function clampPosition() {
  const element = menuRef.value
  if (!element) return
  const margin = 8
  const rect = element.getBoundingClientRect()
  const maxLeft = Math.max(margin, window.innerWidth - rect.width - margin)
  const maxTop = Math.max(margin, window.innerHeight - rect.height - margin)
  position.value = {
    left: Math.min(Math.max(margin, props.x), maxLeft),
    top: Math.min(Math.max(margin, props.y), maxTop),
  }
}

async function copyValue(key: string, text: string) {
  try {
    await writeToClipboard(text)
    copiedKey.value = key
    if (copyTimer !== undefined) window.clearTimeout(copyTimer)
    copyTimer = window.setTimeout(() => {
      copiedKey.value = null
      close()
    }, 650)
  } catch (error) {
    void logDashboardError('contextMenu.copy', error, { notify: true })
    close()
  }
}

async function runAction(action: MenuAction) {
  switch (action) {
    case 'start':
      emit('start', props.service.id)
      close()
      break
    case 'stop':
      emit('stop', props.service.id)
      close()
      break
    case 'restart':
      emit('restart', props.service.id)
      close()
      break
    case 'logs':
      emit('logs', props.service.id)
      close()
      break
    case 'info':
      emit('info', props.service)
      close()
      break
    case 'move':
      emit('move', props.service)
      close()
      break
    case 'edit':
      emit('edit', props.service)
      close()
      break
    case 'git':
      emit('git', props.service)
      close()
      break
    case 'diagnose':
      emit('diagnose', props.service)
      close()
      break
    case 'open-url':
      try {
        await openInBrowser()
      } catch (error) {
        void logDashboardError('contextMenu.openUrl', error, { notify: true })
      }
      close()
      break
    case 'copy-selection':
      await copyValue('copy-selection', props.selectedText ?? '')
      break
    case 'copy-url':
      await copyValue('copy-url', serviceUrl.value)
      break
    case 'open-folder':
      try {
        await openServiceFolder(props.service.cwd)
      } catch (error) {
        void logDashboardError('contextMenu.openFolder', error, { notify: true })
      }
      close()
      break
    case 'copy-command':
      await copyValue('copy-command', command.value)
      break
    case 'copy-path':
      await copyValue('copy-path', props.service.cwd)
      break
    case 'open-terminal':
      try {
        await openTerminal(props.service.cwd)
      } catch (error) {
        void logDashboardError('contextMenu.openTerminal', error, {
          title: 'No se pudo abrir la terminal',
          notify: true,
        })
      }
      close()
      break
    case 'open-editor':
      try {
        await openEditor(props.service.cwd)
      } catch (error) {
        void logDashboardError('contextMenu.openEditor', error, {
          title: 'No se pudo abrir VS Code',
          notify: true,
        })
      }
      close()
      break
    case 'delete':
      emit('delete', props.service.id)
      close()
      break
  }
}

function isCopied(key: string): boolean {
  return copiedKey.value === key
}

function handleWindowMouseDown(event: MouseEvent) {
  if (!menuRef.value?.contains(event.target as Node)) close()
}

function handleWindowContextMenu(event: MouseEvent) {
  if (event.defaultPrevented) return
  if (!menuRef.value?.contains(event.target as Node)) close()
}

function handleWindowScroll(event: Event) {
  if (!menuRef.value?.contains(event.target as Node)) close()
}

watch(
  () => [props.x, props.y],
  async () => {
    position.value = { left: props.x, top: props.y }
    await nextTick()
    clampPosition()
  },
)

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    event.preventDefault()
    close()
    return
  }

  if (!['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) return
  event.preventDefault()

  const focusable = Array.from(
    menuRef.value?.querySelectorAll<HTMLButtonElement>(
      '[role="menuitem"]:not(:disabled)',
    ) ?? [],
  )
  if (focusable.length === 0) return

  const currentIndex = focusable.indexOf(document.activeElement as HTMLButtonElement)
  let nextIndex = 0
  if (event.key === 'ArrowDown') nextIndex = (currentIndex + 1) % focusable.length
  if (event.key === 'ArrowUp') nextIndex = (currentIndex - 1 + focusable.length) % focusable.length
  if (event.key === 'End') nextIndex = focusable.length - 1
  focusable[nextIndex]?.focus()
}

onMounted(async () => {
  await nextTick()
  clampPosition()
  focusFirstItem()
  window.addEventListener('mousedown', handleWindowMouseDown)
  contextMenuListenerTimer = window.setTimeout(() => {
    window.addEventListener('contextmenu', handleWindowContextMenu)
  }, 0)
  window.addEventListener('resize', clampPosition)
  window.addEventListener('scroll', handleWindowScroll, true)
  window.addEventListener('keydown', handleKeydown)
  window.addEventListener('blur', close)
})

onBeforeUnmount(() => {
  if (copyTimer !== undefined) window.clearTimeout(copyTimer)
  if (contextMenuListenerTimer !== undefined) window.clearTimeout(contextMenuListenerTimer)
  window.removeEventListener('mousedown', handleWindowMouseDown)
  window.removeEventListener('contextmenu', handleWindowContextMenu)
  window.removeEventListener('resize', clampPosition)
  window.removeEventListener('scroll', handleWindowScroll, true)
  window.removeEventListener('keydown', handleKeydown)
  window.removeEventListener('blur', close)
})
</script>

<template>
  <Teleport to="body">
    <div
      ref="menuRef"
      class="context-menu"
      :style="menuStyle"
      role="menu"
      aria-label="Acciones del servicio"
      @contextmenu.prevent
    >
      <div class="context-header">
        <span class="context-dot"></span>
        <div>
          <strong>{{ service.name }}</strong>
          <span>{{ statusText }}</span>
        </div>
      </div>

      <div class="context-items">
        <template v-for="item in items" :key="item.key">
          <div v-if="item.separatorBefore" class="context-separator"></div>
          <button
            class="context-item"
            :class="{ 'is-danger': item.danger, 'is-copied': isCopied(item.key) }"
            type="button"
            role="menuitem"
            :disabled="item.disabled"
            @click="runAction(item.action)"
          >
            <Check v-if="isCopied(item.key)" :size="14" class="context-icon" />
            <component :is="item.icon" v-else :size="14" class="context-icon" />
            <span>{{ isCopied(item.key) ? 'Copiado' : item.label }}</span>
            <ExternalLink
              v-if="item.action === 'open-url'"
              :size="11"
              class="context-external"
            />
          </button>
        </template>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.context-menu {
  position: fixed;
  z-index: 1300;
  width: 248px;
  max-height: min(70vh, 560px);
  overflow-y: auto;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface);
  box-shadow: var(--shadow-lg);
  animation: menuIn 120ms ease-out;
}

.context-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 11px;
  border-bottom: 1px solid var(--border);
  background: var(--surface-2);
}

.context-dot {
  width: 8px;
  height: 8px;
  flex-shrink: 0;
  border-radius: 999px;
  background: var(--service-accent, var(--primary));
}

.context-header div {
  display: flex;
  flex-direction: column;
  min-width: 0;
  gap: 1px;
}

.context-header strong {
  overflow: hidden;
  color: var(--text);
  font-size: 11px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.context-header span {
  color: var(--text-subtle);
  font-size: 9px;
}

.context-items {
  padding: 5px;
}

.context-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 7px 8px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--text-muted);
  font-size: 11px;
  text-align: left;
  cursor: pointer;
}

.context-item:hover:not(:disabled),
.context-item:focus-visible {
  outline: none;
  background: var(--surface-2);
  color: var(--text);
}

.context-item:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.context-item.is-danger {
  color: var(--danger);
}

.context-item.is-danger:hover:not(:disabled),
.context-item.is-danger:focus-visible {
  background: var(--danger-soft);
  color: var(--danger);
}

.context-item.is-copied {
  color: var(--success);
}

.context-icon {
  flex-shrink: 0;
}

.context-external {
  margin-left: auto;
  color: var(--text-subtle);
}

.context-separator {
  height: 1px;
  margin: 5px 6px;
  background: var(--border);
}

@keyframes menuIn {
  from {
    opacity: 0;
    transform: scale(0.98) translateY(-2px);
  }
  to {
    opacity: 1;
    transform: none;
  }
}
</style>
