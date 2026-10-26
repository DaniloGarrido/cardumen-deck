<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { confirm } from '@tauri-apps/plugin-dialog'
import {
  AlertTriangle,
  ArrowDown,
  ArrowRightLeft,
  ArrowUp,
  CheckCircle2,
  Download,
  FolderGit2,
  GitBranch,
  GitMerge,
  History,
  Loader2,
  Plus,
  RefreshCw,
  RotateCcw,
  Search,
  Trash2,
  Upload,
  X,
  XCircle,
} from '@lucide/vue'
import { useGit } from '../composables/useGit'
import type {
  GitBranchInfo,
  GitOperationResult,
  GitPullMode,
  Service,
} from '../types'

interface Props {
  services: Service[]
  initialService?: Service | null
  serviceStatus?: string
  workspaceName?: string
}

interface Emits {
  (e: 'close'): void
  (e: 'changed'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const {
  repositories,
  state,
  loading,
  busyAction,
  activity,
  error,
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
} = useGit()

const closeButton = ref<HTMLButtonElement | null>(null)
const selectedPath = ref('')
const branchQuery = ref('')
const branchKind = ref<'local' | 'remote'>('local')
const pullMode = ref<GitPullMode>('merge')
const mergeSource = ref('')
const mergeTarget = ref('')
const pushRemote = ref('')
const showNewBranch = ref(false)
const newBranchName = ref('')
const newBranchStart = ref('')

const servicePaths = computed(() => [
  ...new Set(
    [...props.services.map((service) => service.cwd), props.initialService?.cwd]
      .filter((path): path is string => Boolean(path)),
  ),
])

const repoPath = computed(() => state.value?.root ?? selectedPath.value)
const currentBranch = computed(() => state.value?.currentBranch ?? 'HEAD desacoplado')
const localBranches = computed(
  () => state.value?.branches.filter((branch) => branch.kind === 'local') ?? [],
)
const remoteBranches = computed(
  () => state.value?.branches.filter((branch) => branch.kind === 'remote') ?? [],
)
const visibleBranches = computed(() => {
  const query = branchQuery.value.trim().toLowerCase()
  const source = branchKind.value === 'local' ? localBranches.value : remoteBranches.value
  return query
    ? source.filter((branch) => branch.name.toLowerCase().includes(query))
    : source
})
const mergeSources = computed(() =>
  (state.value?.branches ?? []).filter((branch) => branch.name !== mergeTarget.value),
)
const mergeTargets = computed(() => localBranches.value)
const serviceRunning = computed(() =>
  ['running', 'external', 'starting'].includes(props.serviceStatus ?? ''),
)
const busy = computed(() => Boolean(busyAction.value))
const hasOperation = computed(() => Boolean(state.value?.operation))
const canCheckout = computed(
  () => Boolean(state.value) && !state.value?.dirty && !hasOperation.value && !busy.value,
)
const canPull = computed(
  () =>
    Boolean(state.value?.upstream)
    && !state.value?.dirty
    && !hasOperation.value
    && !busy.value,
)
const canMerge = computed(
  () =>
    Boolean(mergeSource.value)
    && Boolean(mergeTarget.value)
    && mergeSource.value !== mergeTarget.value
    && !state.value?.dirty
    && !hasOperation.value
    && !busy.value,
)
const canPush = computed(
  () => Boolean(state.value?.remotes.length) && !hasOperation.value && !busy.value,
)
const changedPreview = computed(() => state.value?.changedFiles.slice(0, 8) ?? [])
const conflictPreview = computed(() => state.value?.conflictFiles.slice(0, 8) ?? [])
const contextLabel = computed(
  () => props.initialService?.name || props.workspaceName || 'Workspace',
)
const repositoryName = computed(() => {
  const summary = repositories.value.find((repository) => repository.path === repoPath.value)
  if (summary) return summary.name
  const path = repoPath.value.replace(/[\\/]+$/, '')
  return path.split(/[\\/]/).pop() || 'Repositorio'
})
const showRepoName = computed(
  () => Boolean(repositoryName.value) && repositoryName.value !== contextLabel.value,
)
const operationLabel = computed(() => {
  const labels: Record<string, string> = {
    merge: 'merge',
    rebase: 'rebase',
    'cherry-pick': 'cherry-pick',
    revert: 'revert',
  }
  return labels[state.value?.operation ?? ''] ?? 'operación Git'
})
const pushLabel = computed(() => (state.value?.upstream ? 'Push' : 'Publicar rama'))
const branchKindLabel = computed(() =>
  branchKind.value === 'local' ? 'Locales' : 'Remotas',
)

function branchAheadBehind(branch: GitBranchInfo): string {
  const parts = []
  if (branch.ahead > 0) parts.push(`↑${branch.ahead}`)
  if (branch.behind > 0) parts.push(`↓${branch.behind}`)
  return parts.join(' ')
}

function syncSelections() {
  const current = state.value?.currentBranch ?? ''
  const local = localBranches.value
  const all = state.value?.branches ?? []

  if (!mergeTarget.value || !local.some((branch) => branch.name === mergeTarget.value)) {
    mergeTarget.value = current || local[0]?.name || ''
  }
  if (!mergeSource.value || !all.some((branch) => branch.name === mergeSource.value)) {
    mergeSource.value =
      all.find((branch) => branch.name !== mergeTarget.value)?.name ?? ''
  }
  if (!newBranchStart.value || !all.some((branch) => branch.name === newBranchStart.value)) {
    newBranchStart.value = current
  }

  const remotes = state.value?.remotes ?? []
  const upstreamRemote = state.value?.upstream?.split('/')[0]
  if (!pushRemote.value || !remotes.includes(pushRemote.value)) {
    pushRemote.value =
      (upstreamRemote && remotes.includes(upstreamRemote) ? upstreamRemote : undefined)
      ?? (remotes.includes('origin') ? 'origin' : remotes[0] ?? '')
  }
}

async function selectRepository(path: string) {
  if (!path || path === selectedPath.value) return
  selectedPath.value = path
  branchQuery.value = ''
  branchKind.value = 'local'
  await loadState(path)
}

async function initialize() {
  await loadRepositories(servicePaths.value)

  if (props.initialService?.cwd) {
    selectedPath.value = props.initialService.cwd
    const loaded = await loadState(props.initialService.cwd)
    if (loaded) {
      selectedPath.value = loaded.root
      return
    }
  }

  const fallback = repositories.value[0]?.path
  if (fallback) {
    selectedPath.value = fallback
    await loadState(fallback)
  }
}

async function refresh() {
  await loadRepositories(servicePaths.value)
  if (repoPath.value) await loadState(repoPath.value)
}

async function runAction(
  operation: () => Promise<GitOperationResult>,
): Promise<GitOperationResult | null> {
  try {
    const result = await operation()
    emit('changed')
    await loadRepositories(servicePaths.value)
    return result
  } catch {
    emit('changed')
    return null
  }
}

async function doFetch() {
  if (!repoPath.value) return
  await runAction(() => fetch(repoPath.value))
}

async function doPull() {
  if (!repoPath.value) return
  await runAction(() => pull(repoPath.value, pullMode.value))
}

async function doPush() {
  if (!repoPath.value) return
  await runAction(() => push(repoPath.value, pushRemote.value || undefined))
}

async function doCheckout(branch: string) {
  if (!repoPath.value) return
  await runAction(() => checkout(repoPath.value, branch))
}

async function doCreateBranch() {
  if (!repoPath.value || !newBranchName.value.trim()) return
  const result = await runAction(() =>
    createBranch(repoPath.value, newBranchName.value.trim(), newBranchStart.value || undefined),
  )
  if (result?.success) {
    showNewBranch.value = false
    newBranchName.value = ''
  }
}

async function doDeleteBranch(branch: string) {
  if (!repoPath.value) return
  const confirmed = await confirm(`¿Eliminar la rama local "${branch}"?`)
  if (!confirmed) return
  await runAction(() => deleteBranch(repoPath.value, branch))
}

async function doMerge() {
  if (!repoPath.value || !mergeSource.value || !mergeTarget.value) return

  if (mergeTarget.value !== state.value?.currentBranch) {
    const confirmed = await confirm(
      `Se cambiará a la rama "${mergeTarget.value}" y luego se traerán los cambios de "${mergeSource.value}". ¿Continuar?`,
    )
    if (!confirmed) return
  }

  await runAction(() => merge(repoPath.value, mergeSource.value, mergeTarget.value))
}

async function doAbort() {
  if (!repoPath.value) return
  const confirmed = await confirm(`¿Abortar la ${operationLabel.value} en curso?`)
  if (!confirmed) return
  await runAction(() => abortOperation(repoPath.value))
}

function formatCount(count: number, singular: string, plural: string): string {
  return `${count} ${count === 1 ? singular : plural}`
}

watch(state, syncSelections)

onMounted(async () => {
  closeButton.value?.focus()
  await initialize()
})
</script>

<template>
  <div class="git-overlay" @click="$emit('close')">
    <div
      class="git-panel"
      role="dialog"
      aria-modal="true"
      aria-labelledby="git-title"
      @click.stop
    >
      <header class="git-header">
        <div class="git-heading">
          <div class="git-icon"><GitBranch :size="20" /></div>
          <div class="git-title-block">
            <h2 id="git-title">Git</h2>
            <p>
              {{ contextLabel }}
              <span v-if="showRepoName">· {{ repositoryName }}</span>
            </p>
          </div>
        </div>

        <div class="git-head-actions">
          <button
            class="icon-btn"
            type="button"
            title="Actualizar estado"
            aria-label="Actualizar estado Git"
            :disabled="busy || loading"
            @click="refresh"
          >
            <RefreshCw :size="16" :class="{ 'is-spinning': loading }" />
          </button>
          <button
            ref="closeButton"
            class="close-btn"
            type="button"
            title="Cerrar"
            aria-label="Cerrar Git"
            @click="$emit('close')"
          >
            <X :size="17" />
          </button>
        </div>
      </header>

      <div class="git-body">
        <div v-if="repositories.length > 1" class="repo-picker" aria-label="Repositorios">
          <button
            v-for="repository in repositories"
            :key="repository.path"
            class="repo-chip"
            :class="{ 'is-active': repository.path === repoPath }"
            type="button"
            :title="repository.path"
            @click="selectRepository(repository.path)"
          >
            <FolderGit2 :size="14" />
            <span class="repo-chip-name">{{ repository.name }}</span>
            <span v-if="repository.branch" class="repo-chip-branch">{{ repository.branch }}</span>
            <span v-if="repository.dirty" class="repo-dirty-dot" title="Cambios locales"></span>
          </button>
        </div>

        <div v-if="serviceRunning" class="git-warning">
          <AlertTriangle :size="15" />
          <span>El servicio está en ejecución. Cambiar de rama o traer cambios puede afectar el proceso.</span>
        </div>

        <div v-if="error" class="git-error">
          <XCircle :size="15" />
          <span>{{ error }}</span>
        </div>

        <div v-if="loading && !state" class="git-loading">
          <Loader2 :size="20" class="is-spinning" />
          <span>Cargando repositorio...</span>
        </div>

        <div v-else-if="!state && repositories.length === 0" class="git-empty">
          <FolderGit2 :size="30" />
          <strong>No hay repositorios Git en este workspace</strong>
          <p>Agrega un servicio cuya carpeta pertenezca a un repositorio Git.</p>
        </div>

        <template v-else-if="state">
          <div class="repo-status">
            <div class="repo-status-main">
              <span class="current-branch" :title="state.root">
                <GitBranch :size="14" />
                {{ currentBranch }}
              </span>
              <span v-if="state.upstream" class="status-chip">
                <ArrowUp :size="11" />
                {{ state.upstream }}
              </span>
              <span v-if="state.ahead > 0" class="status-chip ahead" title="Commits por delante">
                <ArrowUp :size="11" /> {{ state.ahead }}
              </span>
              <span v-if="state.behind > 0" class="status-chip behind" title="Commits por detrás">
                <ArrowDown :size="11" /> {{ state.behind }}
              </span>
              <span v-if="state.detached" class="status-chip warning">HEAD desacoplado</span>
              <span v-if="state.dirty" class="status-chip warning">
                {{ formatCount(state.changedFiles.length, 'cambio local', 'cambios locales') }}
              </span>
              <span
                v-if="state.untrackedFiles.length"
                class="status-chip"
                :title="state.untrackedFiles.slice(0, 8).join(', ')"
              >
                {{ formatCount(state.untrackedFiles.length, 'sin seguimiento', 'sin seguimiento') }}
              </span>
              <span v-if="!state.dirty" class="status-chip clean">Sin cambios rastreados</span>
            </div>
            <span class="repo-path" :title="state.root">{{ state.root }}</span>
          </div>

          <div v-if="state.operation" class="git-conflict">
            <div class="conflict-copy">
              <AlertTriangle :size="16" />
              <div>
                <strong>Hay una {{ operationLabel }} en curso</strong>
                <p v-if="state.conflictFiles.length">
                  {{ formatCount(state.conflictFiles.length, 'conflicto', 'conflictos') }}:
                  {{ conflictPreview.join(', ') }}
                  <span v-if="state.conflictFiles.length > conflictPreview.length">…</span>
                </p>
                <p v-else>Termina o aborta la operación antes de continuar.</p>
              </div>
            </div>
            <button
              class="btn btn-secondary btn-sm"
              type="button"
              :disabled="busy"
              @click="doAbort"
            >
              <Loader2 v-if="busyAction === 'abort'" :size="14" class="is-spinning" />
              <RotateCcw v-else :size="14" />
              Abortar
            </button>
          </div>

          <div v-else-if="state.dirty" class="git-warning">
            <AlertTriangle :size="15" />
            <div>
              <span>Hay cambios locales. Checkout, pull y merge están bloqueados.</span>
              <span class="warning-files">{{ changedPreview.join(', ') }}<span v-if="state.changedFiles.length > changedPreview.length">…</span></span>
            </div>
          </div>

          <div class="git-grid">
            <section class="git-card branches-card">
              <div class="card-head">
                <div>
                  <h3><GitBranch :size="14" /> Ramas</h3>
                  <p>Cambia de rama o crea una nueva.</p>
                </div>
                <button
                  class="btn btn-secondary btn-sm"
                  type="button"
                  :disabled="busy || hasOperation"
                  @click="showNewBranch = !showNewBranch"
                >
                  <Plus :size="14" />
                  Nueva
                </button>
              </div>

              <div v-if="showNewBranch" class="new-branch">
                <input
                  v-model="newBranchName"
                  class="git-input"
                  type="text"
                  placeholder="nombre-de-rama"
                  aria-label="Nombre de la nueva rama"
                  @keydown.enter.prevent="doCreateBranch"
                />
                <select v-model="newBranchStart" class="git-select" aria-label="Punto de inicio">
                  <option value="">Desde la rama actual</option>
                  <option v-for="branch in state.branches" :key="branch.name" :value="branch.name">
                    {{ branch.name }}
                  </option>
                </select>
                <button
                  class="btn btn-primary btn-sm"
                  type="button"
                  :disabled="busy || !newBranchName.trim()"
                  @click="doCreateBranch"
                >
                  <Loader2 v-if="busyAction === 'create-branch'" :size="14" class="is-spinning" />
                  <Plus v-else :size="14" />
                  Crear
                </button>
              </div>

              <div class="branch-toolbar">
                <div class="branch-tabs" role="group" aria-label="Tipo de rama">
                  <button
                    class="branch-tab"
                    :class="{ 'is-active': branchKind === 'local' }"
                    type="button"
                    :aria-pressed="branchKind === 'local'"
                    @click="branchKind = 'local'"
                  >
                    Locales <span>{{ localBranches.length }}</span>
                  </button>
                  <button
                    class="branch-tab"
                    :class="{ 'is-active': branchKind === 'remote' }"
                    type="button"
                    :aria-pressed="branchKind === 'remote'"
                    @click="branchKind = 'remote'"
                  >
                    Remotas <span>{{ remoteBranches.length }}</span>
                  </button>
                </div>

                <label class="branch-search">
                  <Search :size="13" aria-hidden="true" />
                  <input
                    v-model="branchQuery"
                    type="text"
                    :placeholder="`Buscar ${branchKindLabel.toLowerCase()}...`"
                    aria-label="Buscar rama"
                  />
                </label>
              </div>

              <div class="branch-list">
                <div
                  v-for="branch in visibleBranches"
                  :key="`${branch.kind}-${branch.name}`"
                  class="branch-row"
                  :class="{ 'is-current': branch.current }"
                >
                  <div class="branch-main">
                    <GitBranch :size="13" class="branch-icon" />
                    <span class="branch-name" :title="branch.name">{{ branch.name }}</span>
                    <span v-if="branch.current" class="branch-current">Actual</span>
                    <span v-if="branch.upstream" class="branch-upstream">{{ branch.upstream }}</span>
                    <span v-if="branchAheadBehind(branch)" class="branch-track">
                      {{ branchAheadBehind(branch) }}
                    </span>
                  </div>
                  <div class="branch-actions">
                    <button
                      v-if="!branch.current"
                      class="branch-action"
                      type="button"
                      :disabled="!canCheckout"
                      :title="state.dirty ? 'Hay cambios locales' : `Cambiar a ${branch.name}`"
                      @click="doCheckout(branch.name)"
                    >
                      <Loader2 v-if="busyAction === 'checkout'" :size="13" class="is-spinning" />
                      <ArrowRightLeft v-else :size="13" />
                    </button>
                    <button
                      v-if="branch.kind === 'local' && !branch.current"
                      class="branch-action danger"
                      type="button"
                      :disabled="busy || hasOperation"
                      :title="`Eliminar ${branch.name}`"
                      @click="doDeleteBranch(branch.name)"
                    >
                      <Trash2 :size="13" />
                    </button>
                  </div>
                </div>
                <div v-if="visibleBranches.length === 0" class="branch-empty">
                  No hay ramas que coincidan.
                </div>
              </div>
            </section>

            <div class="git-side">
              <section class="git-card">
                <div class="card-head">
                  <div>
                    <h3><RefreshCw :size="14" /> Sincronizar</h3>
                    <p>Actualiza el repositorio y publica tu rama.</p>
                  </div>
                </div>

                <div class="sync-actions">
                  <button
                    class="btn btn-secondary btn-sm"
                    type="button"
                    :disabled="busy"
                    @click="doFetch"
                  >
                    <Loader2 v-if="busyAction === 'fetch'" :size="14" class="is-spinning" />
                    <Download v-else :size="14" />
                    Fetch
                  </button>
                  <button
                    class="btn btn-secondary btn-sm"
                    type="button"
                    :disabled="!canPush"
                    :title="state.remotes.length === 0 ? 'El repositorio no tiene remotos' : pushLabel"
                    @click="doPush"
                  >
                    <Loader2 v-if="busyAction === 'push'" :size="14" class="is-spinning" />
                    <Upload v-else :size="14" />
                    {{ pushLabel }}
                  </button>
                </div>

                <div class="pull-row">
                  <select v-model="pullMode" class="git-select" aria-label="Estrategia de pull">
                    <option value="merge">Pull con merge</option>
                    <option value="rebase">Pull con rebase</option>
                    <option value="ff-only">Solo fast-forward</option>
                  </select>
                  <button
                    class="btn btn-secondary btn-sm"
                    type="button"
                    :disabled="!canPull"
                    :title="!state.upstream ? 'La rama no tiene upstream' : state.dirty ? 'Hay cambios locales' : 'Pull'"
                    @click="doPull"
                  >
                    <Loader2 v-if="busyAction === 'pull'" :size="14" class="is-spinning" />
                    <Download v-else :size="14" />
                    Pull
                  </button>
                </div>

                <label v-if="state.remotes.length > 1" class="push-remote">
                  <span>Remoto para push</span>
                  <select v-model="pushRemote" class="git-select">
                    <option v-for="remote in state.remotes" :key="remote" :value="remote">
                      {{ remote }}
                    </option>
                  </select>
                </label>
              </section>

              <section class="git-card">
                <div class="card-head">
                  <div>
                    <h3><GitMerge :size="14" /> Traer cambios</h3>
                    <p>Integra una rama origen en una rama destino.</p>
                  </div>
                </div>

                <div class="merge-form">
                  <label class="merge-field">
                    <span>Origen</span>
                    <select v-model="mergeSource" class="git-select">
                      <option value="">Selecciona una rama</option>
                      <option v-for="branch in mergeSources" :key="branch.name" :value="branch.name">
                        {{ branch.name }}{{ branch.kind === 'remote' ? ' (remota)' : '' }}
                      </option>
                    </select>
                  </label>
                  <label class="merge-field">
                    <span>Destino</span>
                    <select v-model="mergeTarget" class="git-select">
                      <option v-for="branch in mergeTargets" :key="branch.name" :value="branch.name">
                        {{ branch.name }}
                      </option>
                    </select>
                  </label>
                </div>

                <p v-if="mergeTarget && mergeTarget !== state.currentBranch" class="merge-warning">
                  <AlertTriangle :size="13" />
                  Se cambiará a <strong>{{ mergeTarget }}</strong> antes del merge.
                </p>

                <button
                  class="btn btn-primary btn-sm merge-button"
                  type="button"
                  :disabled="!canMerge"
                  @click="doMerge"
                >
                  <Loader2 v-if="busyAction === 'merge'" :size="14" class="is-spinning" />
                  <GitMerge v-else :size="14" />
                  Traer cambios
                </button>
              </section>
            </div>
          </div>

          <section class="git-card activity-card">
            <div class="card-head">
              <div>
                <h3><History :size="14" /> Actividad</h3>
                <p>Resultado de las últimas operaciones.</p>
              </div>
              <button
                v-if="activity.length"
                class="text-btn"
                type="button"
                @click="clearActivity"
              >
                Limpiar
              </button>
            </div>

            <div v-if="activity.length === 0" class="activity-empty">
              Todavía no ejecutaste operaciones en este repositorio.
            </div>
            <div v-else class="activity-list">
              <div
                v-for="entry in activity"
                :key="entry.id"
                class="activity-row"
                :class="{ 'is-error': !entry.success }"
              >
                <CheckCircle2 v-if="entry.success" :size="14" class="activity-icon" />
                <XCircle v-else :size="14" class="activity-icon" />
                <div class="activity-main">
                  <div class="activity-head">
                    <strong>{{ entry.action }}</strong>
                    <span>{{ entry.timestamp }}</span>
                  </div>
                  <p>{{ entry.message }}</p>
                  <pre v-if="entry.detail">{{ entry.detail }}</pre>
                </div>
              </div>
            </div>
          </section>
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped>
.git-overlay {
  position: fixed;
  inset: 0;
  z-index: 1200;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  background: var(--overlay);
  backdrop-filter: blur(6px);
}

.git-panel {
  display: flex;
  flex-direction: column;
  width: 100%;
  max-width: 980px;
  max-height: 92vh;
  overflow: hidden;
  border: 1px solid var(--border);
  border-radius: var(--radius-xl);
  background: var(--surface);
  box-shadow: var(--shadow-lg);
  animation: gitIn 180ms ease-out;
}

.git-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  flex-shrink: 0;
  padding: 17px 20px;
  border-bottom: 1px solid var(--border);
}

.git-heading {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.git-icon {
  display: grid;
  place-items: center;
  width: 40px;
  height: 40px;
  flex-shrink: 0;
  border-radius: 12px;
  background: var(--info-soft);
  color: var(--info);
}

.git-title-block {
  min-width: 0;
}

.git-title-block h2 {
  margin: 0;
  color: var(--text);
  font-size: 17px;
  letter-spacing: -0.01em;
}

.git-title-block p {
  margin: 3px 0 0;
  overflow: hidden;
  color: var(--text-muted);
  font-size: 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.git-head-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.close-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  padding: 0;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface);
  color: var(--text-muted);
  cursor: pointer;
}

.close-btn:hover {
  border-color: var(--border-strong);
  background: var(--surface-2);
  color: var(--text);
}

.git-body {
  display: flex;
  flex-direction: column;
  gap: 14px;
  min-height: 0;
  padding: 16px 20px 20px;
  overflow: auto;
}

.git-body > * {
  flex-shrink: 0;
}

.repo-picker {
  display: flex;
  gap: 7px;
  overflow-x: auto;
  padding-bottom: 2px;
}

.repo-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  min-height: 30px;
  flex-shrink: 0;
  padding: 7px 9px;
  border: 1px solid var(--border);
  border-radius: 9px;
  background: var(--surface-2);
  color: var(--text-muted);
  font-size: 11px;
  cursor: pointer;
}

