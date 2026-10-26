import { invoke } from '@tauri-apps/api/core'

export function useServiceTools() {
  async function openTerminal(cwd: string): Promise<void> {
    await invoke('open_service_terminal', { cwd })
  }

  async function openEditor(cwd: string): Promise<void> {
    await invoke('open_service_editor', { cwd })
  }

  return {
    openTerminal,
    openEditor,
  }
}
