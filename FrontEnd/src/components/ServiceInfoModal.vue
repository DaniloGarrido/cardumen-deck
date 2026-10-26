<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import {
  Activity,
  AlertTriangle,
  Check,
  ChevronDown,
  ChevronRight,
  Copy,
  Cpu,
  ExternalLink,
  FolderInput,
  FolderOpen,
  GitBranch,
  Globe,
  HeartPulse,
  MemoryStick,
  Package,
  Pencil,
  ScrollText,
  Search,
  Settings2,
  Terminal,
  Variable,
  X,
} from '@lucide/vue'
import { logDashboardError } from '../composables/useDashboardLog'
import { useServiceDependencies } from '../composables/useServiceDependencies'
import { useLogs } from '../composables/useServices'
import { useServiceItem } from '../composables/useServiceItem'
import { writeToClipboard } from '../utils/clipboard'
import { formatBytes, formatCpuPercent } from '../utils/metricsFormat'
import { formatHealthTime, healthKindLabel, healthLabel } from '../utils/serviceHealth'
import ServiceHealthBadge from './ServiceHealthBadge.vue'
import { resolvePort } from '../utils/serviceDefaults'
import { categoryIconFor } from '../utils/serviceIcons'
import type { Service, ServiceDependency, ServiceRuntime, ServiceUsage } from '../types'

interface Props {
  svc: Service
  status: string
  statusClass: string
  categoryLabel: string
  workspaceName?: string
  gitBranch?: string
  runtime?: ServiceRuntime
  usage?: ServiceUsage
}

