import { invoke } from '@tauri-apps/api/core'
import { ref } from 'vue'
import { logDashboardWarn } from './useDashboardLog'
import type { ServiceUsage, UsageHistory } from '../types'

export const USAGE_HISTORY_LIMIT = 60

const currentUsage = ref<Record<string, ServiceUsage>>({})
const histories = ref<Record<string, UsageHistory>>({})

function emptyHistory(): number[] {
  return Array<number>(USAGE_HISTORY_LIMIT).fill(0)
}

export function useServiceUsage() {
  let polling = false

  function getUsage(id: string): ServiceUsage | undefined {
    return currentUsage.value[id]
  }

  function getHistory(id: string): UsageHistory | undefined {
    return histories.value[id]
  }

  function reset() {
    currentUsage.value = {}
    histories.value = {}
  }

  async function refresh(ids: string[]) {
    if (polling) return

    if (ids.length === 0) {
      reset()
      return
    }

    polling = true
    try {
      const result = await invoke<Record<string, ServiceUsage>>('get_service_usage', { ids })
      const nextUsage: Record<string, ServiceUsage> = {}
      const nextHistories: Record<string, UsageHistory> = {}

      for (const id of ids) {
        const point = result[id]
        const previous = histories.value[id]
        const pidChanged = Boolean(
          point && previous?.pid !== undefined && previous.pid !== point.pid,
        )
        const previousCpu = pidChanged ? emptyHistory() : previous?.cpu ?? emptyHistory()
        const previousMemory = pidChanged ? emptyHistory() : previous?.memory ?? emptyHistory()

        nextHistories[id] = {
          pid: point?.pid ?? previous?.pid,
          cpu: [...previousCpu, point?.cpuPercent ?? 0].slice(-USAGE_HISTORY_LIMIT),
          memory: [...previousMemory, point?.memoryBytes ?? 0].slice(-USAGE_HISTORY_LIMIT),
        }

        if (point) nextUsage[id] = point
      }

      currentUsage.value = nextUsage
      histories.value = nextHistories
    } catch (error) {
      console.error('Error actualizando uso de CPU/RAM:', error)
      void logDashboardWarn('service.usage', error, { throttleMs: 30000 })
    } finally {
      polling = false
    }
  }

  return {
    currentUsage,
    histories,
    getUsage,
    getHistory,
    refresh,
    reset,
  }
}
