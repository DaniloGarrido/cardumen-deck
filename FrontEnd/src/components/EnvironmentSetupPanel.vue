<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import {
  AlertTriangle,
  CheckCircle2,
  FolderCheck,
  Loader2,
  Package,
  Play,
  RefreshCw,
  Square,
  Wrench,
} from '@lucide/vue'
import { logDashboardError, logDashboardWarn } from '../composables/useDashboardLog'
import { useEnvironment } from '../composables/useEnvironment'
import type { EnvironmentInfo, SetupStatus } from '../types'

interface Props {
  cwd: string
  setupCommand: string
}

interface Emits {
  (e: 'update:setupCommand', value: string): void
  (e: 'apply', info: EnvironmentInfo): void
  (e: 'detected', info: EnvironmentInfo): void
}

interface SetupLogEvent {
  taskId: string
  lines: string[]
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const {
  detect,
  startSetup,
  getSetupStatus,
  readSetupLog,
  cancelSetup,
  startSetupLogStream,
  stopSetupLogStream,
} = useEnvironment()

const info = ref<EnvironmentInfo | null>(null)
const detecting = ref(false)
const detectError = ref('')
const setupStatus = ref<SetupStatus | null>(null)
const setupLines = ref<string[]>([])
const setupRunning = ref(false)
const logContainer = ref<HTMLElement | null>(null)
let taskId = ''
let pollTimer: number | undefined
let detectTimer: number | undefined
let unlistenSetupLogs: UnlistenFn | undefined

const setupCommandModel = computed({
  get: () => props.setupCommand,
  set: (value: string) => emit('update:setupCommand', value),
})

const setupStatusLabel = computed(() => {
  const status = setupStatus.value?.status
  if (status === 'success') return 'Completado'
  if (status === 'error') return 'Falló'
  if (status === 'cancelled') return 'Cancelado'
  return 'Preparando…'
})

const venvMismatch = computed(() => {
  const venvPath = info.value?.venvPath
  const command = props.setupCommand.trim().toLowerCase()
  if (!venvPath || !command) return false
  return !command.includes(venvPath.toLowerCase())
})

function stopPolling() {
  if (pollTimer !== undefined) {
    window.clearInterval(pollTimer)
    pollTimer = undefined
  }
}

function scrollSetupToBottom() {
  nextTick(() => {
    if (logContainer.value) {
      logContainer.value.scrollTop = logContainer.value.scrollHeight
    }
  })
}

function appendSetupLines(lines: string[]) {
  if (lines.length === 0) return
  setupLines.value.push(...lines)
  if (setupLines.value.length > 2000) {
    setupLines.value = setupLines.value.slice(-2000)
  }
  scrollSetupToBottom()
}

async function syncSetupLog() {
  if (!taskId) return
  try {
    const log = await readSetupLog(taskId, 500)
    setupLines.value = log.lines
    scrollSetupToBottom()
  } catch (error) {
    console.error('Error sincronizando salida de preparación:', error)
    void logDashboardWarn('environment.syncLog', error, { throttleMs: 10000 })
  }
}

async function refreshSetup() {
  if (!taskId) return

  try {
    const status = await getSetupStatus(taskId)
    setupStatus.value = status

    if (status.status !== 'running') {
      setupRunning.value = false
      stopPolling()
      await stopSetupLogStream(taskId)
      await syncSetupLog()
    }
  } catch (error) {
    console.error('Error actualizando preparación:', error)
    void logDashboardWarn('environment.status', error, { throttleMs: 10000 })
    setupRunning.value = false
    stopPolling()
  }
}

async function detectEnvironment() {
  if (!props.cwd.trim()) {
    info.value = null
    detectError.value = ''
    return
  }

  detecting.value = true
  detectError.value = ''
  try {
    info.value = await detect(props.cwd)
    emit('detected', info.value)
    if (!props.setupCommand.trim() && info.value.setupCommand) {
      emit('update:setupCommand', info.value.setupCommand)
    }
  } catch (error) {
    info.value = null
    detectError.value = (error as Error).message
    void logDashboardWarn('environment.detect', error, {
      throttleMs: 10000,
      context: { cwd: props.cwd },
    })
  } finally {
    detecting.value = false
  }
}

function applySuggestions() {
  if (info.value) emit('apply', info.value)
}

async function startPreparation() {
  if (!props.cwd.trim() || !props.setupCommand.trim() || setupRunning.value) return

  if (taskId) {
    try {
      await stopSetupLogStream(taskId)
    } catch {
      // Ignore stale streams.
    }
    taskId = ''
  }

  setupLines.value = []
  setupStatus.value = null
  setupRunning.value = true

  try {
    const status = await startSetup(props.cwd, props.setupCommand)
    taskId = status.taskId
    setupStatus.value = status
    try {
      await startSetupLogStream(taskId)
    } catch (error) {
      console.error('Error iniciando stream de preparación:', error)
      void logDashboardWarn('environment.startStream', error, { throttleMs: 10000 })
    }
    await syncSetupLog()
    stopPolling()
    pollTimer = window.setInterval(refreshSetup, 1000)
    await refreshSetup()
  } catch (error) {
    setupRunning.value = false
    setupStatus.value = null
    setupLines.value = [`Error iniciando preparación: ${(error as Error).message}`]
    void logDashboardError('environment.startSetup', error, {
      title: 'No se pudo iniciar la preparación del entorno',
      notify: true,
      context: { cwd: props.cwd, setupCommand: props.setupCommand },
    })
  }
}

async function cancelPreparation() {
  if (!taskId) return
  try {
    await cancelSetup(taskId)
    await refreshSetup()
  } catch (error) {
    console.error('Error cancelando preparación:', error)
    void logDashboardWarn('environment.cancel', error, { notify: true })
  }
}

watch(() => props.cwd, () => {
  if (detectTimer !== undefined) window.clearTimeout(detectTimer)
  detectTimer = window.setTimeout(detectEnvironment, 250)
}, { immediate: true })

onMounted(async () => {
  unlistenSetupLogs = await listen<SetupLogEvent>('setup-log', (event) => {
    if (event.payload.taskId !== taskId) return
    appendSetupLines(event.payload.lines)
  })
})

onUnmounted(() => {
  stopPolling()
  if (detectTimer !== undefined) window.clearTimeout(detectTimer)
  if (taskId) void stopSetupLogStream(taskId)
  unlistenSetupLogs?.()
})
</script>

<template>
  <div class="env-panel">
    <div class="env-header">
      <div class="env-title">
        <Wrench :size="15" />
        Entorno y dependencias
      </div>
      <button class="btn btn-ghost btn-sm" type="button" :disabled="detecting || !cwd" @click="detectEnvironment">
        <RefreshCw :size="14" :class="{ spinning: detecting }" />
        {{ detecting ? 'Detectando…' : 'Detectar' }}
      </button>
    </div>

