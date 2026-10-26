<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import {
  AlertTriangle,
  Check,
  Copy,
  FolderOpen,
  Loader2,
  RefreshCw,
  ScrollText,
  Search,
  Terminal,
  X,
} from '@lucide/vue'
import { logDashboardWarn } from '../composables/useDashboardLog'
import { useLogs } from '../composables/useServices'
import type { RunInfo } from '../types'

interface Props {
  serviceId: string | null
  serviceName?: string
  status?: string
  error?: string
}

interface Emits {
  (e: 'close'): void
}

interface LogStreamEvent {
  serviceId: string
  runId: string
  lines: string[]
}

const props = defineProps<Props>()
defineEmits<Emits>()

const { readLogs, listRuns, openLogFolder, startLogStream, stopLogStream } = useLogs()

const logs = ref<string[]>([])
const loading = ref(false)
const runId = ref<string | null>(null)
const availableRuns = ref<RunInfo[]>([])
const tailLines = ref(1000)
const live = ref(true)
const follow = ref(true)
const search = ref('')
const copied = ref(false)
const logContainer = ref<HTMLElement | null>(null)
let unlistenLogs: UnlistenFn | undefined
let copyTimer: number | undefined
let activeStreamRunId = ''

const displayName = computed(() => props.serviceName || props.serviceId || 'Servicio')
const statusLabel = computed(() => {
  const labels: Record<string, string> = {
    running: 'En ejecución',
    stopped: 'Detenido',
    starting: 'Iniciando',
    stopping: 'Deteniendo',
    error: 'Error',
    external: 'Externo',
  }
  return labels[props.status ?? ''] ?? props.status ?? ''
})

const filteredLines = computed(() => {
  const query = search.value.trim().toLowerCase()
  return logs.value
    .map((text, index) => ({ text, index }))
    .filter((line) => !query || line.text.toLowerCase().includes(query))
})

function scrollToBottom() {
  if (!follow.value) return
  nextTick(() => {
    if (logContainer.value) {
      logContainer.value.scrollTop = logContainer.value.scrollHeight
    }
  })
}

function onScroll() {
  const element = logContainer.value
  if (!element) return
  const atBottom = element.scrollHeight - element.scrollTop - element.clientHeight < 32
  if (!atBottom && follow.value) follow.value = false
}

async function loadRuns() {
  if (!props.serviceId) {
    availableRuns.value = []
    return
  }

  try {
    availableRuns.value = await listRuns(props.serviceId)
  } catch (error) {
    console.error('Error listando ejecuciones:', error)
    void logDashboardWarn('logs.listRuns', error, { throttleMs: 10000 })
  }
}

async function loadLogs() {
  if (!props.serviceId) {
    logs.value = []
    return
  }

  loading.value = true
  try {
    const result = await readLogs(props.serviceId, runId.value || undefined, tailLines.value)
    activeStreamRunId = result.runId
    logs.value = result.lines
    scrollToBottom()
  } catch (error) {
    console.error('Error leyendo logs:', error)
    logs.value = ['Error cargando logs: ' + (error as Error).message]
    void logDashboardWarn('logs.read', error, { throttleMs: 10000 })
  } finally {
    loading.value = false
  }
}

function appendLines(lines: string[]) {
  if (lines.length === 0) return
  logs.value.push(...lines)
  const maxLines = Math.max(tailLines.value, 2000)
  if (logs.value.length > maxLines) {
    logs.value = logs.value.slice(-maxLines)
  }
  scrollToBottom()
}

async function startStream() {
  if (!props.serviceId || !live.value) return
  try {
    await startLogStream(props.serviceId, runId.value || undefined)
  } catch (error) {
    console.error('Error iniciando stream de logs:', error)
    void logDashboardWarn('logs.startStream', error, { throttleMs: 10000 })
  }
}

async function stopStream() {
  if (!props.serviceId) return
  try {
    await stopLogStream(props.serviceId)
  } catch (error) {
    console.error('Error deteniendo stream de logs:', error)
    void logDashboardWarn('logs.stopStream', error, { throttleMs: 10000 })
  }
}

async function restartStream() {
  await stopStream()
  await startStream()
}

async function toggleLive() {
  if (live.value) await startStream()
  else await stopStream()
}

async function changeRun() {
  await loadLogs()
  await restartStream()
}

function openFolder() {
  if (props.serviceId) openLogFolder(props.serviceId)
}

async function writeToClipboard(text: string) {
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text)
    return
  }

  const textarea = document.createElement('textarea')
  textarea.value = text
  textarea.style.position = 'fixed'
  textarea.style.top = '-9999px'
  document.body.appendChild(textarea)
  textarea.focus()
  textarea.select()
  const success = document.execCommand('copy')
  document.body.removeChild(textarea)
  if (!success) throw new Error('No se pudo copiar al portapapeles')
}

