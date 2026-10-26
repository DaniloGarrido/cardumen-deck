<script setup lang="ts">
import { ref } from 'vue'
import { confirm } from '@tauri-apps/plugin-dialog'
import { AlertTriangle, Check, Layers, Pencil, Plus, Trash2, X } from '@lucide/vue'
import { logDashboardError } from '../composables/useDashboardLog'
import { useWorkspaces } from '../composables/useWorkspaces'
import type { Workspace, WorkspaceStats } from '../types'

interface Props {
  stats: Record<string, WorkspaceStats>
}

interface Emits {
  (e: 'close'): void
  (e: 'removed', id: string): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const { workspaces, activeWorkspaceId, create, update, remove } = useWorkspaces()

const newName = ref('')
const newColor = ref('#4f46e5')
const editingId = ref('')
const editName = ref('')
const editColor = ref('#4f46e5')
const error = ref('')
const busy = ref(false)

async function createWorkspace() {
  const name = newName.value.trim()
  if (!name || busy.value) return

  busy.value = true
  error.value = ''
  try {
    await create(name, newColor.value)
    newName.value = ''
    newColor.value = '#4f46e5'
  } catch (cause) {
    error.value = String(cause)
    void logDashboardError('workspace.create', cause, {
      title: 'No se pudo crear el workspace',
      notify: true,
    })
  } finally {
    busy.value = false
  }
}

function startEdit(workspace: Workspace) {
  editingId.value = workspace.id
  editName.value = workspace.name
  editColor.value = workspace.color || '#4f46e5'
  error.value = ''
}

function cancelEdit() {
  editingId.value = ''
  error.value = ''
}

async function saveEdit() {
  const name = editName.value.trim()
  if (!editingId.value || !name || busy.value) return

  busy.value = true
  error.value = ''
  try {
    await update(editingId.value, name, editColor.value)
    editingId.value = ''
  } catch (cause) {
    error.value = String(cause)
    void logDashboardError('workspace.update', cause, {
      title: 'No se pudo actualizar el workspace',
      notify: true,
    })
  } finally {
    busy.value = false
  }
}

async function deleteWorkspace(workspace: Workspace) {
  if (workspaces.value.length <= 1 || busy.value) return

  const count = props.stats[workspace.id]?.total ?? 0
  const message = count > 0
    ? `¿Eliminar "${workspace.name}" y sus ${count} servicios? También se eliminarán sus procesos, runtime y logs.`
    : `¿Eliminar el workspace "${workspace.name}"?`

  if (!(await confirm(message))) return

  busy.value = true
  error.value = ''
  try {
    await remove(workspace.id)
    if (editingId.value === workspace.id) editingId.value = ''
    emit('removed', workspace.id)
  } catch (cause) {
    error.value = String(cause)
    void logDashboardError('workspace.delete', cause, {
      title: 'No se pudo eliminar el workspace',
      notify: true,
      context: { workspaceId: workspace.id, workspaceName: workspace.name },
    })
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <div class="workspace-overlay" @click="$emit('close')">
    <div class="workspace-panel" role="dialog" aria-modal="true" aria-labelledby="workspace-title" @click.stop>
      <div class="workspace-header">
        <div class="workspace-title">
          <div class="workspace-icon">
            <Layers :size="19" />
          </div>
          <div>
            <h3 id="workspace-title">Workspaces</h3>
            <p>Organiza tus servicios por proyecto o entorno</p>
          </div>
        </div>
        <button class="icon-btn" type="button" aria-label="Cerrar" @click="$emit('close')">
          <X :size="18" />
        </button>
      </div>

      <div class="workspace-body">
        <form class="create-workspace" @submit.prevent="createWorkspace">
          <div class="create-fields">
            <input v-model="newName" class="input" placeholder="Nombre del nuevo workspace" :disabled="busy" />
            <input v-model="newColor" class="color-input" type="color" :disabled="busy" />
          </div>
          <button class="btn btn-primary" type="submit" :disabled="busy || !newName.trim()">
            <Plus :size="15" />
            Crear workspace
          </button>
        </form>

        <p v-if="error" class="workspace-error" role="alert">
          <AlertTriangle :size="15" />
          {{ error }}
        </p>

        <div class="workspace-list">
          <div v-for="workspace in workspaces" :key="workspace.id" class="workspace-row">
            <template v-if="editingId === workspace.id">
              <input v-model="editName" class="input edit-name" :disabled="busy" />
              <input v-model="editColor" class="color-input" type="color" :disabled="busy" />
              <div class="row-actions">
                <button class="btn btn-primary btn-sm" type="button" :disabled="busy || !editName.trim()" @click="saveEdit">
                  <Check :size="14" />
                  Guardar
                </button>
                <button class="btn btn-ghost btn-sm" type="button" :disabled="busy" @click="cancelEdit">
                  <X :size="14" />
                  Cancelar
                </button>
              </div>
            </template>

            <template v-else>
              <span
                class="workspace-color"
                :style="{ background: workspace.color || 'var(--primary)' }"
              ></span>
              <div class="workspace-info">
                <div class="workspace-name-line">
                  <strong>{{ workspace.name }}</strong>
                  <span v-if="workspace.id === activeWorkspaceId" class="active-badge">Activo</span>
                </div>
                <span class="workspace-meta">
                  {{ stats[workspace.id]?.total ?? 0 }} servicios
                  · {{ stats[workspace.id]?.running ?? 0 }} activos
                  <template v-if="(stats[workspace.id]?.errors ?? 0) > 0">
                    · <span class="errors">{{ stats[workspace.id]?.errors }} con error</span>
                  </template>
                </span>
              </div>
              <div class="row-actions">
                <button class="btn btn-ghost btn-sm" type="button" :disabled="busy" @click="startEdit(workspace)">
                  <Pencil :size="14" />
                  Editar
                </button>
                <button
                  class="btn btn-ghost btn-sm delete-btn"
                  type="button"
                  :disabled="busy || workspaces.length <= 1"
                  :title="workspaces.length <= 1 ? 'No se puede eliminar el último workspace' : 'Eliminar workspace'"
                  @click="deleteWorkspace(workspace)"
                >
                  <Trash2 :size="14" />
                  Eliminar
                </button>
              </div>
            </template>
          </div>
        </div>

        <p class="workspace-warning">
          <AlertTriangle :size="14" />
          Eliminar un workspace también elimina sus servicios, procesos, runtime y logs.
        </p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.workspace-overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  background: var(--overlay);
  backdrop-filter: blur(6px);
}

.workspace-panel {
  width: 100%;
  max-width: 680px;
  max-height: 90vh;
  overflow: auto;
  border: 1px solid var(--border);
  border-radius: var(--radius-xl);
  background: var(--surface);
  box-shadow: var(--shadow-lg);
  animation: modalIn 0.18s ease-out;
}

.workspace-header {
  position: sticky;
  top: 0;
  z-index: 2;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 18px 20px;
  border-bottom: 1px solid var(--border);
  background: var(--surface);
}

.workspace-title {
  display: flex;
  align-items: center;
  gap: 12px;
}

.workspace-icon {
  display: grid;
  place-items: center;
  width: 40px;
  height: 40px;
  border-radius: 13px;
  background: var(--primary-soft);
  color: var(--primary);
}

.workspace-title h3 {
  margin: 0;
  color: var(--text);
  font-size: 16px;
}

.workspace-title p {
  margin: 3px 0 0;
  color: var(--text-muted);
  font-size: 12px;
}

.workspace-body {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 20px;
}

.create-workspace {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface-2);
}

.create-fields {
  display: flex;
  flex: 1;
  gap: 8px;
  min-width: 0;
}

.create-fields .input {
  flex: 1;
  min-width: 0;
}

.color-input {
  width: 40px;
  height: 38px;
  flex-shrink: 0;
  padding: 3px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface);
  cursor: pointer;
}

