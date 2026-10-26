import { invoke } from '@tauri-apps/api/core'
import type { EnvironmentInfo, LogOutput, SetupStatus } from '../types'

export function useEnvironment() {
  async function detect(cwd: string): Promise<EnvironmentInfo> {
    return await invoke('detect_environment', { cwd })
  }

  async function startSetup(cwd: string, command: string): Promise<SetupStatus> {
    return await invoke('start_setup', { cwd, command })
  }

  async function getSetupStatus(taskId: string): Promise<SetupStatus> {
    return await invoke('get_setup_status', { taskId })
  }

  async function readSetupLog(taskId: string, lines = 500): Promise<LogOutput> {
    return await invoke('read_setup_log', { taskId, lines })
  }

  async function cancelSetup(taskId: string): Promise<void> {
    await invoke('cancel_setup', { taskId })
  }

  async function startSetupLogStream(taskId: string): Promise<void> {
    await invoke('start_setup_log_stream', { taskId })
  }

  async function stopSetupLogStream(taskId: string): Promise<void> {
    await invoke('stop_setup_log_stream', { taskId })
  }

  return {
    detect,
    startSetup,
    getSetupStatus,
    readSetupLog,
    cancelSetup,
    startSetupLogStream,
    stopSetupLogStream,
  }
}
