<script setup lang="ts">
import { Activity, GitBranch, LayoutDashboard, Play, Plus, ScrollText, Settings, Square } from '@lucide/vue'
import ThemeToggle from './ThemeToggle.vue'
import WorkspaceUsageMeter from './WorkspaceUsageMeter.vue'

interface Props {
  servicesCount: number
  runningCount: number
  workspaceName?: string
  cpuPercent?: number
  memoryBytes?: number
  processCount?: number
}

interface Emits {
  (e: 'start-all'): void
  (e: 'stop-all'): void
  (e: 'new-service'): void
  (e: 'open-settings'): void
  (e: 'open-git'): void
  (e: 'open-dashboard-log'): void
}

const props = withDefaults(defineProps<Props>(), {
  workspaceName: 'Workspace',
  cpuPercent: 0,
  memoryBytes: 0,
  processCount: 0,
})
defineEmits<Emits>()
</script>

<template>
  <header class="header">
    <div class="header-left">
      <div class="brand">
        <div class="brand-icon">
          <LayoutDashboard :size="21" />
        </div>
        <div class="brand-text">
          <h1>Cardumen Deck</h1>
          <p>Control de servicios locales</p>
        </div>
      </div>

    </div>

    <div class="header-actions">
      <span class="badge badge-active" :title="`${runningCount} servicios activos`">
        <Activity :size="13" />
        {{ runningCount }} / {{ servicesCount }} activos
      </span>

      <WorkspaceUsageMeter
        class="header-usage"
        variant="header"
        :label="props.workspaceName"
        :cpu-percent="props.cpuPercent"
        :memory-bytes="props.memoryBytes"
        :process-count="props.processCount"
      />

      <ThemeToggle />

      <button
        class="icon-btn header-git"
        type="button"
        title="Operaciones Git del workspace"
        aria-label="Abrir operaciones Git"
        :disabled="servicesCount === 0"
        @click="$emit('open-git')"
      >
        <GitBranch :size="17" />
      </button>

      <button
        class="icon-btn header-log"
        type="button"
        title="Registro del dashboard"
        aria-label="Abrir registro del dashboard"
        @click="$emit('open-dashboard-log')"
      >
        <ScrollText :size="17" />
      </button>

      <button
        class="icon-btn header-settings"
        type="button"
        title="Configuración y apariencia"
        aria-label="Abrir configuración"
        @click="$emit('open-settings')"
      >
        <Settings :size="17" />
      </button>

      <span class="divider" aria-hidden="true"></span>

      <button class="btn btn-secondary btn-sm" type="button" @click="$emit('start-all')" :disabled="runningCount === servicesCount">
        <Play :size="14" class="action-icon success" />
        <span>Iniciar todo</span>
      </button>
      <button class="btn btn-secondary btn-sm" type="button" @click="$emit('stop-all')" :disabled="runningCount === 0">
        <Square :size="14" class="action-icon danger" />
        <span>Detener todo</span>
      </button>
      <button class="btn btn-primary btn-sm" type="button" @click="$emit('new-service')">
        <Plus :size="15" />
        <span>Nuevo servicio</span>
      </button>
    </div>
  </header>
</template>

<style scoped>
.header {
  position: sticky;
  top: 0;
  z-index: 100;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 14px 24px;
  border-bottom: 1px solid var(--border);
  background: var(--surface-glass);
  backdrop-filter: blur(18px);
  box-shadow: var(--shadow-sm);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 14px;
  min-width: 0;
  flex-wrap: wrap;
}

.brand {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.brand-icon {
  display: grid;
  place-items: center;
  width: 42px;
  height: 42px;
  border-radius: 13px;
  background: linear-gradient(135deg, var(--primary), var(--brand-accent));
  color: var(--primary-contrast);
  box-shadow: 0 12px 24px -16px var(--primary);
}

.brand-text h1 {
  margin: 0;
  color: var(--text);
  font-size: 18px;
  font-weight: 700;
  letter-spacing: -0.025em;
  line-height: 1.2;
}

.brand-text p {
  margin: 3px 0 0;
  color: var(--text-muted);
  font-size: 12px;
}

.header-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  flex-wrap: wrap;
}

.divider {
  width: 1px;
  height: 24px;
  margin: 0 4px;
  background: var(--border);
}

.header-usage {
  flex-shrink: 0;
}

.header-git:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.header-git:disabled:hover {
  border-color: var(--border);
  background: var(--surface);
  color: var(--text-muted);
}

.action-icon.success {
  color: var(--success);
}

.action-icon.danger {
  color: var(--danger);
}

@media (max-width: 980px) {
  .header {
    align-items: flex-start;
    flex-direction: column;
    padding: 14px 18px;
  }

  .header-actions {
    justify-content: flex-start;
    width: 100%;
  }

  .divider {
    display: none;
  }
}

@media (max-width: 560px) {
  .brand-text p {
    display: none;
  }

  .header-actions .btn span {
    display: none;
  }
}
</style>
