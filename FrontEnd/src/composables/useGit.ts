import { invoke } from '@tauri-apps/api/core'
import { ref } from 'vue'
import { logDashboardWarn } from './useDashboardLog'
import type {
  GitOperationResult,
  GitPullMode,
  GitRepositorySummary,
  GitState,
} from '../types'

export interface GitActivityEntry {
  id: number
  action: string
  success: boolean
  message: string
  detail?: string
  timestamp: string
}

const ACTIVITY_LIMIT = 12

export function useGit() {
  const repositories = ref<GitRepositorySummary[]>([])
  const state = ref<GitState | null>(null)
  const loading = ref(false)
  const busyAction = ref<string | null>(null)
  const activity = ref<GitActivityEntry[]>([])
  const error = ref<string | null>(null)
  const lastResult = ref<GitOperationResult | null>(null)
  let activityId = 0

  function errorText(value: unknown): string {
    return value instanceof Error ? value.message : String(value)
  }

  function pushActivity(
    action: string,
    success: boolean,
    message: string,
    detail?: string,
  ) {
    activityId += 1
    activity.value = [
      {
        id: activityId,
        action,
        success,
        message,
        detail,
        timestamp: new Date().toLocaleTimeString('es-ES', {
          hour: '2-digit',
          minute: '2-digit',
          second: '2-digit',
        }),
      },
      ...activity.value,
    ].slice(0, ACTIVITY_LIMIT)
  }

  async function loadRepositories(paths: string[]) {
    error.value = null
    const uniquePaths = [...new Set(paths.filter(Boolean))]
    if (uniquePaths.length === 0) {
      repositories.value = []
      return
    }

    try {
      repositories.value = await invoke<GitRepositorySummary[]>('get_git_repositories', {
        paths: uniquePaths,
      })
    } catch (loadError) {
      console.error('Error cargando repositorios Git:', loadError)
      void logDashboardWarn('git.repositories', loadError, { throttleMs: 30000 })
      error.value = errorText(loadError)
    }
  }

  async function loadState(path: string) {
    loading.value = true
    error.value = null
    try {
      state.value = await invoke<GitState>('get_git_state', { path })
      return state.value
    } catch (loadError) {
      console.error('Error cargando estado Git:', loadError)
      void logDashboardWarn('git.state', loadError, { throttleMs: 30000 })
      error.value = errorText(loadError)
      state.value = null
      return null
    } finally {
      loading.value = false
    }
  }

  async function run(
    action: string,
    operation: () => Promise<GitOperationResult>,
  ): Promise<GitOperationResult> {
    busyAction.value = action
    error.value = null
    try {
      const result = await operation()
      lastResult.value = result
      state.value = result.state
      const detail = result.success ? result.stdout : result.stderr || result.stdout
      pushActivity(action, result.success, result.message, detail || undefined)
      return result
    } catch (operationError) {
      const message = errorText(operationError)
      error.value = message
      lastResult.value = null
      pushActivity(action, false, message)
      void logDashboardWarn(`git.${action}`, operationError, { throttleMs: 5000 })
      throw operationError
    } finally {
      busyAction.value = null
    }
  }

  function fetch(path: string) {
    return run('fetch', () => invoke<GitOperationResult>('git_fetch', { path }))
  }

  function pull(path: string, mode: GitPullMode) {
    return run('pull', () => invoke<GitOperationResult>('git_pull', { path, mode }))
  }

  function checkout(path: string, branch: string) {
    return run('checkout', () => invoke<GitOperationResult>('git_checkout', { path, branch }))
  }

  function createBranch(path: string, name: string, startPoint?: string) {
    return run('create-branch', () =>
      invoke<GitOperationResult>('git_create_branch', { path, name, startPoint }),
    )
  }

  function deleteBranch(path: string, name: string) {
    return run('delete-branch', () => invoke<GitOperationResult>('git_delete_branch', { path, name }))
  }

  function merge(path: string, source: string, target: string) {
    return run('merge', () => invoke<GitOperationResult>('git_merge', { path, source, target }))
  }

  function abortOperation(path: string) {
    return run('abort', () => invoke<GitOperationResult>('git_abort_operation', { path }))
  }

  function push(path: string, remote?: string) {
    return run('push', () => invoke<GitOperationResult>('git_push', { path, remote }))
  }

  function clearActivity() {
    activity.value = []
    lastResult.value = null
  }

  return {
    repositories,
    state,
    loading,
    busyAction,
    activity,
    error,
    lastResult,
    loadRepositories,
    loadState,
    fetch,
    pull,
    checkout,
    createBranch,
    deleteBranch,
    merge,
    abortOperation,
    push,
    clearActivity,
  }
}