    <div v-if="info" class="env-detected">
      <span class="env-stack">
        <Package :size="13" />
        {{ info.displayName }}
      </span>
      <span v-if="info.packageManager" class="env-chip">{{ info.packageManager }}</span>
      <span v-if="info.venvPath" class="env-chip">
        <FolderCheck :size="12" />
        venv
      </span>
      <span v-for="file in info.detectedFiles" :key="file" class="env-file">{{ file }}</span>
    </div>

    <p v-if="detectError" class="env-message error">{{ detectError }}</p>
    <p v-else-if="!cwd" class="env-message">Selecciona una carpeta para detectar el entorno.</p>

    <div class="env-command-row">
      <input
        v-model="setupCommandModel"
        class="input"
        type="text"
        placeholder="Comando para preparar el entorno"
        @keydown.enter.prevent
      />
      <button class="btn btn-secondary btn-sm" type="button" :disabled="!info" @click="applySuggestions">
        Aplicar sugerencias
      </button>
    </div>

    <div v-if="info?.warnings.length || venvMismatch" class="env-warnings">
      <span v-for="warning in info?.warnings ?? []" :key="warning" class="env-message warning">
        <AlertTriangle :size="13" />
        {{ warning }}
      </span>
      <span v-if="venvMismatch" class="env-message warning">
        <AlertTriangle :size="13" />
        Se detectó un venv. El comando actual parece usar el Python global; usa “Aplicar sugerencias”.
      </span>
    </div>

