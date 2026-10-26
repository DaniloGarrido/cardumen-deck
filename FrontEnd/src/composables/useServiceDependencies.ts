import { invoke } from '@tauri-apps/api/core'
import { ref } from 'vue'
import { logDashboardWarn } from './useDashboardLog'
import type { ServiceDependencyReport } from '../types'

export function useServiceDependencies() {
  const report = ref<ServiceDependencyReport | null>(null)
  const loading = ref(false)
  const error = ref('')
  let requestId = 0

  async function load(cwd: string) {
    if (!cwd.trim()) {
      report.value = null
      error.value = ''
      return
    }

    const currentRequest = ++requestId
    loading.value = true
    error.value = ''

    try {
      const result = await invoke<ServiceDependencyReport>('get_service_dependencies', { cwd })
      if (currentRequest === requestId) {
        report.value = result
      }
    } catch (cause) {
      if (currentRequest === requestId) {
        report.value = null
        error.value = String(cause)
      }
      void logDashboardWarn('service.dependencies', cause, {
        throttleMs: 15000,
        context: { cwd },
      })
    } finally {
      if (currentRequest === requestId) {
        loading.value = false
      }
    }
  }

  return {
    report,
    loading,
    error,
    load,
  }
}