.workspace-error {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
  padding: 10px 12px;
  border: 1px solid var(--danger);
  border-radius: var(--radius-sm);
  background: var(--danger-soft);
  color: var(--danger);
  font-size: 12px;
  font-weight: 600;
}

.workspace-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.workspace-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 11px 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface);
}

.workspace-color {
  width: 10px;
  height: 10px;
  flex-shrink: 0;
  border-radius: 50%;
}

.workspace-info {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-width: 0;
  gap: 2px;
}

.workspace-name-line {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.workspace-name-line strong {
  overflow: hidden;
  color: var(--text);
  font-size: 13px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.active-badge {
  padding: 1px 6px;
  border-radius: 999px;
  background: var(--success-soft);
  color: var(--success);
  font-size: 9px;
  font-weight: 700;
  text-transform: uppercase;
}

.workspace-meta {
  color: var(--text-muted);
  font-size: 11px;
}

.errors {
  color: var(--danger);
  font-weight: 600;
}

.edit-name {
  flex: 1;
  min-width: 0;
}

.row-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.delete-btn {
  color: var(--danger);
}

.delete-btn:hover:not(:disabled) {
  background: var(--danger-soft);
  color: var(--danger);
}

.workspace-warning {
  display: flex;
  align-items: center;
  gap: 7px;
  margin: 0;
  color: var(--text-subtle);
  font-size: 11px;
}

@media (max-width: 640px) {
  .workspace-overlay {
    padding: 12px;
  }

  .create-workspace,
  .workspace-row {
    align-items: flex-start;
    flex-direction: column;
  }

  .create-fields {
    width: 100%;
  }

  .row-actions {
    width: 100%;
    justify-content: flex-end;
  }
}

@keyframes modalIn {
  from {
    opacity: 0;
    transform: translateY(8px) scale(0.985);
  }
  to {
    opacity: 1;
    transform: none;
  }
}
</style>
