<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch, type Component } from 'vue'
import { ClipboardPaste, Copy, Scissors, TextCursorInput } from '@lucide/vue'
import { logDashboardError } from '../composables/useDashboardLog'
import {
  captureTextContext,
  copyTextContext,
  cutTextContext,
  pasteTextContext,
  selectAllTextContext,
  type TextContextAction,
} from '../utils/textContext'

interface Props {
  x: number
  y: number
  target: HTMLElement | null
}

interface Emits {
  (e: 'close'): void
}

interface MenuAction {
  key: TextContextAction
  label: string
  icon: Component
  disabled?: boolean
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const menuRef = ref<HTMLElement | null>(null)
const position = ref({ left: props.x, top: props.y })
const context = ref(captureTextContext(props.target))
let contextMenuListenerTimer: number | undefined

const actions = computed<MenuAction[]>(() => {
  const current = context.value
  if (current.editable) {
    return [
      { key: 'copy', label: 'Copiar', icon: Copy, disabled: !current.canCopy },
      { key: 'cut', label: 'Cortar', icon: Scissors, disabled: !current.canCut },
      { key: 'paste', label: 'Pegar', icon: ClipboardPaste, disabled: !current.canPaste },
      {
        key: 'select-all',
        label: 'Seleccionar todo',
        icon: TextCursorInput,
        disabled: !current.canSelectAll,
      },
    ]
  }

  if (current.canCopy) {
    return [{ key: 'copy', label: 'Copiar', icon: Copy }]
  }

  return []
})

const hasActions = computed(() => actions.value.length > 0)

function close() {
  emit('close')
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

watch(
  () => [props.x, props.y, props.target] as const,
  async () => {
    context.value = captureTextContext(props.target)
    position.value = { left: props.x, top: props.y }
    await nextTick()
    clampPosition()
  },
)

async function runAction(action: TextContextAction) {
  try {
    if (action === 'copy') await copyTextContext(context.value)
    else if (action === 'cut') await cutTextContext(context.value)
    else if (action === 'paste') await pasteTextContext(context.value)
    else selectAllTextContext(context.value)
  } catch (error) {
    void logDashboardError(`contextMenu.${action}`, error, {
      title:
        action === 'paste'
          ? 'No se pudo pegar desde el portapapeles'
          : 'No se pudo acceder al portapapeles',
      notify: true,
    })
  } finally {
    close()
  }
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
  if (event.key === 'ArrowUp') {
    nextIndex =
      currentIndex < 0
        ? focusable.length - 1
        : (currentIndex - 1 + focusable.length) % focusable.length
  }
  if (event.key === 'End') nextIndex = focusable.length - 1
  focusable[nextIndex]?.focus()
}

onMounted(async () => {
  await nextTick()
  clampPosition()
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
      class="global-context-menu"
      :style="{ left: `${position.left}px`, top: `${position.top}px` }"
      role="menu"
      aria-label="Acciones contextuales"
      @contextmenu.prevent
      @mousedown.stop
    >
      <div v-if="hasActions" class="context-actions">
        <button
          v-for="action in actions"
          :key="action.key"
          class="context-action"
          type="button"
          role="menuitem"
          :disabled="action.disabled"
          @mousedown.prevent
          @click="runAction(action.key)"
        >
          <component :is="action.icon" :size="14" />
          <span>{{ action.label }}</span>
        </button>
      </div>

      <div v-if="hasActions" class="context-separator"></div>

      <div class="menu-brand">
        <strong>Cardumen Deck</strong>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.global-context-menu {
  position: fixed;
  z-index: 1300;
  min-width: 190px;
  max-width: 240px;
  max-height: min(70vh, 420px);
  padding: 5px;
  overflow-y: auto;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface);
  box-shadow: var(--shadow-lg);
  animation: globalMenuIn 120ms ease-out;
  user-select: none;
}

.context-actions {
  display: flex;
  flex-direction: column;
}

.context-action {
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
  white-space: nowrap;
}

.context-action:hover:not(:disabled),
.context-action:focus-visible {
  outline: none;
  background: var(--surface-2);
  color: var(--text);
}

.context-action:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.context-separator {
  height: 1px;
  margin: 5px 6px;
  background: var(--border);
}

.menu-brand {
  padding: 7px 8px;
  color: var(--text-muted);
  font-size: 11px;
  letter-spacing: 0.01em;
  cursor: default;
}

.menu-brand strong {
  color: var(--primary);
  font-weight: 700;
}

@keyframes globalMenuIn {
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
