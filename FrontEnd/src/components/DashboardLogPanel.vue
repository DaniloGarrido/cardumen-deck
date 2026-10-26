<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { confirm } from '@tauri-apps/plugin-dialog'
import {
  AlertTriangle,
  Check,
  ChevronDown,
  ChevronRight,
  Copy,
  FolderOpen,
  RefreshCw,
  ScrollText,
  Search,
  Trash2,
  X,
} from '@lucide/vue'
import { logDashboardError, useDashboardLog } from '../composables/useDashboardLog'
import { writeToClipboard } from '../utils/clipboard'
import type { DashboardLogEntry, DashboardLogPage } from '../types'

const { read, clear, openFolder } = useDashboardLog()

const page = ref<DashboardLogPage | null>(null)
const loading = ref(true)
const error = ref('')
const level = ref('all')
const query = ref('')
const autoRefresh = ref(true)
const expandedId = ref<string | null>(null)
const copiedId = ref<string | null>(null)
let refreshTimer: number | undefined
let searchTimer: number | undefined
let copyTimer: number | undefined

const levels = [
  { value: 'all', label: 'Todos' },
  { value: 'error', label: 'Errores' },
  { value: 'warn', label: 'Advertencias' },
  { value: 'info', label: 'Información' },
]

async function load(silent = false) {
  if (!silent) loading.value = true
  error.value = ''
  try {
    page.value = await read(500, level.value, query.value)
  } catch (cause) {
    error.value = String(cause)
  } finally {
    loading.value = false
  }
}

function scheduleSearch() {
  if (searchTimer !== undefined) window.clearTimeout(searchTimer)
  searchTimer = window.setTimeout(() => {
    void load()
  }, 320)
}

async function clearLog() {
  if (!(await confirm('¿Limpiar todo el registro del dashboard? Esta acción no se puede deshacer.'))) {
    return
  }
  try {
    await clear()
    expandedId.value = null
    await load()
  } catch (cause) {
    error.value = String(cause)
    void logDashboardError('dashboardLog.clear', cause)
  }
}

async function openLogFolder() {
  try {
    await openFolder()
  } catch (cause) {
    error.value = String(cause)
    void logDashboardError('dashboardLog.openFolder', cause)
  }
}

async function copyEntry(entry: DashboardLogEntry) {
  try {
    await writeToClipboard(formatEntry(entry))
    copiedId.value = entry.id
    if (copyTimer !== undefined) window.clearTimeout(copyTimer)
    copyTimer = window.setTimeout(() => {
      copiedId.value = null
    }, 1500)
  } catch (cause) {
    void logDashboardError('dashboardLog.copy', cause, { notify: true })
  }
}

function toggleEntry(id: string) {
  expandedId.value = expandedId.value === id ? null : id
}

function formatEntry(entry: DashboardLogEntry): string {
  const lines = [
    `[${entry.timestamp}] ${entry.level.toUpperCase()} ${entry.scope}`,
    entry.message,
  ]
  if (entry.serviceName || entry.serviceId) {
    lines.push(`Servicio: ${entry.serviceName ?? ''} ${entry.serviceId ? `(${entry.serviceId})` : ''}`)
  }
  if (entry.details) lines.push(`Detalles:\n${entry.details}`)
  if (entry.context) lines.push(`Contexto:\n${JSON.stringify(entry.context, null, 2)}`)
  return lines.join('\n')
}