interface Emits {
  (e: 'close'): void
  (e: 'edit', svc: Service): void
  (e: 'move', svc: Service): void
  (e: 'logs', id: string): void
  (e: 'git', svc: Service): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const { statusText, accentStyle, serviceUrl, copied, copyUrl, openInBrowser } = useServiceItem({
  service: () => props.svc,
  status: () => props.status,
  pidAlive: () => props.runtime?.pidAlive ?? false,
})

const { openServiceFolder } = useLogs()
const { report: dependencyReport, load: loadDependencies } = useServiceDependencies()

const closeButton = ref<HTMLButtonElement | null>(null)
const commandCopied = ref(false)
const showEnvValues = ref(false)
const dependenciesExpanded = ref(false)
const dependencyQuery = ref('')
const dependencyKind = ref('all')
const dependenciesCopied = ref(false)
let copyTimer: number | undefined
let dependenciesCopyTimer: number | undefined

const dependencyKindLabels: Record<string, string> = {
  runtime: 'Producción',
  dev: 'Desarrollo',
  build: 'Build',
  peer: 'Peer',
  optional: 'Opcional',
  indirect: 'Indirecta',
}
const dependencyKindOrder = ['runtime', 'dev', 'build', 'peer', 'optional', 'indirect']

const command = computed(() =>
  [props.svc.command, ...(props.svc.args ?? [])].filter(Boolean).join(' '),
)
const port = computed(() => resolvePort(props.svc))
const envEntries = computed(() => Object.entries(props.svc.env ?? {}))
const hasRuntimeDetails = computed(
  () => props.runtime?.pid != null || props.runtime?.exitCode != null || Boolean(props.runtime?.error),
)
const cpuUsageLabel = computed(() =>
  props.usage ? formatCpuPercent(props.usage.cpuPercent) : 'Sin datos',
)
const memoryUsageLabel = computed(() =>
  props.usage ? formatBytes(props.usage.memoryBytes) : 'Sin datos',
)
const processUsageLabel = computed(() =>
  props.usage
    ? `${props.usage.processCount} ${props.usage.processCount === 1 ? 'proceso' : 'procesos'}`
    : 'Sin datos',
)

const health = computed(() => props.runtime?.health)
const healthStateLabel = computed(() => healthLabel(health.value?.state))
const healthKind = computed(() => healthKindLabel(health.value?.kind))
const healthCheckedAt = computed(() => formatHealthTime(health.value?.checkedAt))
const healthLatency = computed(() =>
  health.value?.latencyMs != null ? `${health.value.latencyMs} ms` : 'Sin datos',
)

const dependencies = computed(() => dependencyReport.value?.dependencies ?? [])
const hasDependencyInfo = computed(() => Boolean(
  dependencyReport.value
  && (
    dependencyReport.value.total > 0
    || dependencyReport.value.manifests.length > 0
    || dependencyReport.value.warnings.length > 0
  ),
))
const dependencySummary = computed(() => {
  const total = dependencyReport.value?.total ?? 0
  return `${total} ${total === 1 ? 'dependencia' : 'dependencias'}`
})
const previewDependencies = computed(() => dependencies.value.slice(0, 6))
const hiddenDependencyCount = computed(() =>
  Math.max(0, (dependencyReport.value?.total ?? 0) - previewDependencies.value.length),
)
const dependencyKindFilters = computed(() => {
  const counts = new Map<string, number>()
  for (const dependency of dependencies.value) {
    counts.set(dependency.kind, (counts.get(dependency.kind) ?? 0) + 1)
  }

  const filters = [{ value: 'all', label: 'Todas', count: dependencies.value.length }]
  for (const kind of dependencyKindOrder) {
    const count = counts.get(kind) ?? 0
    if (count > 0) {
      filters.push({
        value: kind,
        label: dependencyKindLabels[kind] ?? kind,
        count,
      })
    }
  }
  return filters
})
const filteredDependencies = computed(() => {
  const query = dependencyQuery.value.trim().toLowerCase()
  return dependencies.value.filter((dependency) => {
    if (dependencyKind.value !== 'all' && dependency.kind !== dependencyKind.value) {
      return false
    }
    if (!query) return true
    return [
      dependency.name,
      dependency.version ?? '',
      dependency.group ?? '',
      dependency.manifest,
      dependency.ecosystem,
    ]
      .join(' ')
      .toLowerCase()
      .includes(query)
  })
})
const dependencyGroups = computed(() => {
  return dependencyKindOrder
    .map((kind) => ({
      kind,
      label: dependencyKindLabels[kind] ?? kind,
      dependencies: filteredDependencies.value.filter((dependency) => dependency.kind === kind),
    }))
    .filter((group) => group.dependencies.length > 0)
})

async function copyCommand() {
  try {
    await writeToClipboard(command.value)
    commandCopied.value = true
    if (copyTimer !== undefined) window.clearTimeout(copyTimer)
    copyTimer = window.setTimeout(() => {
      commandCopied.value = false
    }, 1500)
  } catch (error) {
    console.error('No se pudo copiar el comando:', error)
  }
}

async function openFolder() {
  try {
    await openServiceFolder(props.svc.cwd)
  } catch (error) {
    console.error('No se pudo abrir la carpeta:', error)
  }
}

function editService() {
  emit('edit', props.svc)
}

function dependencyLabel(dependency: ServiceDependency): string {
  return `${dependency.name}${dependency.version ? `@${dependency.version}` : ''}`
}

async function copyDependencies() {
  const content = dependencies.value
    .map((dependency) => {
      const group = dependency.group ? `${dependency.group}:` : ''
      return `${group}${dependencyLabel(dependency)} (${dependencyKindLabels[dependency.kind] ?? dependency.kind}, ${dependency.manifest})`
    })
    .join('\n')

  try {
    await writeToClipboard(content)
    dependenciesCopied.value = true
    if (dependenciesCopyTimer !== undefined) window.clearTimeout(dependenciesCopyTimer)
    dependenciesCopyTimer = window.setTimeout(() => {
      dependenciesCopied.value = false
    }, 1500)
  } catch (error) {
    void logDashboardError('service.dependencies.copy', error, { notify: true })
  }
}

watch(
  () => props.svc.cwd,
  (cwd) => {
    dependenciesExpanded.value = false
    dependencyQuery.value = ''
    dependencyKind.value = 'all'
    void loadDependencies(cwd)
  },
  { immediate: true },
)

onMounted(() => {
  closeButton.value?.focus()
})

onBeforeUnmount(() => {
  if (copyTimer !== undefined) window.clearTimeout(copyTimer)
  if (dependenciesCopyTimer !== undefined) window.clearTimeout(dependenciesCopyTimer)
})
</script>

<template>
  <div class="info-overlay" @click="$emit('close')">
    <div
      class="info-panel"
      :style="accentStyle"
      role="dialog"
      aria-modal="true"
      aria-labelledby="service-info-title"
      @click.stop
    >
      <span class="panel-accent" aria-hidden="true"></span>

