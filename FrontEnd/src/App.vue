<script setup lang="ts">
import { computed, markRaw, onMounted, onUnmounted, ref, watch } from 'vue'
import { confirm } from '@tauri-apps/plugin-dialog'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Activity, AlertTriangle, Box, LayoutGrid, Monitor, Plus, ScrollText, Search, SearchX, Server, ServerOff, X } from '@lucide/vue'
import { logDashboardError, logDashboardWarn } from './composables/useDashboardLog'
import { useGitBranches } from './composables/useGitBranches'
import { useNotifications } from './composables/useNotifications'
import { useServiceUsage } from './composables/useServiceUsage'
import { useServices, useSettings } from './composables/useServices'
import { useViewMode } from './composables/useViewMode'
import { useWorkspaces } from './composables/useWorkspaces'
import DashboardLogModal from './components/DashboardLogModal.vue'
import ErrorToast from './components/ErrorToast.vue'
import GitModal from './components/GitModal.vue'
import GlobalContextMenu from './components/GlobalContextMenu.vue'
import HeaderBar from './components/HeaderBar.vue'
import LogDrawer from './components/LogDrawer.vue'
import MoveServiceModal from './components/MoveServiceModal.vue'
import ServiceContextMenu from './components/ServiceContextMenu.vue'
import ServiceDiagnosticsModal from './components/ServiceDiagnosticsModal.vue'
import ServiceForm from './components/ServiceForm.vue'
import ServiceGrid from './components/ServiceGrid.vue'
import ServiceInfoModal from './components/ServiceInfoModal.vue'
import ServiceWizard from './components/ServiceWizard.vue'
import SettingsModal from './components/SettingsModal.vue'
import ViewModeToggle from './components/ViewModeToggle.vue'
import WorkspaceManager from './components/WorkspaceManager.vue'
import WorkspaceSidebar from './components/WorkspaceSidebar.vue'
import type { AppNotification, Service, ServiceInput, Settings, WorkspaceStats, WorkspaceUsage } from './types'

const { services, load, refreshStatuses, startAll, stopAll, getStatus, getRuntime, start, stop, restart, remove, update, add, move, reorder, dismissError } = useServices()
const { workspaces, activeWorkspaceId, load: loadWorkspaces, setActive: setActiveWorkspace } = useWorkspaces()
const { viewMode } = useViewMode()
const { load: loadGitBranches, getBranch } = useGitBranches()
const { getUsage, getHistory: getUsageHistory, refresh: refreshUsage, reset: resetUsage } = useServiceUsage()
const { load: loadSettings } = useSettings()
const { notifications, dismiss: dismissNotification } = useNotifications()

const showLogDrawer = ref(false)
const showDashboardLog = ref(false)
const showWorkspaceManager = ref(false)
const showSettings = ref(false)
const showGit = ref(false)
const gitService = ref<Service | null>(null)
const selectedServiceId = ref<string | null>(null)
const infoServiceId = ref<string | null>(null)
const showServiceForm = ref(false)
const editingService = ref<Service | null>(null)
const movingService = ref<Service | null>(null)
const diagnosticsService = ref<Service | null>(null)
const contextMenu = ref<{
  serviceId: string
  x: number
  y: number
  selectedText: string
} | null>(null)
const globalContextMenu = ref<{
  x: number
  y: number
  target: HTMLElement | null
} | null>(null)
const draggedServiceId = ref<string | null>(null)
const confirmExit = ref(true)
const filterCategory = ref('')
const filterText = ref('')
const showOnlyActive = ref(false)
let statusTimer: number | undefined
let usageTimer: number | undefined
let unlistenClose: UnlistenFn | undefined

const activeServices = computed(() =>
  services.value.filter((service) => service.workspaceId === activeWorkspaceId.value),
)

watch(activeServices, (currentServices) => {
  void loadGitBranches(currentServices)
}, { immediate: true })

const runningCount = computed(() =>
  activeServices.value.filter((service) => ['running', 'external'].includes(getStatus(service.id))).length,
)

const workspaceStats = computed<Record<string, WorkspaceStats>>(() => {
  const stats: Record<string, WorkspaceStats> = {}
  for (const workspace of workspaces.value) {
    const workspaceServices = services.value.filter((service) => service.workspaceId === workspace.id)
    stats[workspace.id] = {
      total: workspaceServices.length,
      running: workspaceServices.filter((service) =>
        ['running', 'external'].includes(getStatus(service.id)),
      ).length,
      errors: workspaceServices.filter((service) => getStatus(service.id) === 'error').length,
    }
  }
  return stats
})