.repo-chip:hover {
  border-color: var(--border-strong);
  color: var(--text);
}

.repo-chip.is-active {
  border-color: var(--primary);
  background: var(--primary-soft);
  color: var(--primary);
}

.repo-chip-name {
  max-width: 160px;
  overflow: hidden;
  font-weight: 700;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.repo-chip-branch {
  color: var(--text-subtle);
  font-family: var(--mono);
  font-size: 10px;
}

.repo-dirty-dot {
  width: 6px;
  height: 6px;
  border-radius: 999px;
  background: var(--warning);
}

.git-warning,
.git-error,
.git-conflict {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 9px 11px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  font-size: 11px;
  line-height: 1.45;
}

.git-warning {
  border-color: var(--warning);
  background: var(--warning-soft);
  color: var(--warning);
}

.git-warning > svg,
.git-error > svg,
.git-conflict > svg {
  flex-shrink: 0;
  margin-top: 1px;
}

.warning-files {
  display: block;
  margin-top: 3px;
  color: var(--text-muted);
  font-family: var(--mono);
  font-size: 10px;
  overflow-wrap: anywhere;
}

.git-error {
  border-color: var(--danger);
  background: var(--danger-soft);
  color: var(--danger);
}

.git-conflict {
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  border-color: var(--warning);
  background: var(--warning-soft);
  color: var(--warning);
}

.conflict-copy {
  display: flex;
  align-items: flex-start;
  gap: 8px;
}

.conflict-copy p {
  margin: 2px 0 0;
  color: var(--text-muted);
  font-size: 10px;
  overflow-wrap: anywhere;
}

.git-loading,
.git-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  min-height: 220px;
  color: var(--text-muted);
  font-size: 12px;
  text-align: center;
}