      <header class="info-header">
        <div class="info-title">
          <div class="info-icon">
            <component :is="categoryIconFor(svc.category)" :size="19" />
          </div>
          <div class="info-heading">
            <h3 id="service-info-title">{{ svc.name }}</h3>
            <p>{{ categoryLabel }} · {{ svc.preset }}</p>
          </div>
        </div>

        <div class="header-actions">
          <span class="status-pill" :class="statusClass">
            <span class="status-dot"></span>
            {{ statusText }}
          </span>
          <button
            ref="closeButton"
            class="close-btn"
            type="button"
            title="Cerrar"
            aria-label="Cerrar información del servicio"
            @click="$emit('close')"
          >
            <X :size="17" />
          </button>
        </div>
      </header>

      <div class="info-body">
        <section class="info-section">
          <h4 class="section-title"><Settings2 :size="13" /> General</h4>
          <div class="info-grid">
            <div class="info-item">
              <span class="item-label">Workspace</span>
              <strong>{{ workspaceName || 'Sin workspace' }}</strong>
            </div>
            <div class="info-item">
              <span class="item-label">Categoría</span>
              <strong>{{ categoryLabel }}</strong>
            </div>
            <div class="info-item">
              <span class="item-label">Preset</span>
              <strong>{{ svc.preset }}</strong>
            </div>
            <div v-if="gitBranch" class="info-item">
              <span class="item-label">Rama Git</span>
              <strong class="branch-value"><GitBranch :size="13" /> {{ gitBranch }}</strong>
            </div>
          </div>
        </section>

        <section v-if="health" class="info-section">
          <div class="section-heading">
            <h4 class="section-title"><HeartPulse :size="13" /> Salud</h4>
            <ServiceHealthBadge :health="health" :compact="false" />
          </div>

          <div class="info-grid">
            <div class="info-item">
              <span class="item-label">Estado</span>
              <strong>{{ healthStateLabel }}</strong>
            </div>
            <div class="info-item">
              <span class="item-label">Comprobación</span>
              <strong>{{ healthKind }}</strong>
            </div>
            <div class="info-item">
              <span class="item-label">Último check</span>
              <strong>{{ healthCheckedAt }}</strong>
            </div>
            <div class="info-item">
              <span class="item-label">Latencia</span>
              <strong>{{ healthLatency }}</strong>
            </div>
            <div v-if="health.consecutiveFailures > 0" class="info-item">
              <span class="item-label">Fallos consecutivos</span>
              <strong>{{ health.consecutiveFailures }}</strong>
            </div>
            <div v-if="health.target" class="info-item full">
              <span class="item-label">Objetivo</span>
              <strong class="mono">{{ health.target }}</strong>
            </div>
          </div>

          <p v-if="health.message" class="health-message" :class="`is-${health.state}`">
            {{ health.message }}
          </p>
        </section>

        <section v-if="hasDependencyInfo" class="info-section">
          <div class="section-heading deps-heading">
            <h4 class="section-title"><Package :size="13" /> Dependencias</h4>
            <div class="deps-head-actions">
              <button v-if="dependencies.length" class="text-btn deps-copy" type="button" @click="copyDependencies">
                <Check v-if="dependenciesCopied" :size="12" />
                <Copy v-else :size="12" />
                {{ dependenciesCopied ? 'Copiadas' : 'Copiar' }}
              </button>
              <button
                v-if="(dependencyReport?.total ?? 0) > previewDependencies.length || dependenciesExpanded"
                class="text-btn deps-toggle"
                type="button"
                @click="dependenciesExpanded = !dependenciesExpanded"
              >
                <component :is="dependenciesExpanded ? ChevronDown : ChevronRight" :size="12" />
                {{ dependenciesExpanded ? 'Ver menos' : 'Ver todas' }}
              </button>
            </div>
          </div>