async function copyLogs() {
  try {
    await writeToClipboard(filteredLines.value.map((line) => line.text).join('\n'))
    copied.value = true
    if (copyTimer !== undefined) window.clearTimeout(copyTimer)
    copyTimer = window.setTimeout(() => {
      copied.value = false
    }, 1500)
  } catch (error) {
    console.error('Error copiando logs:', error)
    void logDashboardWarn('logs.copy', error, { notify: true })
  }
}

function lineClass(text: string) {
  const normalized = text.toLowerCase()
  if (
    normalized.includes('error')
    || normalized.includes('traceback')
    || normalized.includes('exception')
    || normalized.includes('modulenotfounderror')
    || normalized.includes('importerror')
    || normalized.includes('npm err!')
    || normalized.includes('failed')
    || normalized.includes('fatal')
    || normalized.includes('eaddrinuse')
  ) {
    return 'is-error'
  }
  if (normalized.includes('warn')) return 'is-warn'
  if (
    normalized.includes('info')
    || normalized.includes('ready')
    || normalized.includes('listening')
    || normalized.includes('started')
    || normalized.includes('running')
  ) {
    return 'is-info'
  }
  return ''
}

onMounted(async () => {
  unlistenLogs = await listen<LogStreamEvent>('service-log', (event) => {
    if (event.payload.serviceId !== props.serviceId) return
    if (runId.value && event.payload.runId !== runId.value) return
    if (!runId.value && activeStreamRunId && event.payload.runId !== activeStreamRunId) {
      logs.value = []
    }
    activeStreamRunId = event.payload.runId
    appendLines(event.payload.lines)
  })

  await loadRuns()
  await loadLogs()
  await startStream()
})

onUnmounted(() => {
  unlistenLogs?.()
  void stopStream()
  if (copyTimer !== undefined) window.clearTimeout(copyTimer)
})
</script>

<template>
  <div class="log-drawer-overlay" @click="$emit('close')">
    <aside class="log-drawer" role="dialog" aria-modal="true" aria-labelledby="log-drawer-title" @click.stop>
      <div class="drawer-header">
        <div class="drawer-title">
          <div class="drawer-icon">
            <ScrollText :size="18" />
          </div>
          <div>
            <h3 id="log-drawer-title">Logs</h3>
            <p>{{ displayName }}</p>
          </div>
        </div>
        <div class="drawer-header-actions">
          <span class="live-indicator" :class="{ active: live }">
            <span class="status-dot"></span>
            {{ live ? 'En vivo' : 'Pausado' }}
          </span>
          <span v-if="status" class="status-pill" :class="`status-${status}`">
            <span class="status-dot"></span>
            {{ statusLabel }}
          </span>
          <button class="icon-btn" type="button" aria-label="Cerrar logs" @click="$emit('close')">
            <X :size="18" />
          </button>
        </div>
      </div>

      <div class="drawer-toolbar">
        <div class="toolbar-group">
          <label for="log-run">Ejecución</label>
          <select id="log-run" v-model="runId" class="input small-input" :disabled="loading" @change="changeRun">
            <option value="">Actual</option>
            <option v-for="run in availableRuns" :key="run.runId" :value="run.runId">
              {{ run.runId }} ({{ run.startTime }}) {{ run.endTime ? '– ' + run.endTime : ' (activa)' }}
            </option>
          </select>
          <input v-model.number="tailLines" class="input small-input lines-input" type="number" min="50" max="5000" step="50" />
          <span class="toolbar-hint">líneas</span>
        </div>

        <div class="toolbar-group">
          <label class="switch">
            <input v-model="live" type="checkbox" @change="toggleLive" />
            <span class="switch-track">
              <span class="switch-thumb"></span>
            </span>
            <span>En vivo</span>
          </label>
          <label class="switch">
            <input v-model="follow" type="checkbox" />
            <span class="switch-track">
              <span class="switch-thumb"></span>
            </span>
            <span>Seguir</span>
          </label>
        </div>
      </div>

      <div class="drawer-toolbar secondary">
        <div class="search-box">
          <Search :size="14" />
          <input v-model="search" class="search-input" type="text" placeholder="Buscar en el log..." />
        </div>
        <div class="toolbar-group">
          <button class="btn btn-secondary btn-sm" type="button" @click="copyLogs">
            <Check v-if="copied" :size="14" />
            <Copy v-else :size="14" />
            {{ copied ? 'Copiado' : 'Copiar' }}
          </button>
          <button class="btn btn-secondary btn-sm" type="button" :disabled="loading" @click="loadLogs">
            <RefreshCw :size="14" :class="{ spinning: loading }" />
            Actualizar
          </button>
          <button class="btn btn-secondary btn-sm" type="button" @click="openFolder">
            <FolderOpen :size="14" />
            Carpeta
          </button>
        </div>
      </div>

      <div v-if="status === 'error' || error" class="log-alert">
        <AlertTriangle :size="18" />
        <div>
          <strong>El servicio reportó un error</strong>
          <span>{{ error || 'Revisa las últimas líneas del log.' }}</span>
        </div>
      </div>

      <div ref="logContainer" class="log-content" @scroll="onScroll">
        <div v-if="filteredLines.length === 0 && !loading" class="log-empty">
          <Terminal :size="30" />
          <span>{{ search ? 'Sin coincidencias' : live ? 'Esperando logs del servicio…' : 'Sin logs disponibles' }}</span>
        </div>
        <div v-for="line in filteredLines" :key="line.index" class="log-line" :class="lineClass(line.text)">
          <span class="line-number">{{ line.index + 1 }}</span>
          <span class="line-text">{{ line.text }}</span>
        </div>
        <div v-if="loading" class="loading-overlay">
          <Loader2 class="spinning" :size="20" />
          Cargando…
        </div>
      </div>
    </aside>
  </div>