.git-empty strong {
  color: var(--text);
  font-size: 14px;
}

.git-empty p {
  margin: 0;
  color: var(--text-subtle);
  font-size: 11px;
}

.repo-status {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  min-width: 0;
  padding: 9px 11px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
}

.repo-status-main {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
  min-width: 0;
}

.current-branch {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  color: var(--info);
  font-family: var(--mono);
  font-size: 12px;
  font-weight: 700;
}

.status-chip {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  padding: 2px 6px;
  border-radius: 999px;
  background: var(--surface);
  color: var(--text-muted);
  font-size: 9px;
  font-weight: 700;
  white-space: nowrap;
}

.status-chip.ahead {
  background: var(--success-soft);
  color: var(--success);
}

.status-chip.behind {
  background: var(--info-soft);
  color: var(--info);
}

.status-chip.warning {
  background: var(--warning-soft);
  color: var(--warning);
}

.status-chip.clean {
  background: var(--success-soft);
  color: var(--success);
}

.repo-path {
  overflow: hidden;
  color: var(--text-subtle);
  font-family: var(--mono);
  font-size: 9px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.git-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.15fr) minmax(0, 1fr);
  gap: 12px;
  align-items: start;
}

.git-side {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-width: 0;
}

.git-card {
  display: flex;
  flex-direction: column;
  gap: 11px;
  min-width: 0;
  padding: 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface-2);
}