          <div class="deps-summary">
            <span class="deps-count">{{ dependencySummary }}</span>
            <span v-if="dependencyReport?.framework" class="deps-framework">
              {{ dependencyReport.framework }}
            </span>
            <span
              v-for="manifest in dependencyReport?.manifests"
              :key="manifest"
              class="deps-manifest"
            >
              {{ manifest }}
            </span>
          </div>

          <div v-if="!dependencies.length" class="deps-empty">
            No se detectaron dependencias declaradas.
          </div>

          <div v-else-if="!dependenciesExpanded" class="deps-chips">
            <span
              v-for="dependency in previewDependencies"
              :key="`${dependency.ecosystem}-${dependency.name}`"
              class="dep-chip"
              :title="`${dependency.manifest} · ${dependencyKindLabels[dependency.kind] ?? dependency.kind}`"
            >
              <span class="dep-chip-name">{{ dependency.name }}</span>
              <span v-if="dependency.version" class="dep-chip-version">{{ dependency.version }}</span>
            </span>
            <button
              v-if="hiddenDependencyCount > 0"
              class="dep-chip dep-chip-more"
              type="button"
              @click="dependenciesExpanded = true"
            >
              +{{ hiddenDependencyCount }} más
            </button>
          </div>

          <template v-else>
            <div class="deps-toolbar">
              <div class="deps-search">
                <Search :size="13" aria-hidden="true" />
                <input
                  v-model="dependencyQuery"
                  type="text"
                  placeholder="Buscar librería..."
                  aria-label="Buscar librería"
                />
                <button
                  v-if="dependencyQuery"
                  type="button"
                  aria-label="Limpiar búsqueda"
                  @click="dependencyQuery = ''"
                >
                  <X :size="12" />
                </button>
              </div>

              <div class="deps-filters" role="group" aria-label="Filtrar dependencias">
                <button
                  v-for="filter in dependencyKindFilters"
                  :key="filter.value"
                  class="deps-filter"
                  :class="{ 'is-active': dependencyKind === filter.value }"
                  type="button"
                  :aria-pressed="dependencyKind === filter.value"
                  @click="dependencyKind = filter.value"
                >
                  {{ filter.label }}
                  <span>{{ filter.count }}</span>
                </button>
              </div>
            </div>

            <div v-if="dependencyGroups.length" class="deps-list">
              <div v-for="group in dependencyGroups" :key="group.kind" class="dep-group">
                <div class="dep-group-head">
                  <span>{{ group.label }}</span>
                  <span>{{ group.dependencies.length }}</span>
                </div>
                <div
                  v-for="dependency in group.dependencies"
                  :key="`${dependency.ecosystem}-${dependency.name}-${dependency.manifest}`"
                  class="dep-row"
                >
                  <div class="dep-main">
                    <code class="dep-name" :title="dependency.name">{{ dependency.name }}</code>
                    <span v-if="dependency.group" class="dep-group-id">{{ dependency.group }}</span>
                  </div>
                  <code v-if="dependency.version" class="dep-version">{{ dependency.version }}</code>
                  <span class="dep-source">{{ dependency.manifest }}</span>
                </div>
              </div>
            </div>
            <div v-else class="deps-empty">No hay dependencias que coincidan.</div>
          </template>

          <p v-if="dependencyReport?.warnings.length" class="deps-warning">
            <AlertTriangle :size="13" />
            {{ dependencyReport.warnings.join(' · ') }}
          </p>
          <p v-if="dependencyReport?.truncated" class="deps-warning">
            <AlertTriangle :size="13" />
            Se muestran las primeras 200 dependencias.
          </p>
        </section>

