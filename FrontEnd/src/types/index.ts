export interface Workspace {
  id: string
  name: string
  color?: string
  createdAt: string
  updatedAt: string
}

export interface WorkspaceState {
  workspaces: Workspace[]
  activeWorkspaceId: string
}

export interface WorkspaceStats {
  total: number
  running: number
  errors: number
}

export interface WorkspaceUsage {
  cpuPercent: number
  memoryBytes: number
  processCount: number
}

export type ServiceCategory = 'frontend' | 'backend' | 'custom'
export type ServicePreset =
  | 'vue'
  | 'spring-boot'
  | 'python'
  | 'dotnet'
  | 'rust'
  | 'go'
  | 'custom'

export interface Service {
  id: string
  workspaceId: string
  name: string
  category: ServiceCategory
  preset: ServicePreset
  cwd: string
  command: string
  args: string[]
  setupCommand?: string
  env: Record<string, string>
  url?: string
  port?: number
  color?: string
  createdAt: string
  updatedAt: string
}

export interface ServiceInput {
  workspaceId?: string
  name: string
  category: ServiceCategory
  preset: ServicePreset
  cwd: string
  command: string
  args: string[]
  setupCommand?: string
  env: Record<string, string>
  url?: string
  port?: number
  color?: string
}

export type ServiceStatus = 'stopped' | 'starting' | 'running' | 'stopping' | 'error' | 'external'
export type HealthState = 'unknown' | 'checking' | 'healthy' | 'degraded' | 'unhealthy' | 'stopped'

export interface ServiceHealth {
  state: HealthState
  kind: 'http' | 'tcp' | 'process'
  target?: string
  checkedAt?: number
  latencyMs?: number
  consecutiveFailures: number
  message?: string
}

export interface ServiceRuntime {
  status: ServiceStatus
  pid?: number
  pidAlive?: boolean
  startTime?: number
  runId?: string
  exitCode?: number
  error?: string
  health?: ServiceHealth
}

export interface ServiceUsage {
  pid: number
  cpuPercent: number
  memoryBytes: number
  processCount: number
}

export interface UsageHistory {
  pid?: number
  cpu: number[]
  memory: number[]
}

export type GitBranchKind = 'local' | 'remote'
export type GitOperationMode = 'merge' | 'rebase' | 'cherry-pick' | 'revert'
export type GitPullMode = 'merge' | 'rebase' | 'ff-only'

export interface GitBranchInfo {
  name: string
  kind: GitBranchKind
  current: boolean
  upstream?: string
  ahead: number
  behind: number
}

export interface GitRepositorySummary {
  path: string
  name: string
  branch?: string
  upstream?: string
  ahead: number
  behind: number
  dirty: boolean
}

export interface GitState {
  path: string
  root: string
  currentBranch?: string
  detached: boolean
  upstream?: string
  ahead: number
  behind: number
  dirty: boolean
  changedFiles: string[]
  untrackedFiles: string[]
  conflictFiles: string[]
  branches: GitBranchInfo[]
  remotes: string[]
  operation?: GitOperationMode
}

export interface GitOperationResult {
  success: boolean
  action: string
  message: string
  stdout: string
  stderr: string
  state: GitState
}

export type DashboardLogLevel = 'error' | 'warn' | 'info'

export interface DashboardLogEntry {
  id: string
  timestamp: string
  level: DashboardLogLevel
  source: string
  scope: string
  message: string
  details?: string
  serviceId?: string
  serviceName?: string
  context?: Record<string, unknown>
}

export interface DashboardLogInput {
  level: DashboardLogLevel
  source: string
  scope: string
  message: string
  details?: string
  serviceId?: string
  serviceName?: string
  context?: Record<string, unknown>
}

export interface DashboardLogPage {
  entries: DashboardLogEntry[]
  total: number
  path: string
  sizeBytes: number
  truncated: boolean
}

export interface AppNotification {
  id: string
  level: 'error' | 'warn'
  title: string
  message: string
  createdAt: number
  scope: string
}

export interface LogOutput {
  runId: string
  lines: string[]
  truncated: boolean
  totalLines?: number
  updatedAt?: number
}

export interface EnvironmentInfo {
  stack: string
  displayName: string
  setupCommand?: string
  runCommand?: string
  suggestedPreset?: string
  suggestedPort?: number
  venvPath?: string
  packageManager?: string
  detectedFiles: string[]
  warnings: string[]
}

export interface SetupStatus {
  taskId: string
  status: 'running' | 'success' | 'error' | 'cancelled'
  exitCode?: number
  logPath: string
  startedAt: number
  endedAt?: number
}

export interface Diagnostics {
  cwdExists: boolean
  commandExists: boolean
  portAvailable: boolean
  wrapperFound: boolean
  warnings: string[]
}

export interface ServiceDependency {
  name: string
  version?: string
  kind: string
  ecosystem: string
  manifest: string
  group?: string
}

export interface ServiceDependencyReport {
  dependencies: ServiceDependency[]
  manifests: string[]
  framework?: string
  total: number
  truncated: boolean
  warnings: string[]
}

export interface Settings {
  confirmOnExit: boolean
  logRetentionRuns: number
  logRetentionDays: number
  healthCheckEnabled: boolean
  healthCheckIntervalSecs: number
  healthCheckTimeoutSecs: number
  healthCheckFailures: number
  healthCheckStartupGraceSecs: number
}

export interface RunInfo {
  runId: string
  startTime: string
  endTime?: string
  status: ServiceStatus
  logPath: string
}