.card-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
}

.card-head h3 {
  display: flex;
  align-items: center;
  gap: 6px;
  margin: 0;
  color: var(--text);
  font-size: 12px;
}

.card-head p {
  margin: 3px 0 0;
  color: var(--text-subtle);
  font-size: 10px;
}

.text-btn {
  padding: 0;
  border: 0;
  background: transparent;
  color: var(--primary);
  font-size: 10px;
  font-weight: 700;
  cursor: pointer;
}

.text-btn:hover {
  text-decoration: underline;
}

.new-branch {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) auto;
  gap: 6px;
  padding: 8px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface);
}

.git-input,
.git-select {
  min-width: 0;
  padding: 7px 8px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  outline: 0;
  background: var(--surface);
  color: var(--text);
  font-size: 11px;
}

.git-input:focus,
.git-select:focus {
  border-color: var(--primary);
  box-shadow: var(--focus-ring);
}

.branch-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.branch-tabs {
  display: inline-flex;
  padding: 2px;
  border: 1px solid var(--border);
  border-radius: 999px;
  background: var(--surface);
}

.branch-tab {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  border: 0;
  border-radius: 999px;
  background: transparent;
  color: var(--text-muted);
  font-size: 10px;
  font-weight: 600;
  cursor: pointer;
}

.branch-tab span {
  color: var(--text-subtle);
  font-family: var(--mono);
  font-size: 9px;
}