        <section class="info-section">
          <h4 class="section-title"><Terminal :size="13" /> Ejecución</h4>

          <div class="info-item">
            <span class="item-label"><FolderOpen :size="12" /> Carpeta de trabajo</span>
            <div class="path-row">
              <code class="path-value">{{ svc.cwd }}</code>
              <button
                class="inline-action"
                type="button"
                title="Abrir carpeta"
                aria-label="Abrir carpeta de trabajo"
                @click="openFolder"
              >
                <FolderOpen :size="14" />
              </button>
            </div>
          </div>

          <div class="info-item">
            <span class="item-label"><Terminal :size="12" /> Comando de inicio</span>
            <div class="code-row">
              <pre class="code-value">{{ command }}</pre>
              <button
                class="inline-action"
                type="button"
                :title="commandCopied ? 'Comando copiado' : 'Copiar comando'"
                :aria-label="commandCopied ? 'Comando copiado' : 'Copiar comando'"
                @click="copyCommand"
              >
                <Check v-if="commandCopied" :size="14" />
                <Copy v-else :size="14" />
              </button>
            </div>
          </div>

          <div v-if="svc.args?.length" class="info-item">
            <span class="item-label">Argumentos</span>
            <div class="args-list">
              <code v-for="(arg, index) in svc.args" :key="`${arg}-${index}`">{{ arg }}</code>
            </div>
          </div>

          <div v-if="svc.setupCommand" class="info-item">
            <span class="item-label">Comando de preparación</span>
            <pre class="code-value setup-value">{{ svc.setupCommand }}</pre>
          </div>
        </section>

        <section class="info-section">
          <h4 class="section-title"><Globe :size="13" /> Red</h4>
          <div class="info-grid">
            <div class="info-item full">
              <span class="item-label">URL</span>
              <div class="url-row">
                <a :href="serviceUrl" target="_blank" rel="noreferrer" class="url-value">{{ serviceUrl }}</a>
                <button
                  class="inline-action"
                  type="button"
                  :title="copied ? 'URL copiada' : 'Copiar URL'"
                  :aria-label="copied ? 'URL copiada' : 'Copiar URL'"
                  @click="copyUrl"
                >
                  <Check v-if="copied" :size="14" />
                  <Copy v-else :size="14" />
                </button>
                <button
                  class="inline-action"
                  type="button"
                  title="Abrir en el navegador"
                  aria-label="Abrir en el navegador"
                  @click="openInBrowser"
                >
                  <ExternalLink :size="14" />
                </button>
              </div>
            </div>
            <div class="info-item">
              <span class="item-label">Puerto</span>
              <strong class="mono">{{ port }}</strong>
            </div>
          </div>
        </section>

        <section v-if="envEntries.length" class="info-section">
          <div class="section-heading">
            <h4 class="section-title"><Variable :size="13" /> Variables de entorno</h4>
            <button class="text-btn" type="button" @click="showEnvValues = !showEnvValues">
              {{ showEnvValues ? 'Ocultar valores' : 'Mostrar valores' }}
            </button>
          </div>
          <div class="env-list">
            <div v-for="[key, value] in envEntries" :key="key" class="env-row">
              <code class="env-key">{{ key }}</code>
              <code class="env-value">{{ showEnvValues ? value || '—' : '••••••' }}</code>
            </div>
          </div>
        </section>

        <section v-if="usage" class="info-section">
          <h4 class="section-title"><Activity :size="13" /> Rendimiento</h4>
          <div class="info-grid">
            <div class="info-item">
              <span class="item-label"><Cpu :size="12" /> CPU</span>
              <strong class="mono">{{ cpuUsageLabel }}</strong>
            </div>
            <div class="info-item">
              <span class="item-label"><MemoryStick :size="12" /> RAM</span>
              <strong class="mono">{{ memoryUsageLabel }}</strong>
            </div>
            <div class="info-item">
              <span class="item-label">Procesos</span>
              <strong>{{ processUsageLabel }}</strong>
            </div>
          </div>
        </section>

