import { invoke } from '@tauri-apps/api/core'
import { useNotifications } from './useNotifications'
import type { DashboardLogEntry, DashboardLogInput, DashboardLogLevel, DashboardLogPage } from '../types'

interface LogOptions {
  title?: string
  notify?: boolean
  details?: string
  serviceId?: string
  serviceName?: string
  context?: Record<string, unknown>
  throttleMs?: number
}

const throttleTimes = new Map<string, number>()
const { notify } = useNotifications()

function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message
  return String(error ?? 'Error desconocido')
}

function errorDetails(error: unknown, extra?: string): string | undefined {
  const parts: string[] = []
  if (error instanceof Error && error.stack) parts.push(error.stack)
  if (extra?.trim()) parts.push(extra.trim())
  return parts.length > 0 ? parts.join('\n\n') : undefined
}

async function write(
  level: DashboardLogLevel,
  scope: string,
  message: string,
  options: LogOptions = {},
) {
  if (options.throttleMs) {
    const key = `${scope}|${message}`
    const last = throttleTimes.get(key) ?? 0
    const now = Date.now()
    if (now - last < options.throttleMs) return
    throttleTimes.set(key, now)
  }

  const entry: DashboardLogInput = {
    level,
    source: 'frontend',
    scope,
    message,
    details: options.details,
    serviceId: options.serviceId,
    serviceName: options.serviceName,
    context: options.context,
  }

  try {
    await invoke('write_dashboard_log', { entry })
  } catch (error) {
    console.error('No se pudo escribir en el registro del dashboard:', error)
  }

  if (options.notify) {
    notify({
      level: level === 'warn' ? 'warn' : 'error',
      title: options.title ?? (level === 'warn' ? 'Advertencia' : 'Error'),
      message,
      scope,
    })
  }
}

export function logDashboardError(scope: string, error: unknown, options: LogOptions = {}) {
  const message = errorMessage(error)
  const title = options.title ?? message
  void write('error', scope, message, {
    ...options,
    title,
    details: errorDetails(error, options.details),
  })
}

export function logDashboardWarn(scope: string, error: unknown, options: LogOptions = {}) {
  const message = errorMessage(error)
  void write('warn', scope, message, {
    ...options,
    details: errorDetails(error, options.details),
  })
}

export function logDashboardInfo(scope: string, message: string, options: LogOptions = {}) {
  void write('info', scope, message, options)
}

export function useDashboardLog() {
  async function read(limit = 500, level?: string, query?: string): Promise<DashboardLogPage> {
    return invoke('read_dashboard_log', {
      limit,
      level: level && level !== 'all' ? level : null,
      query: query?.trim() ? query.trim() : null,
    })
  }

  async function clear(): Promise<void> {
    await invoke('clear_dashboard_log')
  }

  async function openFolder(): Promise<void> {
    await invoke('open_dashboard_log_folder')
  }

  async function listEntries(limit = 500): Promise<DashboardLogEntry[]> {
    const page = await read(limit)
    return page.entries
  }

  return {
    read,
    clear,
    openFolder,
    listEntries,
  }
}