.branch-tab.is-active {
  background: var(--primary-soft);
  color: var(--primary);
}

.branch-tab.is-active span {
  color: var(--primary);
}

.branch-search {
  display: flex;
  align-items: center;
  gap: 5px;
  min-width: 140px;
  flex: 1;
  padding: 0 7px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface);
  color: var(--text-subtle);
}

.branch-search:focus-within {
  border-color: var(--primary);
  box-shadow: var(--focus-ring);
}

.branch-search input {
  min-width: 0;
  flex: 1;
  padding: 7px 0;
  border: 0;
  outline: 0;
  background: transparent;
  color: var(--text);
  font-size: 10px;
}

.branch-list {
  display: flex;
  flex-direction: column;
  max-height: 300px;
  overflow: auto;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface);
}

.branch-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  min-height: 34px;
  padding: 6px 8px;
  border-top: 1px solid var(--border);
}

.branch-row:first-child {
  border-top: 0;
}

.branch-row.is-current {
  background: var(--info-soft);
}

.branch-main {
  display: flex;
  align-items: center;
  gap: 5px;
  min-width: 0;
  flex-wrap: wrap;
}

.branch-icon {
  flex-shrink: 0;
  color: var(--text-subtle);
}

.branch-row.is-current .branch-icon {
  color: var(--info);
}