    <div class="env-actions">
      <button
        class="btn btn-primary btn-sm"
        type="button"
        :disabled="!cwd || !setupCommand.trim() || setupRunning"
        @click="startPreparation"
      >
        <Loader2 v-if="setupRunning" :size="14" class="spinning" />
        <Play v-else :size="14" />
        Preparar entorno
      </button>
      <button v-if="setupRunning" class="btn btn-secondary btn-sm" type="button" @click="cancelPreparation">
        <Square :size="13" />
        Cancelar
      </button>
      <span v-if="setupStatus" class="setup-status" :class="`is-${setupStatus.status}`">
        <CheckCircle2 v-if="setupStatus.status === 'success'" :size="13" />
        <AlertTriangle v-else-if="setupStatus.status === 'error' || setupStatus.status === 'cancelled'" :size="13" />
        <Loader2 v-else :size="13" class="spinning" />
        {{ setupStatusLabel }}
        <span v-if="setupStatus.exitCode !== undefined && setupStatus.exitCode !== 0">(código {{ setupStatus.exitCode }})</span>
      </span>
    </div>

    <div v-if="setupLines.length > 0 || setupRunning" ref="logContainer" class="setup-output">
      <div v-if="setupLines.length === 0" class="setup-empty">Esperando salida del comando…</div>
      <div v-for="(line, index) in setupLines" :key="index" class="setup-line">{{ line }}</div>
    </div>
  </div>
</template>

<style scoped>
.env-panel {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface-2);
}

.env-header,
.env-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  flex-wrap: wrap;
}

.env-title {
  display: flex;
  align-items: center;
  gap: 7px;
  color: var(--text);
  font-size: 12px;
  font-weight: 700;
}

.env-detected {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

.env-stack,
.env-chip,
.env-file {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 7px;
  border-radius: 999px;
  font-size: 10px;
  font-weight: 600;
}

.env-stack {
  background: var(--primary-soft);
  color: var(--primary);
}

.env-chip {
  background: var(--success-soft);
  color: var(--success);
}

.env-file {
  background: var(--surface);
  color: var(--text-muted);
  font-family: var(--mono);
}

.env-command-row {
  display: flex;
  gap: 8px;
}

.env-command-row .input {
  flex: 1;
  min-width: 0;
  font-family: var(--mono);
  font-size: 11px;
}

.env-message {
  display: flex;
  align-items: center;
  gap: 5px;
  margin: 0;
  color: var(--text-muted);
  font-size: 11px;
}

.env-message.error {
  color: var(--danger);
}

.env-message.warning {
  color: var(--warning);
}

.env-warnings {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.setup-status {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 11px;
  font-weight: 600;
}

.setup-status.is-running {
  color: var(--primary);
}

.setup-status.is-success {
  color: var(--success);
}

.setup-status.is-error,
.setup-status.is-cancelled {
  color: var(--danger);
}

.setup-output {
  max-height: 220px;
  overflow: auto;
  padding: 10px;
  border: 1px solid var(--log-border);
  border-radius: var(--radius-sm);
  background: var(--log-bg);
  color: var(--log-text);
  font-family: var(--mono);
  font-size: 11px;
  line-height: 1.55;
}

.setup-empty {
  color: var(--log-muted);
}

.setup-line {
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

@media (max-width: 620px) {
  .env-command-row {
    flex-direction: column;
  }
}
</style>