const workspaceUsage = computed<Record<string, WorkspaceUsage>>(() => {
  const usage: Record<string, WorkspaceUsage> = {}
  for (const workspace of workspaces.value) {
    let cpuPercent = 0
    let memoryBytes = 0
    let processCount = 0

    for (const service of services.value) {
      if (service.workspaceId !== workspace.id) continue
      const point = getUsage(service.id)
      if (!point) continue
      cpuPercent += point.cpuPercent
      memoryBytes += point.memoryBytes
      processCount += point.processCount
    }

    usage[workspace.id] = {
      cpuPercent: Math.min(100, Math.max(0, cpuPercent)),
      memoryBytes,
      processCount,
    }
  }
  return usage
})

const activeWorkspaceName = computed(
  () => workspaces.value.find((workspace) => workspace.id === activeWorkspaceId.value)?.name ?? 'Workspace',
)

const activeWorkspaceUsage = computed<WorkspaceUsage>(
  () => workspaceUsage.value[activeWorkspaceId.value] ?? {
    cpuPercent: 0,
    memoryBytes: 0,
    processCount: 0,
  },
)

const categoryFilters = computed(() => [
  { value: '', label: 'Todas', icon: markRaw(LayoutGrid), count: activeServices.value.length },
  {
    value: 'frontend',
    label: 'Frontend',
    icon: markRaw(Monitor),
    count: activeServices.value.filter((service) => service.category === 'frontend').length,
  },
  {
    value: 'backend',
    label: 'Backend',
    icon: markRaw(Server),
    count: activeServices.value.filter((service) => service.category === 'backend').length,
  },
  {
    value: 'custom',
    label: 'Personalizado',
    icon: markRaw(Box),
    count: activeServices.value.filter((service) => service.category === 'custom').length,
  },
])

const hasActiveFilters = computed(() =>
  Boolean(filterText.value || filterCategory.value || showOnlyActive.value),
)

const filteredServices = computed(() =>
  activeServices.value.filter((service) => {
    if (filterCategory.value && service.category !== filterCategory.value) return false
    if (filterText.value && !service.name.toLowerCase().includes(filterText.value.toLowerCase())) return false
    if (showOnlyActive.value) {
      const status = getStatus(service.id)
      if (status !== 'running' && status !== 'external') return false
    }
    return true
  }),
)

const selectedServiceName = computed(
  () => services.value.find((service) => service.id === selectedServiceId.value)?.name ?? '',
)

const selectedServiceStatus = computed(
  () => (selectedServiceId.value ? getStatus(selectedServiceId.value) : 'stopped'),
)

const selectedServiceError = computed(
  () => (selectedServiceId.value ? getRuntime(selectedServiceId.value)?.error : undefined),
)

const infoService = computed(
  () => services.value.find((service) => service.id === infoServiceId.value) ?? null,
)

const infoWorkspaceName = computed(
  () => workspaces.value.find((workspace) => workspace.id === infoService.value?.workspaceId)?.name,
)

const infoStatus = computed(
  () => (infoService.value ? getStatus(infoService.value.id) : 'stopped'),
)

const infoRuntime = computed(
  () => (infoService.value ? getRuntime(infoService.value.id) : undefined),
)

const contextMenuService = computed(
  () => services.value.find((service) => service.id === contextMenu.value?.serviceId) ?? null,
)

const contextMenuStatus = computed(
  () => (contextMenuService.value ? getStatus(contextMenuService.value.id) : 'stopped'),
)

const contextMenuRuntime = computed(
  () => (contextMenuService.value ? getRuntime(contextMenuService.value.id) : undefined),
)

const failedServices = computed(
  () => activeServices.value.filter((service) => getStatus(service.id) === 'error'),
)

function clearFilters() {
  filterText.value = ''
  filterCategory.value = ''
  showOnlyActive.value = false
}

const USAGE_STATUSES = new Set<string>(['starting', 'running', 'stopping', 'external'])

function usageServiceIds() {
  return services.value
    .filter((service) => USAGE_STATUSES.has(getStatus(service.id)))
    .map((service) => service.id)
}

function pollUsage() {
  if (document.hidden) return
  void refreshUsage(usageServiceIds())
}

