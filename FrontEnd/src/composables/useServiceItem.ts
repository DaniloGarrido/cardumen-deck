import { computed, onBeforeUnmount, ref } from 'vue'
import { openUrl } from '@tauri-apps/plugin-opener'
import { writeToClipboard } from '../utils/clipboard'
import { resolveServiceUrl } from '../utils/serviceDefaults'
import type { Service } from '../types'

interface Options {
  service: () => Service
  status: () => string
  pidAlive: () => boolean
}

const statusLabels: Record<string, string> = {
  running: 'En ejecución',
  stopped: 'Detenido',
  starting: 'Iniciando',
  stopping: 'Deteniendo',
  error: 'Error',
  external: 'Externo',
}

export function useServiceItem(options: Options) {
  const service = computed(options.service)
  const status = computed(options.status)
  const pidAlive = computed(options.pidAlive)

  const canStart = computed(
    () => status.value === 'stopped' || (status.value === 'error' && !pidAlive.value),
  )
  const canStop = computed(
    () => pidAlive.value && ['running', 'external', 'error'].includes(status.value),
  )
  const canRestart = computed(
    () => ['running', 'external', 'error'].includes(status.value),
  )

  const statusText = computed(() => statusLabels[status.value] ?? status.value)
  const accentStyle = computed(() => ({
    '--service-accent': service.value.color || 'var(--primary)',
  }))
  const serviceUrl = computed(() => resolveServiceUrl(service.value))

  const copied = ref(false)
  let copyTimer: number | undefined

  async function copyUrl() {
    if (!serviceUrl.value) return
    try {
      await writeToClipboard(serviceUrl.value)
      copied.value = true
      if (copyTimer !== undefined) window.clearTimeout(copyTimer)
      copyTimer = window.setTimeout(() => {
        copied.value = false
      }, 1500)
    } catch (error) {
      console.error('No se pudo copiar la URL:', error)
    }
  }

  async function openInBrowser() {
    if (!serviceUrl.value) return
    try {
      await openUrl(serviceUrl.value)
    } catch (error) {
      console.error('No se pudo abrir la URL:', error)
    }
  }

  onBeforeUnmount(() => {
    if (copyTimer !== undefined) window.clearTimeout(copyTimer)
  })

  return {
    canStart,
    canStop,
    canRestart,
    statusText,
    accentStyle,
    serviceUrl,
    copied,
    copyUrl,
    openInBrowser,
  }
}
