import { invoke } from '@tauri-apps/api/core'
import { ref, computed } from 'vue'
import { logDashboardError, logDashboardWarn } from './useDashboardLog'
import type { Service, ServiceInput, ServiceStatus, ServiceRuntime, ServiceHealth, Diagnostics, Settings, RunInfo, ServiceCategory, ServicePreset } from '../types'

const services = ref<Service[]>([])
const runtimes = ref<Map<string, ServiceRuntime>>(new Map())

interface ServiceStatusInfo {
  status: ServiceStatus
  pidAlive: boolean
  exitCode?: number
  lastError?: string
  health?: ServiceHealth
}

interface RawService {
  id: string
  workspaceId?: string
  workspace_id?: string
  name: string
  category: ServiceCategory
  preset: ServicePreset
  cwd: string
  command: string
  args?: string[]
  setupCommand?: string
  setup_command?: string
  env?: Record<string, string>
  url?: string
  port?: number
  color?: string
  createdAt?: string
  created_at?: string
  updatedAt?: string
  updated_at?: string
}

function mapService(raw: RawService): Service {
  return {
    id: raw.id,
    workspaceId: raw.workspaceId ?? raw.workspace_id ?? 'default',
    name: raw.name,
    category: raw.category,
    preset: raw.preset,
    cwd: raw.cwd,
    command: raw.command,
    args: raw.args ?? [],
    setupCommand: raw.setupCommand ?? raw.setup_command,
    env: raw.env ?? {},
    url: raw.url,
    port: raw.port,
    color: raw.color,
    createdAt: raw.createdAt ?? raw.created_at ?? new Date().toISOString(),
    updatedAt: raw.updatedAt ?? raw.updated_at ?? new Date().toISOString(),
  }
}