function handleVisibilityChange() {
  if (document.hidden) {
    resetUsage()
    return
  }
  void refreshUsage(usageServiceIds())
}

async function switchWorkspace(id: string) {
  if (id === activeWorkspaceId.value) return
  try {
    await setActiveWorkspace(id)
  } catch (error) {
    console.error('Error cambiando de workspace:', error)
    void logDashboardError('workspace.switch', error, {
      title: 'No se pudo cambiar de workspace',
      notify: true,
      throttleMs: 3000,
    })
  }
}

function openWorkspaceManager() {
  showWorkspaceManager.value = true
}

function closeWorkspaceManager() {
  showWorkspaceManager.value = false
}

function openSettings() {
  closeDashboardLog()
  showSettings.value = true
}

function closeSettings() {
  showSettings.value = false
}

function handleSettingsSaved(next: Settings) {
  confirmExit.value = next.confirmOnExit
}

function openDashboardLog() {
  closeSettings()
  closeServiceInfo()
  closeLogDrawer()
  closeContextMenu()
  closeGlobalContextMenu()
  showDashboardLog.value = true
}

function closeDashboardLog() {
  showDashboardLog.value = false
}

function openLogFromNotification(item: AppNotification) {
  dismissNotification(item.id)
  openDashboardLog()
}

async function handleWorkspaceRemoved() {
  closeWorkspaceManager()
  clearFilters()
  await load()
  await refreshStatuses()
}

function openLogs(id: string) {
  closeServiceInfo()
  selectedServiceId.value = id
  showLogDrawer.value = true
}

function closeLogDrawer() {
  showLogDrawer.value = false
  selectedServiceId.value = null
}

function newService() {
  editingService.value = null
  showServiceForm.value = true
}

function openServiceInfo(service: Service) {
  infoServiceId.value = service.id
}

function closeServiceInfo() {
  infoServiceId.value = null
}

function openGit(service?: Service | null) {
  closeServiceInfo()
  closeContextMenu()
  closeGlobalContextMenu()
  gitService.value = service ?? null
  showGit.value = true
}

function closeGit() {
  showGit.value = false
  gitService.value = null
  void loadGitBranches(activeServices.value)
}

function refreshGitBranches() {
  void loadGitBranches(activeServices.value)
}

function openMoveService(service: Service) {
  closeServiceInfo()
  movingService.value = service
}

function closeMoveService() {
  movingService.value = null
}

function handleServiceMoved() {
  closeMoveService()
}

function handleServiceDragStart(service: Service) {
  closeContextMenu()
  closeGlobalContextMenu()
  draggedServiceId.value = service.id
}

function handleServiceDragEnd() {
  draggedServiceId.value = null
}

async function handleServiceDrop(targetId: string, after: boolean) {
  const draggedId = draggedServiceId.value
  draggedServiceId.value = null
  if (!draggedId || draggedId === targetId) return

  const source = services.value.find((service) => service.id === draggedId)
  if (!source) return

  const workspaceServices = services.value.filter(
    (service) => service.workspaceId === source.workspaceId,
  )
  const ids = workspaceServices.map((service) => service.id)
  const sourceIndex = ids.indexOf(draggedId)
  const targetIndex = ids.indexOf(targetId)
  if (sourceIndex < 0 || targetIndex < 0) return

  const next = ids.filter((id) => id !== draggedId)
  const nextTargetIndex = next.indexOf(targetId)
  next.splice(nextTargetIndex + (after ? 1 : 0), 0, draggedId)

  if (next.every((id, index) => id === ids[index])) return

  try {
    await reorder(source.workspaceId, next)
  } catch (error) {
    void logDashboardError('service.reorder', error, {
      title: 'No se pudo guardar el orden de los servicios',
      notify: true,
    })
  }
}

async function handleServiceDropOnWorkspace(workspaceId: string) {
  const draggedId = draggedServiceId.value
  draggedServiceId.value = null
  if (!draggedId) return

  const service = services.value.find((item) => item.id === draggedId)
  if (!service || service.workspaceId === workspaceId) return

  try {
    await move(draggedId, workspaceId)
  } catch (error) {
    void logDashboardError('service.moveDrag', error, {
      title: 'No se pudo mover el servicio',
      notify: true,
      serviceId: service.id,
      serviceName: service.name,
    })
  }
}