        <section v-if="hasRuntimeDetails" class="info-section">
          <h4 class="section-title"><Activity :size="13" /> Estado técnico</h4>
          <div class="info-grid">
            <div v-if="runtime?.pid != null" class="info-item">
              <span class="item-label">PID</span>
              <strong class="mono">{{ runtime.pid }}</strong>
            </div>
            <div v-if="runtime?.exitCode != null" class="info-item">
              <span class="item-label">Código de salida</span>
              <strong class="mono">{{ runtime.exitCode }}</strong>
            </div>
            <div v-if="runtime?.error" class="info-item full">
              <span class="item-label">Último error</span>
              <pre class="runtime-error">{{ runtime.error }}</pre>
            </div>
          </div>
        </section>
      </div>

      <footer class="info-footer">
        <button class="btn btn-ghost" type="button" @click="$emit('close')">
          Cerrar
        </button>
        <div class="footer-actions">
          <button v-if="gitBranch" class="btn btn-secondary" type="button" @click="$emit('git', svc)">
            <GitBranch :size="15" />
            Git
          </button>
          <button class="btn btn-secondary" type="button" @click="$emit('move', svc)">
            <FolderInput :size="15" />
            Mover
          </button>
          <button class="btn btn-secondary" type="button" @click="$emit('logs', svc.id)">
            <ScrollText :size="15" />
            Ver logs
          </button>
          <button class="btn btn-primary" type="button" @click="editService">
            <Pencil :size="15" />
            Editar
          </button>
        </div>
      </footer>
    </div>
  </div>
</template>

<style scoped>
.info-overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  background: var(--overlay);
  backdrop-filter: blur(6px);
}

.info-panel {
  position: relative;
  display: flex;
  flex-direction: column;
  width: 100%;
  max-width: 640px;
  max-height: 90vh;
  overflow: hidden;
  border: 1px solid var(--border);
  border-radius: var(--radius-xl);
  background: var(--surface);
  box-shadow: var(--shadow-lg);
  animation: infoModalIn 0.18s ease-out;
}

.panel-accent {
  position: absolute;
  inset: 0 0 auto;
  z-index: 3;
  height: 3px;
  background: var(--service-accent, var(--primary));
}

.info-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 14px;
  flex-shrink: 0;
  padding: 20px 20px 16px;
  border-bottom: 1px solid var(--border);
}