export function useServices() {
  function errorText(error: unknown): string {
    return error instanceof Error ? error.message : String(error)
  }

  function serviceContext(id: string) {
    const service = services.value.find((item) => item.id === id)
    return {
      serviceId: id,
      serviceName: service?.name,
      context: service
        ? { command: service.command, args: service.args, cwd: service.cwd }
        : undefined,
    }
  }

  async function load() {
    try {
      const result = await invoke<RawService[]>('get_services')
      services.value = result.map(mapService)

      const validIds = new Set(services.value.map((service) => service.id))
      for (const id of runtimes.value.keys()) {
        if (!validIds.has(id)) runtimes.value.delete(id)
      }
    } catch (e) {
      console.error('Error cargando servicios:', e)
      void logDashboardWarn('service.load', e, { throttleMs: 10000 })
    }
  }

  async function add(input: ServiceInput) {
    const created = mapService(await invoke<RawService>('add_service', { service: input }))
    services.value.push(created)
    return created
  }

  async function update(id: string, input: ServiceInput) {
    const updated = mapService(await invoke<RawService>('update_service', { id, service: input }))
    const idx = services.value.findIndex(s => s.id === id)
    if (idx >= 0) services.value[idx] = updated
    return updated
  }

  async function remove(id: string) {
    await invoke('remove_service', { id })
    services.value = services.value.filter(s => s.id !== id)
    runtimes.value.delete(id)
  }

  async function move(id: string, workspaceId: string) {
    const updated = mapService(await invoke<RawService>('move_service', { id, workspaceId }))
    const idx = services.value.findIndex((service) => service.id === id)
    if (idx >= 0) {
      const next = [...services.value]
      next.splice(idx, 1)
      next.push(updated)
      services.value = next
    }
    return updated
  }

  function applyWorkspaceOrder(workspaceId: string, orderedIds: string[]): boolean {
    const workspaceServices = services.value.filter(
      (service) => service.workspaceId === workspaceId,
    )
    if (workspaceServices.length !== orderedIds.length) return false
    if (new Set(orderedIds).size !== orderedIds.length) return false

    const byId = new Map(workspaceServices.map((service) => [service.id, service]))
    if (orderedIds.some((id) => !byId.has(id))) return false

    const positions: number[] = []
    services.value.forEach((service, index) => {
      if (service.workspaceId === workspaceId) positions.push(index)
    })

    const next = [...services.value]
    positions.forEach((position, index) => {
      const service = byId.get(orderedIds[index])
      if (service) next[position] = service
    })
    services.value = next
    return true
  }

  async function reorder(workspaceId: string, orderedIds: string[]) {
    const previous = [...services.value]
    if (!applyWorkspaceOrder(workspaceId, orderedIds)) {
      throw new Error('Orden de servicios inválido')
    }

    try {
      await invoke('reorder_services', { workspaceId, orderedIds })
    } catch (error) {
      services.value = previous
      throw error
    }
  }

  async function start(id: string) {
    const rt = runtimes.value.get(id) ?? { status: 'stopped' as ServiceStatus }
    try {
      await invoke('start_service', { id })
      runtimes.value.set(id, { ...rt, status: 'starting', error: undefined, exitCode: undefined, health: undefined })
    } catch (error) {
      const current = runtimes.value.get(id) ?? rt
      runtimes.value.set(id, { ...current, status: 'error', error: errorText(error) })
      const service = serviceContext(id)
      void logDashboardError('service.start', error, {
        title: `No se pudo iniciar ${service.serviceName ?? 'el servicio'}`,
        notify: true,
        serviceId: service.serviceId,
        serviceName: service.serviceName,
        context: service.context,
      })
    }
  }

  async function stop(id: string) {
    const rt = runtimes.value.get(id) ?? { status: 'running' as ServiceStatus }
    try {
      await invoke('stop_service', { id })
      runtimes.value.set(id, { ...rt, status: 'stopping', health: undefined })
    } catch (error) {
      const current = runtimes.value.get(id) ?? rt
      runtimes.value.set(id, { ...current, status: 'error', error: errorText(error) })
      const service = serviceContext(id)
      void logDashboardError('service.stop', error, {
        title: `No se pudo detener ${service.serviceName ?? 'el servicio'}`,
        notify: true,
        serviceId: service.serviceId,
        serviceName: service.serviceName,
      })
    }
  }

  async function restart(id: string) {
    const rt = runtimes.value.get(id) ?? { status: 'running' as ServiceStatus }
    try {
      await invoke('restart_service', { id })
      runtimes.value.set(id, { ...rt, status: 'starting', error: undefined, exitCode: undefined, health: undefined })
    } catch (error) {
      const current = runtimes.value.get(id) ?? rt
      runtimes.value.set(id, { ...current, status: 'error', error: errorText(error) })
      const service = serviceContext(id)
      void logDashboardError('service.restart', error, {
        title: `No se pudo reiniciar ${service.serviceName ?? 'el servicio'}`,
        notify: true,
        serviceId: service.serviceId,
        serviceName: service.serviceName,
        context: service.context,
      })
    }
  }

  async function refreshStatuses() {
    try {
      const statuses = await invoke<Record<string, ServiceStatusInfo>>('get_service_statuses')
      const next = new Map(runtimes.value)
      let changed = false
      for (const [id, info] of Object.entries(statuses)) {
        const current = next.get(id) ?? { status: 'stopped' as ServiceStatus }
        const preserveLocalError = Boolean(
          current.status === 'error'
          && current.error
          && info.status === 'stopped'
          && !info.lastError,
        )
        const updated: ServiceRuntime = {
          ...current,
          status: preserveLocalError ? 'error' : info.status,
          pidAlive: info.pidAlive,
          exitCode: info.exitCode,
          error: preserveLocalError ? current.error : info.lastError,
          health: info.health,
        }
        if (
          current.status !== updated.status
          || current.pidAlive !== updated.pidAlive
          || current.exitCode !== updated.exitCode
          || current.error !== updated.error
          || JSON.stringify(current.health) !== JSON.stringify(updated.health)
        ) {
          next.set(id, updated)
          changed = true
        }
      }
      if (changed) runtimes.value = next
    } catch (e) {
      console.error('Error actualizando estados:', e)
      void logDashboardWarn('service.statuses', e, { throttleMs: 30000 })
    }
  }

  async function startAll(targets: Service[] = services.value) {
    for (const svc of targets) {
      const status = getStatus(svc.id)
      if (status === 'stopped') await start(svc.id)
      else if (status === 'error') await restart(svc.id)
    }
  }

  async function stopAll(targets: Service[] = services.value) {
    for (const svc of targets) {
      const status = getStatus(svc.id)
      if (status === 'running' || status === 'external' || status === 'error') await stop(svc.id)
    }
  }

  function getStatus(id: string): ServiceStatus {
    return runtimes.value.get(id)?.status ?? 'stopped'
  }

  function getRuntime(id: string): ServiceRuntime | undefined {
    return runtimes.value.get(id)
  }

  function setRuntime(id: string, rt: ServiceRuntime) {
    runtimes.value.set(id, rt)
  }

  function dismissError(id: string) {
    const current = runtimes.value.get(id)
    if (!current) return
    runtimes.value.set(id, {
      ...current,
      status: 'stopped',
      error: undefined,
      exitCode: undefined,
    })
  }

  const byCategory = computed(() => {
    const groups: Record<string, Service[]> = { frontend: [], backend: [], custom: [] }
    for (const svc of services.value) groups[svc.category].push(svc)
    return groups
  })

  return {
    services,
    runtimes,
    load,
    add,
    update,
    remove,
    move,
    reorder,
    start,
    stop,
    restart,
    refreshStatuses,
    startAll,
    stopAll,
    getStatus,
    getRuntime,
    setRuntime,
    dismissError,
    byCategory,
  }
}

export function useLogs() {
  async function readLogs(id: string, runId?: string, lines = 500): Promise<{ runId: string; lines: string[]; truncated: boolean }> {
    return await invoke('read_logs', { id, runId, lines })
  }

  async function listRuns(id: string): Promise<RunInfo[]> {
    return await invoke('list_log_runs', { id })
  }

  async function openLogFolder(id: string) {
    await invoke('open_log_folder', { id })
  }

  async function openServiceFolder(path: string) {
    await invoke('open_service_folder', { path })
  }

  async function openServiceUrl(url: string) {
    await invoke('open_service_url', { url })
  }

  async function startLogStream(id: string, runId?: string) {
    await invoke('start_log_stream', { id, runId })
  }

  async function stopLogStream(id: string) {
    await invoke('stop_log_stream', { id })
  }

  return {
    readLogs,
    listRuns,
    openLogFolder,
    openServiceFolder,
    openServiceUrl,
    startLogStream,
    stopLogStream,
  }
}

export function useDiagnostics() {
  async function diagnose(id: string): Promise<Diagnostics> {
    return await invoke('diagnose_service', { id })
  }
  return { diagnose }
}

export function useSettings() {
  async function load(): Promise<Settings> {
    return await invoke('get_settings')
  }

  async function save(settings: Settings): Promise<void> {
    await invoke('save_settings', { settings })
  }
  return { load, save }
}

export function useApp() {
  async function shutdown() {
    await invoke('shutdown_app')
  }
  return { shutdown }
}