function selectionInsideService(event: MouseEvent): string {
  const selection = window.getSelection()
  if (!selection || selection.isCollapsed || selection.rangeCount === 0) return ''

  const target =
    event.target instanceof Element
      ? event.target.closest('.card, .service-row')
      : null
  if (!target) return ''

  const range = selection.getRangeAt(0)
  if (!target.contains(range.commonAncestorContainer)) return ''

  const text = selection.toString()
  return text.trim() ? text : ''
}

function openContextMenu(service: Service, event: MouseEvent) {
  globalContextMenu.value = null
  contextMenu.value = {
    serviceId: service.id,
    x: event.clientX,
    y: event.clientY,
    selectedText: selectionInsideService(event),
  }
}

function closeContextMenu() {
  contextMenu.value = null
}

function closeGlobalContextMenu() {
  globalContextMenu.value = null
}

function handleAppContextMenu(event: MouseEvent) {
  if (event.defaultPrevented) return

  event.preventDefault()
  closeContextMenu()
  globalContextMenu.value = {
    x: event.clientX,
    y: event.clientY,
    target: event.target instanceof HTMLElement ? event.target : null,
  }
}

function openDiagnostics(service: Service) {
  closeContextMenu()
  diagnosticsService.value = service
}

function closeDiagnostics() {
  diagnosticsService.value = null
}

function openLogsFromDiagnostics(id: string) {
  closeDiagnostics()
  openLogs(id)
}

function editService(service: Service) {
  closeServiceInfo()
  editingService.value = service
  showServiceForm.value = true
}

function closeServiceForm() {
  showServiceForm.value = false
  editingService.value = null
}

async function handleSave(input: ServiceInput) {
  try {
    if (editingService.value) {
      await update(editingService.value.id, input)
    } else {
      await add(input)
    }
    closeServiceForm()
  } catch (error) {
    void logDashboardError('service.save', error, {
      title: editingService.value ? 'No se pudieron guardar los cambios' : 'No se pudo crear el servicio',
      notify: true,
      serviceId: editingService.value?.id,
      serviceName: editingService.value?.name,
    })
  }
}

async function confirmDelete(id: string) {
  if (await confirm('¿Eliminar este servicio? Se detendrá el proceso y se eliminarán sus logs.')) {
    const service = services.value.find((item) => item.id === id)
    try {
      if (infoServiceId.value === id) closeServiceInfo()
      if (gitService.value?.id === id) closeGit()
      await remove(id)
    } catch (error) {
      void logDashboardError('service.delete', error, {
        title: 'No se pudo eliminar el servicio',
        notify: true,
        serviceId: id,
        serviceName: service?.name,
      })
    }
  }
}

function getStatusClass(status: string) {
  const map: Record<string, string> = {
    running: 'status-running',
    stopped: 'status-stopped',
    starting: 'status-starting',
    stopping: 'status-stopping',
    error: 'status-error',
    external: 'status-external',
  }
  return map[status] || 'status-stopped'
}

function categoryLabel(category: string) {
  const map: Record<string, string> = { frontend: 'Frontend', backend: 'Backend', custom: 'Personalizado' }
  return map[category] || category
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key !== 'Escape') return
  if (globalContextMenu.value) {
    closeGlobalContextMenu()
    return
  }
  if (contextMenu.value) {
    closeContextMenu()
    return
  }
  if (showGit.value) {
    closeGit()
    return
  }
  if (movingService.value) {
    closeMoveService()
    return
  }
  if (diagnosticsService.value) {
    closeDiagnostics()
    return
  }
  if (showDashboardLog.value) {
    closeDashboardLog()
    return
  }
  if (showSettings.value) {
    closeSettings()
    return
  }
  if (showWorkspaceManager.value) {
    closeWorkspaceManager()
    return
  }
  if (showServiceForm.value) {
    closeServiceForm()
    return
  }
  if (infoServiceId.value) {
    closeServiceInfo()
    return
  }
  if (showLogDrawer.value) closeLogDrawer()
}

watch(activeWorkspaceId, () => {
  clearFilters()
  closeLogDrawer()
  closeServiceInfo()
  closeServiceForm()
  closeGit()
  pollUsage()
})

