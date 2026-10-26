import { invoke } from '@tauri-apps/api/core'
import { computed, ref } from 'vue'
import { logDashboardWarn } from './useDashboardLog'
import type { Workspace, WorkspaceState } from '../types'

const workspaces = ref<Workspace[]>([])
const activeWorkspaceId = ref('')

function applyState(state: WorkspaceState) {
  workspaces.value = state.workspaces
  activeWorkspaceId.value = state.activeWorkspaceId
}

export function useWorkspaces() {
  async function load() {
    try {
      const state = await invoke<WorkspaceState>('get_workspace_state')
      applyState(state)
    } catch (error) {
      console.error('Error cargando workspaces:', error)
      void logDashboardWarn('workspace.load', error, { throttleMs: 10000 })
    }
  }

  async function create(name: string, color?: string): Promise<WorkspaceState> {
    const state = await invoke<WorkspaceState>('create_workspace', { name, color })
    applyState(state)
    return state
  }

  async function update(id: string, name: string, color?: string): Promise<WorkspaceState> {
    const state = await invoke<WorkspaceState>('update_workspace', { id, name, color })
    applyState(state)
    return state
  }

  async function remove(id: string): Promise<WorkspaceState> {
    const state = await invoke<WorkspaceState>('delete_workspace', { id })
    applyState(state)
    return state
  }

  async function setActive(id: string): Promise<WorkspaceState> {
    const state = await invoke<WorkspaceState>('set_active_workspace', { id })
    applyState(state)
    return state
  }

  const activeWorkspace = computed(
    () => workspaces.value.find((workspace) => workspace.id === activeWorkspaceId.value),
  )

  return {
    workspaces,
    activeWorkspaceId,
    activeWorkspace,
    load,
    create,
    update,
    remove,
    setActive,
  }
}