.branch-name {
  overflow: hidden;
  color: var(--text);
  font-family: var(--mono);
  font-size: 10px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.branch-current {
  padding: 1px 5px;
  border-radius: 999px;
  background: var(--info);
  color: var(--primary-contrast);
  font-size: 8px;
  font-weight: 800;
  text-transform: uppercase;
}

.branch-upstream,
.branch-track {
  color: var(--text-subtle);
  font-family: var(--mono);
  font-size: 9px;
}

.branch-track {
  color: var(--info);
}

.branch-actions {
  display: flex;
  align-items: center;
  gap: 3px;
  flex-shrink: 0;
}

.branch-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 25px;
  height: 25px;
  padding: 0;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  color: var(--text-subtle);
  cursor: pointer;
}

.branch-action:hover:not(:disabled) {
  border-color: var(--border);
  background: var(--surface-2);
  color: var(--text);
}

.branch-action.danger:hover:not(:disabled) {
  border-color: var(--danger);
  background: var(--danger-soft);
  color: var(--danger);
}

.branch-action:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

.branch-empty {
  padding: 16px;
  color: var(--text-subtle);
  font-size: 10px;
  text-align: center;
}

.sync-actions,
.pull-row {
  display: flex;
  gap: 6px;
}

.sync-actions > .btn,
.pull-row > .btn {
  flex: 1;
}