onMounted(async () => {
  window.addEventListener('keydown', handleKeydown)
  document.addEventListener('contextmenu', handleAppContextMenu)
  document.addEventListener('visibilitychange', handleVisibilityChange)
  await loadWorkspaces()
  await load()
  await refreshStatuses()
  const settings = await loadSettings()
  confirmExit.value = settings?.confirmOnExit ?? true

  try {
    const currentWindow = getCurrentWindow()
    unlistenClose = await currentWindow.onCloseRequested(async (event) => {
      if (!confirmExit.value) return
      event.preventDefault()
      const message = runningCount.value > 0
        ? `¿Cerrar Cardumen Deck? Hay ${runningCount.value} servicios activos.`
        : '¿Cerrar Cardumen Deck?'
      if (await confirm(message)) await currentWindow.destroy()
    })
  } catch (error) {
    void logDashboardWarn('window.close', error)
  }

  statusTimer = window.setInterval(refreshStatuses, 2000)
  await refreshUsage(usageServiceIds())
  usageTimer = window.setInterval(pollUsage, 1000)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
  document.removeEventListener('contextmenu', handleAppContextMenu)
  document.removeEventListener('visibilitychange', handleVisibilityChange)
  if (statusTimer !== undefined) window.clearInterval(statusTimer)
  if (usageTimer !== undefined) window.clearInterval(usageTimer)
  unlistenClose?.()
  resetUsage()
})
</script>

