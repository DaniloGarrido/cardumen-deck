import { invoke } from '@tauri-apps/api/core'
import { ref } from 'vue'
import { logDashboardWarn } from './useDashboardLog'
import type { Service } from '../types'

const branches = ref<Record<string, string>>({})
let requestId = 0

export function useGitBranches() {
  async function load(services: Service[]) {
    const paths = [...new Set(services.map((service) => service.cwd).filter(Boolean))]
    const currentRequest = ++requestId

    if (paths.length === 0) {
      branches.value = {}
      return
    }

    try {
      const result = await invoke<Record<string, string>>('get_git_branches', { paths })
      if (currentRequest === requestId) {
        branches.value = result
      }
    } catch (error) {
      console.error('Error detectando ramas Git:', error)
      void logDashboardWarn('git.branches', error, { throttleMs: 30000 })
    }
  }

  function getBranch(cwd: string | undefined): string | undefined {
    return cwd ? branches.value[cwd] : undefined
  }

  return { branches, load, getBranch }
}