.pull-row {
  align-items: center;
}

.pull-row .git-select {
  flex: 1;
}

.push-remote,
.merge-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.push-remote > span,
.merge-field > span {
  color: var(--text-subtle);
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

.merge-form {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: 7px;
}

.merge-warning {
  display: flex;
  align-items: flex-start;
  gap: 5px;
  margin: 0;
  color: var(--warning);
  font-size: 10px;
  line-height: 1.4;
}

.merge-warning svg {
  flex-shrink: 0;
  margin-top: 1px;
}

.merge-button {
  width: 100%;
  justify-content: center;
}

.activity-card {
  gap: 8px;
}

.activity-empty {
  padding: 12px;
  border: 1px dashed var(--border);
  border-radius: var(--radius-sm);
  color: var(--text-subtle);
  font-size: 10px;
  text-align: center;
}

.activity-list {
  display: flex;
  flex-direction: column;
  max-height: 220px;
  overflow: auto;
}

.activity-row {
  display: flex;
  align-items: flex-start;
  gap: 7px;
  padding: 7px 2px;
  border-top: 1px solid var(--border);
}

.activity-row:first-child {
  border-top: 0;
}

.activity-icon {
  flex-shrink: 0;
  margin-top: 1px;
  color: var(--success);
}

.activity-row.is-error .activity-icon {
  color: var(--danger);
}

.activity-main {
  min-width: 0;
  flex: 1;
}

.activity-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.activity-head strong {
  color: var(--text);
  font-size: 10px;
  text-transform: capitalize;
}

.activity-head span {
  color: var(--text-subtle);
  font-family: var(--mono);
  font-size: 9px;
}

.activity-main p {
  margin: 2px 0 0;
  color: var(--text-muted);
  font-size: 10px;
  line-height: 1.4;
  overflow-wrap: anywhere;
}

.activity-main pre {
  max-height: 110px;
  margin: 5px 0 0;
  padding: 6px 7px;
  overflow: auto;
  border: 1px solid var(--border);
  border-radius: 5px;
  background: var(--surface);
  color: var(--text-subtle);
  font-family: var(--mono);
  font-size: 9px;
  line-height: 1.45;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.is-spinning {
  animation: gitSpin 0.9s linear infinite;
}

@keyframes gitSpin {
  to {
    transform: rotate(360deg);
  }
}

@keyframes gitIn {
  from {
    opacity: 0;
    transform: translateY(8px) scale(0.985);
  }
  to {
    opacity: 1;
    transform: none;
  }
}

@media (max-width: 760px) {
  .git-grid {
    grid-template-columns: 1fr;
  }

  .new-branch {
    grid-template-columns: 1fr;
  }

  .repo-status {
    align-items: flex-start;
    flex-direction: column;
  }

  .repo-path {
    max-width: 100%;
  }
}

@media (max-width: 520px) {
  .git-overlay {
    padding: 10px;
  }

  .git-panel {
    max-height: 96vh;
  }

  .git-header,
  .git-body {
    padding-inline: 14px;
  }

  .merge-form {
    grid-template-columns: 1fr;
  }

  .git-conflict {
    align-items: stretch;
    flex-direction: column;
  }
}

@media (prefers-reduced-motion: reduce) {
  .is-spinning {
    animation: none;
  }

  .git-panel {
    animation: none;
  }
}
</style>