<template>
  <div class="app">
    <HeaderBar
      :services-count="activeServices.length"
      :running-count="runningCount"
      :workspace-name="activeWorkspaceName"
      :cpu-percent="activeWorkspaceUsage.cpuPercent"
      :memory-bytes="activeWorkspaceUsage.memoryBytes"
      :process-count="activeWorkspaceUsage.processCount"
      @start-all="startAll(activeServices)"
      @stop-all="stopAll(activeServices)"
      @new-service="newService"
      @open-settings="openSettings"
      @open-git="openGit()"
      @open-dashboard-log="openDashboardLog"
    />

    <div class="app-body">
      <WorkspaceSidebar
        :workspaces="workspaces"
        :active-workspace-id="activeWorkspaceId"
        :stats="workspaceStats"
        :usage="workspaceUsage"
        :dragging="Boolean(draggedServiceId)"
        @switch="switchWorkspace"
        @manage="openWorkspaceManager"
        @drop-service="handleServiceDropOnWorkspace"
      />

      <main class="main-content">
      <section class="toolbar" aria-label="Filtros de servicios">
        <div class="search-box">
          <Search class="search-icon" :size="16" aria-hidden="true" />
          <input
            v-model="filterText"
            class="search-input"
            type="text"
            placeholder="Buscar servicio..."
            aria-label="Buscar servicio"
          />
          <button
            v-if="filterText"
            class="search-clear"
            type="button"
            aria-label="Limpiar búsqueda"
            @click="filterText = ''"
          >
            <X :size="14" />
          </button>
        </div>

        <div class="category-chips" role="group" aria-label="Filtrar por categoría">
          <button
            v-for="filter in categoryFilters"
            :key="filter.value"
            class="chip"
            :class="{ 'is-active': filterCategory === filter.value }"
            type="button"
            :aria-pressed="filterCategory === filter.value"
            @click="filterCategory = filter.value"
          >
            <component :is="filter.icon" :size="14" />
            {{ filter.label }}
            <span class="chip-count">{{ filter.count }}</span>
          </button>
        </div>

        <label class="switch">
          <input type="checkbox" v-model="showOnlyActive" />
          <span class="switch-track">
            <span class="switch-thumb"></span>
          </span>
          <Activity :size="14" />
          <span>Solo activos</span>
        </label>

        <ViewModeToggle />
      </section>

      <div class="results-meta">
        <span><strong>{{ filteredServices.length }}</strong> de {{ activeServices.length }} servicios</span>
        <button v-if="hasActiveFilters" class="btn btn-ghost btn-sm" type="button" @click="clearFilters">
          <X :size="14" />
          Limpiar filtros
        </button>
      </div>

      <section v-if="failedServices.length > 0" class="error-banner">
        <AlertTriangle :size="18" />
        <div class="error-banner-content">
          <strong>{{ failedServices.length }} servicio{{ failedServices.length === 1 ? '' : 's' }} con errores</strong>
          <span>
            {{ failedServices[0].name }}:
            {{ getRuntime(failedServices[0].id)?.error || 'Revisa los logs para más detalles.' }}
          </span>
        </div>
        <button class="btn btn-secondary btn-sm" type="button" @click="openLogs(failedServices[0].id)">
          <ScrollText :size="14" />
          Ver logs
        </button>
        <button
          class="btn btn-ghost btn-icon"
          type="button"
          title="Descartar error"
          aria-label="Descartar error"
          @click="dismissError(failedServices[0].id)"
        >
          <X :size="15" />
        </button>
      </section>

      <ServiceGrid
        :services="filteredServices"
        :view-mode="viewMode"
        :get-status="getStatus"
        :get-runtime="getRuntime"
        :get-git-branch="getBranch"
        :get-status-class="getStatusClass"
        :category-label="categoryLabel"
        :get-usage="getUsage"
        :get-usage-history="getUsageHistory"
        :dragging-id="draggedServiceId"
        @start="start"
        @stop="stop"
        @restart="restart"
        @logs="openLogs"
        @info="openServiceInfo"
        @move="openMoveService"
        @edit="editService"
        @delete="confirmDelete"
        @contextmenu="openContextMenu"
        @git="openGit"
        @drag-start="handleServiceDragStart"
        @drag-end="handleServiceDragEnd"
        @drop-service="handleServiceDrop"
      />

      <section v-if="filteredServices.length === 0" class="empty-state">
        <div class="empty-icon">
          <ServerOff v-if="activeServices.length === 0" :size="30" />
          <SearchX v-else :size="30" />
        </div>
        <h2>{{ activeServices.length === 0 ? 'Este workspace no tiene servicios' : 'No se encontraron servicios' }}</h2>
        <p v-if="activeServices.length === 0">
          Agrega tu primer servicio con un nombre y su comando de inicio.
        </p>
        <p v-else>
          Prueba con otro término de búsqueda o limpia los filtros activos.
        </p>
        <button v-if="activeServices.length === 0" class="btn btn-primary" type="button" @click="newService">
          <Plus :size="16" />
          Crear servicio
        </button>
        <button v-else class="btn btn-secondary" type="button" @click="clearFilters">
          <X :size="16" />
          Limpiar filtros
        </button>
      </section>
      </main>
    </div>

    <LogDrawer
      v-if="showLogDrawer"
      :service-id="selectedServiceId"
      :service-name="selectedServiceName"
      :status="selectedServiceStatus"
      :error="selectedServiceError"
      @close="closeLogDrawer"
    />

    <GitModal
      v-if="showGit"
      :services="activeServices"
      :initial-service="gitService"
      :service-status="gitService ? getStatus(gitService.id) : undefined"
      :workspace-name="activeWorkspaceName"
      @close="closeGit"
      @changed="refreshGitBranches"
    />

    <ServiceInfoModal
      v-if="infoService"
      :svc="infoService"
      :status="infoStatus"
      :status-class="getStatusClass(infoStatus)"
      :category-label="categoryLabel(infoService.category)"
      :workspace-name="infoWorkspaceName"
      :git-branch="getBranch(infoService.cwd)"
      :runtime="infoRuntime"
      :usage="getUsage(infoService.id)"
      @close="closeServiceInfo"
      @edit="editService"
      @move="openMoveService"
      @logs="openLogs"
      @git="openGit"
    />

    <ServiceWizard
      v-if="showServiceForm && !editingService"
      :workspaces="workspaces"
      :active-workspace-id="activeWorkspaceId"
      @save="handleSave"
      @cancel="closeServiceForm"
    />

    <ServiceForm
      v-else-if="showServiceForm"
      :service="editingService"
      :workspaces="workspaces"
      :active-workspace-id="activeWorkspaceId"
      @save="handleSave"
      @cancel="closeServiceForm"
    />

    <MoveServiceModal
      v-if="movingService"
      :service="movingService"
      :workspaces="workspaces"
      :stats="workspaceStats"
      @close="closeMoveService"
      @moved="handleServiceMoved"
    />

    <ServiceDiagnosticsModal
      v-if="diagnosticsService"
      :service="diagnosticsService"
      @close="closeDiagnostics"
      @logs="openLogsFromDiagnostics"
    />

    <ServiceContextMenu
      v-if="contextMenu && contextMenuService"
      :service="contextMenuService"
      :status="contextMenuStatus"
      :pid-alive="contextMenuRuntime?.pidAlive ?? false"
      :can-move="workspaces.length > 1"
      :selected-text="contextMenu.selectedText"
      :x="contextMenu.x"
      :y="contextMenu.y"
      @close="closeContextMenu"
      @start="start"
      @stop="stop"
      @restart="restart"
      @logs="openLogs"
      @info="openServiceInfo"
      @move="openMoveService"
      @edit="editService"
      @git="openGit"
      @diagnose="openDiagnostics"
      @delete="confirmDelete"
    />

    <GlobalContextMenu
      v-if="globalContextMenu"
      :x="globalContextMenu.x"
      :y="globalContextMenu.y"
      :target="globalContextMenu.target"
      @close="closeGlobalContextMenu"
    />

    <WorkspaceManager
      v-if="showWorkspaceManager"
      :stats="workspaceStats"
      @close="closeWorkspaceManager"
      @removed="handleWorkspaceRemoved"
    />

    <DashboardLogModal
      v-if="showDashboardLog"
      @close="closeDashboardLog"
    />

    <SettingsModal
      v-if="showSettings"
      @close="closeSettings"
      @saved="handleSettingsSaved"
    />

    <ErrorToast
      :items="notifications"
      @dismiss="dismissNotification"
      @open-log="openLogFromNotification"
    />
  </div>