.info-title {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.info-icon {
  display: grid;
  place-items: center;
  width: 42px;
  height: 42px;
  flex-shrink: 0;
  border: 1px solid var(--border);
  border-radius: 13px;
  background: var(--surface-2);
  color: var(--service-accent, var(--primary));
}

.info-heading {
  min-width: 0;
}

.info-heading h3 {
  margin: 0;
  overflow: hidden;
  color: var(--text);
  font-size: 17px;
  letter-spacing: -0.01em;
  line-height: 1.3;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.info-heading p {
  margin: 3px 0 0;
  color: var(--text-muted);
  font-size: 12px;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.close-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  padding: 0;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface);
  color: var(--text-muted);
  cursor: pointer;
}

.close-btn:hover {
  border-color: var(--border-strong);
  background: var(--surface-2);
  color: var(--text);
}

.info-body {
  display: flex;
  flex-direction: column;
  gap: 20px;
  min-height: 0;
  padding: 20px;
  overflow: auto;
}

.info-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.info-section + .info-section {
  padding-top: 18px;
  border-top: 1px solid var(--border);
}

.section-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 7px;
  margin: 0;
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.text-btn {
  padding: 0;
  border: 0;
  background: transparent;
  color: var(--primary);
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
}

.text-btn:hover {
  text-decoration: underline;
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
}

.health-message {
  margin: 0;
  padding: 9px 10px;
  border: 1px solid var(--danger);
  border-radius: var(--radius-sm);
  background: var(--danger-soft);
  color: var(--danger);
  font-size: 11px;
  line-height: 1.45;
  overflow-wrap: anywhere;
}

.health-message.is-degraded {
  border-color: var(--warning);
  background: var(--warning-soft);
  color: var(--warning);
}

.info-item {
  display: flex;
  flex-direction: column;
  gap: 5px;
  min-width: 0;
}

.info-item.full {
  grid-column: 1 / -1;
}

.item-label {
  display: flex;
  align-items: center;
  gap: 5px;
  color: var(--text-subtle);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.info-item strong {
  overflow-wrap: anywhere;
  color: var(--text);
  font-size: 13px;
  font-weight: 600;
}

.branch-value {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  color: var(--info) !important;
  font-family: var(--mono);
  font-size: 12px !important;
}

.mono {
  font-family: var(--mono);
}

.path-row,
.code-row,
.url-row {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.path-value,
.code-value,
.runtime-error {
  min-width: 0;
  flex: 1;
  overflow: auto;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  color: var(--text);
  font-family: var(--mono);
  font-size: 11px;
  line-height: 1.5;
}

.path-value {
  padding: 8px 10px;
  overflow-wrap: anywhere;
}

.code-value {
  margin: 0;
  padding: 9px 10px;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.setup-value {
  color: var(--text-muted);
}

.inline-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  flex-shrink: 0;
  padding: 0;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface);
  color: var(--text-muted);
  cursor: pointer;
}

.inline-action:hover {
  border-color: var(--border-strong);
  background: var(--surface-2);
  color: var(--text);
}

.args-list {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.args-list code {
  padding: 3px 7px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--surface-2);
  color: var(--text-muted);
  font-size: 11px;
}

.url-value {
  min-width: 0;
  flex: 1;
  overflow: hidden;
  color: var(--primary);
  font-size: 12px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.env-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.env-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 7px 9px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
}

.env-key,
.env-value {
  overflow: hidden;
  font-family: var(--mono);
  font-size: 11px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.env-key {
  color: var(--text);
}

.env-value {
  color: var(--text-muted);
}

.runtime-error {
  margin: 0;
  padding: 9px 10px;
  border-color: var(--danger);
  background: var(--danger-soft);
  color: var(--danger);
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.deps-heading {
  align-items: center;
}

.deps-head-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.deps-copy,
.deps-toggle {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.deps-summary {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

.deps-count {
  color: var(--text);
  font-size: 11px;
  font-weight: 700;
}

.deps-framework,
.deps-manifest {
  padding: 3px 7px;
  border-radius: 999px;
  font-size: 9px;
  font-weight: 600;
}

.deps-framework {
  background: var(--info-soft);
  color: var(--info);
}

.deps-manifest {
  background: var(--surface-2);
  color: var(--text-muted);
  font-family: var(--mono);
}

.deps-chips {
  display: flex;
  align-items: center;
  gap: 5px;
  flex-wrap: wrap;
}

.dep-chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  max-width: 100%;
  padding: 4px 8px;
  border: 1px solid var(--border);
  border-radius: 999px;
  background: var(--surface-2);
  font-family: var(--mono);
  font-size: 10px;
}

.dep-chip-name {
  overflow: hidden;
  color: var(--text);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dep-chip-version {
  flex-shrink: 0;
  color: var(--text-subtle);
}

.dep-chip-more {
  border-color: var(--primary);
  background: var(--primary-soft);
  color: var(--primary);
  font-weight: 700;
  cursor: pointer;
}

.dep-chip-more:hover {
  filter: brightness(0.98);
}

.deps-toolbar {
  display: flex;
  align-items: center;
  gap: 9px;
  flex-wrap: wrap;
}

.deps-search {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 180px;
  flex: 1;
  padding: 0 8px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface);
  color: var(--text-subtle);
}

.deps-search:focus-within {
  border-color: var(--primary);
  box-shadow: var(--focus-ring);
}

.deps-search input {
  min-width: 0;
  flex: 1;
  padding: 7px 0;
  border: 0;
  outline: 0;
  background: transparent;
  color: var(--text);
  font-size: 11px;
}

.deps-search button {
  display: inline-flex;
  padding: 1px;
  border: 0;
  background: transparent;
  color: var(--text-subtle);
  cursor: pointer;
}

.deps-filters {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-wrap: wrap;
}

.deps-filter {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 7px;
  border: 1px solid var(--border);
  border-radius: 999px;
  background: var(--surface);
  color: var(--text-muted);
  font-size: 9px;
  font-weight: 600;
  cursor: pointer;
}

.deps-filter span {
  color: var(--text-subtle);
  font-family: var(--mono);
}

.deps-filter:hover {
  border-color: var(--border-strong);
  color: var(--text);
}

.deps-filter.is-active {
  border-color: var(--primary);
  background: var(--primary-soft);
  color: var(--primary);
}

.deps-filter.is-active span {
  color: var(--primary);
}

.deps-list {
  max-height: 240px;
  overflow: auto;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
}

.dep-group + .dep-group {
  border-top: 1px solid var(--border);
}

.dep-group-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 7px 9px;
  background: var(--surface);
  color: var(--text-subtle);
  font-size: 9px;
  font-weight: 800;
  letter-spacing: 0.07em;
  text-transform: uppercase;
}

.dep-group-head span:last-child {
  font-family: var(--mono);
  letter-spacing: 0;
}

.dep-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto auto;
  align-items: center;
  gap: 8px;
  padding: 6px 9px;
  border-top: 1px solid var(--border);
}

.dep-main {
  display: flex;
  flex-direction: column;
  min-width: 0;
  gap: 1px;
}

.dep-name {
  overflow: hidden;
  color: var(--text);
  font-family: var(--mono);
  font-size: 10px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dep-group-id {
  overflow: hidden;
  color: var(--text-subtle);
  font-family: var(--mono);
  font-size: 9px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dep-version {
  padding: 2px 5px;
  border-radius: 4px;
  background: var(--surface);
  color: var(--text-muted);
  font-family: var(--mono);
  font-size: 9px;
  white-space: nowrap;
}

.dep-source {
  color: var(--text-subtle);
  font-family: var(--mono);
  font-size: 9px;
  white-space: nowrap;
}

.deps-empty {
  padding: 14px;
  border: 1px dashed var(--border);
  border-radius: var(--radius-sm);
  color: var(--text-subtle);
  font-size: 11px;
  text-align: center;
}

.deps-warning {
  display: flex;
  align-items: flex-start;
  gap: 5px;
  margin: 0;
  color: var(--warning);
  font-size: 10px;
  line-height: 1.4;
}

.deps-warning svg {
  flex-shrink: 0;
  margin-top: 1px;
}

.info-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-shrink: 0;
  padding: 14px 20px;
  border-top: 1px solid var(--border);
  background: var(--surface);
}

.footer-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

@media (max-width: 520px) {
  .info-overlay {
    padding: 12px;
  }

  .info-panel {
    max-height: 94vh;
  }

  .info-header {
    align-items: flex-start;
    padding: 18px 16px 14px;
  }

  .info-title {
    align-items: flex-start;
  }

  .header-actions {
    flex-direction: column;
    align-items: flex-end;
  }

  .info-body {
    padding: 16px;
  }

  .info-grid {
    grid-template-columns: 1fr;
  }

  .deps-toolbar {
    align-items: stretch;
    flex-direction: column;
  }

  .deps-search {
    min-width: 0;
  }

  .dep-row {
    grid-template-columns: minmax(0, 1fr) auto;
  }

  .dep-source {
    grid-column: 1 / -1;
  }

  .info-footer {
    align-items: stretch;
    flex-direction: column;
    padding: 12px 16px;
  }

  .footer-actions {
    justify-content: flex-end;
  }

  .info-footer > .btn:first-child {
    align-self: flex-start;
  }
}

@keyframes infoModalIn {
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