function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp)
  if (Number.isNaN(date.getTime())) return timestamp
  return new Intl.DateTimeFormat('es', {
    dateStyle: 'short',
    timeStyle: 'medium',
  }).format(date)
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`
}

function prettyContext(entry: DashboardLogEntry): string {
  return entry.context ? JSON.stringify(entry.context, null, 2) : ''
}

function levelLabel(value: string): string {
  if (value === 'error') return 'Error'
  if (value === 'warn') return 'Aviso'
  return 'Info'
}

watch(level, () => {
  void load()
})

watch(query, () => {
  scheduleSearch()
})

onMounted(async () => {
  await load()
  refreshTimer = window.setInterval(() => {
    if (autoRefresh.value && !document.hidden) void load(true)
  }, 2000)
})

onBeforeUnmount(() => {
  if (refreshTimer !== undefined) window.clearInterval(refreshTimer)
  if (searchTimer !== undefined) window.clearTimeout(searchTimer)
  if (copyTimer !== undefined) window.clearTimeout(copyTimer)
})
</script>

<template>
  <section class="log-panel">
    <header class="panel-head">
      <div class="panel-title">
        <div class="panel-icon"><ScrollText :size="19" /></div>
        <div>
          <h3>Registro del dashboard</h3>
          <p>Errores y eventos internos de la aplicación.</p>
        </div>
      </div>

      <div class="panel-actions">
        <button class="btn btn-secondary btn-sm" type="button" title="Actualizar" @click="load()">
          <RefreshCw :size="14" />
          Actualizar
        </button>
        <button class="btn btn-secondary btn-sm" type="button" title="Abrir carpeta" @click="openLogFolder">
          <FolderOpen :size="14" />
          Abrir carpeta
        </button>
        <button class="btn btn-ghost btn-sm log-clear" type="button" @click="clearLog">
          <Trash2 :size="14" />
          Limpiar
        </button>
      </div>
    </header>

    <div class="log-toolbar">
      <div class="level-chips" role="group" aria-label="Filtrar por nivel">
        <button
          v-for="item in levels"
          :key="item.value"
          class="level-chip"
          :class="[`is-${item.value}`, { 'is-active': level === item.value }]"
          type="button"
          :aria-pressed="level === item.value"
          @click="level = item.value"
        >
          {{ item.label }}
        </button>
      </div>

      <div class="log-search">
        <Search :size="14" aria-hidden="true" />
        <input v-model="query" type="text" placeholder="Buscar en el registro..." aria-label="Buscar en el registro" />
        <button v-if="query" type="button" aria-label="Limpiar búsqueda" @click="query = ''">
          <X :size="13" />
        </button>
      </div>

      <label class="live-toggle">
        <input v-model="autoRefresh" type="checkbox" />
        <span class="live-dot" :class="{ 'is-on': autoRefresh }"></span>
        En vivo
      </label>
    </div>

    <div class="log-meta">
      <span>
        <strong>{{ page?.entries.length ?? 0 }}</strong>
        de {{ page?.total ?? 0 }} entradas
      </span>
      <span>{{ formatSize(page?.sizeBytes ?? 0) }}</span>
      <code :title="page?.path">{{ page?.path || 'Sin archivo todavía' }}</code>
    </div>

    <div class="log-list">
      <div v-if="loading" class="log-state">
        <RefreshCw :size="18" class="spin" />
        Cargando registro...
      </div>

      <div v-else-if="error" class="log-state is-error">
        <AlertTriangle :size="18" />
        {{ error }}
      </div>

      <div v-else-if="!page?.entries.length" class="log-state">
        <ScrollText :size="20" />
        <strong>No hay entradas</strong>
        <span>Los errores y eventos internos aparecerán aquí.</span>
      </div>

      <template v-else>
        <article
          v-for="entry in page?.entries"
          :key="entry.id"
          class="log-entry"
          :class="`is-${entry.level}`"
          tabindex="0"
          role="button"
          :aria-expanded="expandedId === entry.id"
          @click="toggleEntry(entry.id)"
          @keydown.enter.prevent="toggleEntry(entry.id)"
          @keydown.space.prevent="toggleEntry(entry.id)"
        >
        <div class="entry-main">
          <span class="level-badge">{{ levelLabel(entry.level) }}</span>

          <div class="entry-body">
            <div class="entry-top">
              <strong>{{ entry.message }}</strong>
              <time>{{ formatTimestamp(entry.timestamp) }}</time>
            </div>

            <div class="entry-sub">
              <span>{{ entry.scope }}</span>
              <span v-if="entry.serviceName">{{ entry.serviceName }}</span>
              <span>{{ entry.source }}</span>
            </div>

            <div v-if="expandedId === entry.id" class="entry-details">
              <div v-if="entry.details" class="detail-block">
                <span>Detalles</span>
                <pre>{{ entry.details }}</pre>
              </div>
              <div v-if="entry.context" class="detail-block">
                <span>Contexto</span>
                <pre>{{ prettyContext(entry) }}</pre>
              </div>
              <div v-if="entry.serviceId" class="detail-block">
                <span>ID del servicio</span>
                <code>{{ entry.serviceId }}</code>
              </div>
            </div>
          </div>

          <button
            class="entry-copy"
            type="button"
            :title="copiedId === entry.id ? 'Entrada copiada' : 'Copiar entrada'"
            :aria-label="copiedId === entry.id ? 'Entrada copiada' : 'Copiar entrada'"
            @click.stop="copyEntry(entry)"
          >
            <Check v-if="copiedId === entry.id" :size="13" />
            <Copy v-else :size="13" />
          </button>

          <component
            :is="expandedId === entry.id ? ChevronDown : ChevronRight"
            :size="15"
            class="entry-chevron"
            aria-hidden="true"
          />
        </div>
        </article>
      </template>
    </div>

    <footer v-if="page?.truncated" class="log-footer">
      Se muestran las 500 entradas más recientes. Abre el archivo para ver el historial completo.
    </footer>
  </section>
</template>

<style scoped>
.log-panel {
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.panel-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  flex-wrap: wrap;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--border);
}

.panel-title {
  display: flex;
  align-items: center;
  gap: 12px;
}

.panel-icon {
  display: grid;
  place-items: center;
  width: 42px;
  height: 42px;
  border-radius: 13px;
  background: var(--primary-soft);
  color: var(--primary);
}

.panel-title h3 {
  margin: 0;
  color: var(--text);
  font-size: 16px;
  letter-spacing: -0.01em;
}

.panel-title p {
  margin: 3px 0 0;
  color: var(--text-muted);
  font-size: 12px;
}

.panel-actions {
  display: flex;
  align-items: center;
  gap: 7px;
  flex-wrap: wrap;
}

.log-clear {
  color: var(--danger);
}

.log-clear:hover:not(:disabled) {
  background: var(--danger-soft);
  color: var(--danger);
}

.log-toolbar {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  padding: 14px 0 10px;
}

.level-chips {
  display: flex;
  align-items: center;
  gap: 5px;
  flex-wrap: wrap;
}

.level-chip {
  padding: 5px 9px;
  border: 1px solid var(--border);
  border-radius: 999px;
  background: var(--surface);
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
}

.level-chip:hover {
  border-color: var(--border-strong);
  color: var(--text);
}

.level-chip.is-active {
  border-color: var(--primary);
  background: var(--primary-soft);
  color: var(--primary);
}

.level-chip.is-error.is-active {
  border-color: var(--danger);
  background: var(--danger-soft);
  color: var(--danger);
}

.level-chip.is-warn.is-active {
  border-color: var(--warning);
  background: var(--warning-soft);
  color: var(--warning);
}

.log-search {
  display: flex;
  align-items: center;
  gap: 7px;
  min-width: 210px;
  flex: 1;
  padding: 0 9px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface);
  color: var(--text-subtle);
}

.log-search:focus-within {
  border-color: var(--primary);
  box-shadow: var(--focus-ring);
}

.log-search input {
  min-width: 0;
  flex: 1;
  padding: 8px 0;
  border: 0;
  outline: 0;
  background: transparent;
  color: var(--text);
  font-size: 12px;
}

.log-search button {
  display: inline-flex;
  padding: 2px;
  border: 0;
  background: transparent;
  color: var(--text-subtle);
  cursor: pointer;
}

.live-toggle {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
}

.live-toggle input {
  position: absolute;
  opacity: 0;
  pointer-events: none;
}

.live-dot {
  width: 7px;
  height: 7px;
  border-radius: 999px;
  background: var(--text-subtle);
}

.live-dot.is-on {
  background: var(--success);
  box-shadow: 0 0 0 3px var(--success-soft);
  animation: livePulse 2s ease-in-out infinite;
}

.log-meta {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
  padding: 0 0 10px;
  color: var(--text-subtle);
  font-size: 10px;
}

.log-meta code {
  min-width: 0;
  overflow: hidden;
  color: var(--text-subtle);
  font-family: var(--mono);
  font-size: 10px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.log-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-height: 220px;
  max-height: 52vh;
  padding: 2px;
  overflow: auto;
}

.log-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  min-height: 220px;
  flex-direction: column;
  color: var(--text-muted);
  font-size: 12px;
  text-align: center;
}

.log-state strong {
  color: var(--text);
  font-size: 13px;
}

.log-state span {
  color: var(--text-subtle);
}

.log-state.is-error {
  flex-direction: row;
  color: var(--danger);
}

.spin {
  animation: spin 1s linear infinite;
}

.log-entry {
  border: 1px solid var(--border);
  border-left-width: 4px;
  border-radius: var(--radius-sm);
  background: var(--surface);
  cursor: pointer;
  transition:
    border-color var(--transition),
    background var(--transition);
}

.log-entry:hover {
  border-color: var(--border-strong);
  background: var(--surface-2);
}

.log-entry.is-error {
  border-left-color: var(--danger);
}

.log-entry.is-warn {
  border-left-color: var(--warning);
}

.log-entry.is-info {
  border-left-color: var(--info);
}

.entry-main {
  display: flex;
  align-items: flex-start;
  gap: 9px;
  padding: 9px 10px;
}

.level-badge {
  flex-shrink: 0;
  padding: 3px 6px;
  border-radius: 5px;
  background: var(--surface-2);
  color: var(--text-muted);
  font-size: 9px;
  font-weight: 800;
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

.is-error .level-badge {
  background: var(--danger-soft);
  color: var(--danger);
}

.is-warn .level-badge {
  background: var(--warning-soft);
  color: var(--warning);
}

.is-info .level-badge {
  background: var(--info-soft);
  color: var(--info);
}

.entry-body {
  min-width: 0;
  flex: 1;
}

.entry-top {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 10px;
}

.entry-top strong {
  overflow: hidden;
  color: var(--text);
  font-size: 12px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.entry-top time {
  flex-shrink: 0;
  color: var(--text-subtle);
  font-family: var(--mono);
  font-size: 9px;
}

.entry-sub {
  display: flex;
  gap: 7px;
  flex-wrap: wrap;
  margin-top: 3px;
  color: var(--text-subtle);
  font-size: 10px;
}

.entry-sub span + span::before {
  content: "·";
  margin-right: 7px;
}

.entry-details {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 9px;
  padding-top: 8px;
  border-top: 1px dashed var(--border);
}

.detail-block {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.detail-block > span {
  color: var(--text-subtle);
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.detail-block pre,
.detail-block code {
  margin: 0;
  overflow: auto;
  color: var(--text-muted);
  font-family: var(--mono);
  font-size: 10px;
  line-height: 1.45;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.entry-copy {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  flex-shrink: 0;
  padding: 0;
  border: 1px solid transparent;
  border-radius: 5px;
  background: transparent;
  color: var(--text-subtle);
  cursor: pointer;
}

.entry-copy:hover {
  border-color: var(--border);
  background: var(--surface);
  color: var(--text);
}

.entry-chevron {
  flex-shrink: 0;
  margin-top: 4px;
  color: var(--text-subtle);
}

.log-footer {
  padding-top: 9px;
  border-top: 1px solid var(--border);
  color: var(--text-subtle);
  font-size: 10px;
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

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

@media (prefers-reduced-motion: reduce) {
  .live-dot.is-on,
  .spin {
    animation: none;
  }
}

@media (max-width: 620px) {
  .log-list {
    max-height: 46vh;
  }

  .panel-actions {
    width: 100%;
  }

  .panel-actions .btn {
    flex: 1;
    justify-content: center;
  }

  .log-toolbar {
    align-items: stretch;
    flex-direction: column;
  }

  .log-search {
    min-width: 0;
  }
}
</style>