</template>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
  background: var(--bg-gradient), var(--bg);
  background-attachment: fixed;
  color: var(--text);
}

.app-body {
  display: flex;
  flex: 1;
  min-height: 0;
}

.main-content {
  flex: 1;
  min-width: 0;
  width: 100%;
  max-width: 1440px;
  margin: 0 auto;
  padding: 24px;
  overflow: auto;
}

.toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
  margin-bottom: 14px;
  padding: 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  background: var(--surface-glass);
  box-shadow: var(--shadow-sm);
  backdrop-filter: blur(14px);
}

.search-box {
  position: relative;
  flex: 1;
  min-width: 220px;
}

.search-icon {
  position: absolute;
  top: 50%;
  left: 12px;
  color: var(--text-subtle);
  pointer-events: none;
  transform: translateY(-50%);
}

.search-input {
  width: 100%;
  min-height: 38px;
  padding: 9px 36px 9px 38px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface);
  color: var(--text);
  font-size: 13px;
  transition:
    border-color var(--transition),
    box-shadow var(--transition);
}

.search-input::placeholder {
  color: var(--text-subtle);
}

.search-input:hover {
  border-color: var(--border-strong);
}

.search-input:focus {
  outline: none;
  border-color: var(--primary);
  box-shadow: var(--focus-ring);
}

.search-clear {
  position: absolute;
  top: 50%;
  right: 8px;
  display: grid;
  place-items: center;
  width: 24px;
  height: 24px;
  padding: 0;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  transform: translateY(-50%);
}

.search-clear:hover {
  background: var(--surface-2);
  color: var(--text);
}

.category-chips {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.results-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  margin: 0 2px 14px;
  color: var(--text-muted);
  font-size: 12px;
}

.results-meta strong {
  color: var(--text);
}

.error-banner {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 14px;
  padding: 12px 14px;
  border: 1px solid var(--danger);
  border-radius: var(--radius-md);
  background: var(--danger-soft);
  color: var(--danger);
}

.error-banner-content {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-width: 0;
  gap: 2px;
}

.error-banner-content strong {
  font-size: 12px;
}

.error-banner-content span {
  overflow: hidden;
  color: var(--text-muted);
  font-family: var(--mono);
  font-size: 11px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  min-height: 320px;
  padding: 48px 24px;
  border: 1px dashed var(--border-strong);
  border-radius: var(--radius-xl);
  background: var(--surface);
  text-align: center;
  animation: fadeIn var(--transition);
}

.empty-icon {
  display: grid;
  place-items: center;
  width: 64px;
  height: 64px;
  margin-bottom: 4px;
  border-radius: 20px;
  background: var(--primary-soft);
  color: var(--primary);
}

.empty-state h2 {
  margin: 0;
  color: var(--text);
  font-size: 18px;
  letter-spacing: -0.01em;
}

.empty-state p {
  max-width: 420px;
  margin: 0;
  color: var(--text-muted);
  font-size: 13px;
}

.empty-state .btn {
  margin-top: 8px;
}

@media (max-width: 820px) {
  .main-content {
    padding: 16px;
  }

  .search-box {
    flex-basis: 100%;
  }
}
</style>