</template>

<style scoped>
.log-drawer-overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: flex;
  justify-content: flex-end;
  padding: 12px;
  background: var(--overlay);
  backdrop-filter: blur(4px);
}

.log-drawer {
  display: flex;
  flex-direction: column;
  width: min(900px, 100%);
  height: 100%;
  overflow: hidden;
  border: 1px solid var(--border);
  border-radius: var(--radius-xl);
  background: var(--surface);
  box-shadow: var(--shadow-lg);
  animation: slideIn 0.22s ease-out;
}

.drawer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 16px 18px;
  border-bottom: 1px solid var(--border);
  background: var(--surface);
}

.drawer-title {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.drawer-icon {
  display: grid;
  place-items: center;
  width: 40px;
  height: 40px;
  border-radius: 13px;
  background: var(--primary-soft);
  color: var(--primary);
}

.drawer-title h3 {
  margin: 0;
  color: var(--text);
  font-size: 15px;
  letter-spacing: -0.01em;
}

.drawer-title p {
  margin: 3px 0 0;
  overflow: hidden;
  color: var(--text-muted);
  font-size: 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.drawer-header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.live-indicator {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 9px;
  border-radius: 999px;
  background: var(--surface-2);
  color: var(--text-subtle);
  font-size: 11px;
  font-weight: 700;
}

.live-indicator.active {
  background: var(--success-soft);
  color: var(--success);
}

.live-indicator.active .status-dot {
  animation: livePulse 1.6s ease-in-out infinite;
}

@keyframes livePulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.35;
  }
}

.drawer-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
  padding: 12px 18px;
  border-bottom: 1px solid var(--border);
  background: var(--surface-2);
  font-size: 12px;
}

.drawer-toolbar.secondary {
  background: var(--surface);
}

.toolbar-group {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.toolbar-group > label {
  color: var(--text-muted);
}

.small-input {
  width: auto;
  min-height: 32px;
  padding: 6px 9px;
  font-size: 12px;
}

.lines-input {
  width: 72px;
}

.toolbar-hint {
  color: var(--text-subtle);
}

.search-box {
  position: relative;
  flex: 1;
  min-width: 200px;
}

.search-box svg {
  position: absolute;
  top: 50%;
  left: 10px;
  color: var(--text-subtle);
  pointer-events: none;
  transform: translateY(-50%);
}

.search-input {
  width: 100%;
  min-height: 32px;
  padding: 6px 10px 6px 32px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface);
  color: var(--text);
  font-size: 12px;
}

.search-input:focus {
  outline: none;
  border-color: var(--primary);
  box-shadow: var(--focus-ring);
}

.log-alert {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  margin: 12px 18px 0;
  padding: 11px 12px;
  border: 1px solid var(--danger);
  border-radius: var(--radius-sm);
  background: var(--danger-soft);
  color: var(--danger);
  font-size: 12px;
}

.log-alert div {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.log-alert span {
  color: var(--text-muted);
  font-family: var(--mono);
  font-size: 11px;
}

.log-content {
  position: relative;
  flex: 1;
  overflow: auto;
  margin: 12px 0 0;
  padding: 12px 0;
  background: var(--log-bg);
  color: var(--log-text);
  font-family: var(--mono);
  font-size: 11.5px;
  line-height: 1.65;
}

.log-line {
  display: flex;
  gap: 12px;
  padding: 0 16px;
}

.log-line:hover {
  background: rgba(148, 163, 184, 0.08);
}

.line-number {
  width: 40px;
  flex-shrink: 0;
  color: var(--log-muted);
  text-align: right;
  user-select: none;
}

.line-text {
  min-width: 0;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.log-line.is-error .line-text {
  color: #fca5a5;
}

.log-line.is-warn .line-text {
  color: #fcd34d;
}

.log-line.is-info .line-text {
  color: #7dd3fc;
}

.log-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  height: 100%;
  color: var(--log-muted);
}

.loading-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  background: var(--overlay-strong);
  color: var(--log-text);
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateX(24px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

@media (max-width: 640px) {
  .log-drawer-overlay {
    padding: 0;
  }

  .log-drawer {
    border-radius: 0;
  }

  .drawer-toolbar {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
