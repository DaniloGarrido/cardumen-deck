<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import {
  Activity,
  AlertTriangle,
  CheckCircle2,
  FolderOpen,
  RefreshCw,
  ScrollText,
  X,
  XCircle,
} from '@lucide/vue'
import { logDashboardError } from '../composables/useDashboardLog'
import { useDiagnostics, useLogs } from '../composables/useServices'
import type { Diagnostics, Service } from '../types'

interface Props {
  service: Service
}

interface Emits {
  (e: 'close'): void
  (e: 'logs', id: string): void
}

const props = defineProps<Props>()
defineEmits<Emits>()

const { diagnose } = useDiagnostics()
const { openServiceFolder } = useLogs()

const result = ref<Diagnostics | null>(null)
const loading = ref(true)
const error = ref('')

const checks = computed(() => {
  if (!result.value) return []
  return [
    {
      label: 'Carpeta de trabajo',
      ok: result.value.cwdExists,
      fail: 'La carpeta no existe o no es accesible.',
    },
    {
      label: 'Comando disponible',
      ok: result.value.commandExists,
      fail: 'El comando no se encontró en PATH.',
    },
    {
      label: 'Puerto disponible',
      ok: result.value.portAvailable,
      fail: 'El puerto configurado ya está en uso.',
    },
    {
      label: 'Wrapper del proyecto',
      ok: result.value.wrapperFound,
      fail: 'No se encontró el wrapper de Maven/Gradle.',
    },
  ]
})

async function load() {
  loading.value = true
  error.value = ''
  try {
    result.value = await diagnose(props.service.id)
  } catch (cause) {
    error.value = String(cause)
    void logDashboardError('service.diagnostics', cause, {
      title: `No se pudo diagnosticar ${props.service.name}`,
      notify: true,
      serviceId: props.service.id,
      serviceName: props.service.name,
    })
  } finally {
    loading.value = false
  }
}

async function openFolder() {
  try {
    await openServiceFolder(props.service.cwd)
  } catch (cause) {
    void logDashboardError('service.diagnostics.openFolder', cause, { notify: true })
  }
}

onMounted(load)
</script>

<template>
  <div class="diagnostics-overlay" @click="$emit('close')">
    <div
      class="diagnostics-panel"
      role="dialog"
      aria-modal="true"
      aria-labelledby="diagnostics-title"
      @click.stop
    >
      <header class="diagnostics-header">
        <div class="diagnostics-title">
          <div class="diagnostics-icon"><Activity :size="19" /></div>
          <div>
            <h3 id="diagnostics-title">Diagnóstico</h3>
            <p>{{ service.name }}</p>
          </div>
        </div>
        <button
          class="diagnostics-close"
          type="button"
          title="Cerrar"
          aria-label="Cerrar diagnóstico"
          @click="$emit('close')"
        >
          <X :size="17" />
        </button>
      </header>

      <div class="diagnostics-body">
        <div v-if="loading" class="diagnostics-state">
          <RefreshCw :size="18" class="spin" />
          Analizando servicio...
        </div>

        <div v-else-if="error" class="diagnostics-state is-error">
          <AlertTriangle :size="18" />
          <span>{{ error }}</span>
          <button class="btn btn-secondary btn-sm" type="button" @click="load">
            Reintentar
          </button>
        </div>

        <template v-else-if="result">
          <div class="checks">
            <div v-for="check in checks" :key="check.label" class="check-row">
              <CheckCircle2 v-if="check.ok" :size="17" class="check-ok" />
              <XCircle v-else :size="17" class="check-fail" />
              <div>
                <strong>{{ check.label }}</strong>
                <span>{{ check.ok ? 'Correcto' : check.fail }}</span>
              </div>
            </div>
          </div>

          <div v-if="result.warnings.length" class="warning-box">
            <div class="warning-title">
              <AlertTriangle :size="14" />
              Advertencias
            </div>
            <ul>
              <li v-for="warning in result.warnings" :key="warning">{{ warning }}</li>
            </ul>
          </div>

          <div v-else class="success-box">
            <CheckCircle2 :size="15" />
            No se detectaron problemas.
          </div>
        </template>
      </div>

      <footer class="diagnostics-footer">
        <button class="btn btn-ghost" type="button" @click="$emit('close')">Cerrar</button>
        <div class="diagnostics-actions">
          <button class="btn btn-secondary" type="button" @click="openFolder">
            <FolderOpen :size="15" />
            Abrir carpeta
          </button>
          <button class="btn btn-secondary" type="button" @click="$emit('logs', service.id)">
            <ScrollText :size="15" />
            Ver logs
          </button>
        </div>
      </footer>
    </div>
  </div>
</template>

<style scoped>
.diagnostics-overlay {
  position: fixed;
  inset: 0;
  z-index: 1150;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  background: var(--overlay);
  backdrop-filter: blur(6px);
}

.diagnostics-panel {
  display: flex;
  flex-direction: column;
  width: 100%;
  max-width: 500px;
  max-height: 90vh;
  overflow: hidden;
  border: 1px solid var(--border);
  border-radius: var(--radius-xl);
  background: var(--surface);
  box-shadow: var(--shadow-lg);
  animation: diagnosticsIn 180ms ease-out;
}

.diagnostics-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-shrink: 0;
  padding: 18px 20px;
  border-bottom: 1px solid var(--border);
}

.diagnostics-title {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.diagnostics-icon {
  display: grid;
  place-items: center;
  width: 40px;
  height: 40px;
  flex-shrink: 0;
  border-radius: 12px;
  background: var(--primary-soft);
  color: var(--primary);
}

.diagnostics-title h3 {
  margin: 0;
  color: var(--text);
  font-size: 16px;
}

.diagnostics-title p {
  margin: 3px 0 0;
  overflow: hidden;
  color: var(--text-muted);
  font-size: 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.diagnostics-close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  flex-shrink: 0;
  padding: 0;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface);
  color: var(--text-muted);
  cursor: pointer;
}

.diagnostics-close:hover {
  border-color: var(--border-strong);
  background: var(--surface-2);
  color: var(--text);
}

.diagnostics-body {
  display: flex;
  flex-direction: column;
  gap: 14px;
  min-height: 0;
  padding: 18px 20px;
  overflow: auto;
}

.diagnostics-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  min-height: 150px;
  flex-direction: column;
  color: var(--text-muted);
  font-size: 12px;
  text-align: center;
}

.diagnostics-state.is-error {
  color: var(--danger);
}

.spin {
  animation: spin 1s linear infinite;
}

.checks {
  display: flex;
  flex-direction: column;
  gap: 7px;
}

.check-row {
  display: flex;
  align-items: flex-start;
  gap: 9px;
  padding: 9px 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
}

.check-row > svg {
  flex-shrink: 0;
  margin-top: 1px;
}

.check-ok {
  color: var(--success);
}

.check-fail {
  color: var(--danger);
}

.check-row div {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.check-row strong {
  color: var(--text);
  font-size: 11px;
}

.check-row span {
  color: var(--text-muted);
  font-size: 10px;
}

.warning-box,
.success-box {
  padding: 10px;
  border: 1px solid var(--warning);
  border-radius: var(--radius-sm);
  background: var(--warning-soft);
  color: var(--warning);
  font-size: 11px;
}

.success-box {
  display: flex;
  align-items: center;
  gap: 6px;
  border-color: var(--success);
  background: var(--success-soft);
  color: var(--success);
}

.warning-title {
  display: flex;
  align-items: center;
  gap: 5px;
  font-weight: 700;
}

.warning-box ul {
  margin: 7px 0 0;
  padding-left: 18px;
  color: var(--text-muted);
  line-height: 1.5;
}

.diagnostics-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  flex-shrink: 0;
  padding: 14px 20px;
  border-top: 1px solid var(--border);
}

.diagnostics-actions {
  display: flex;
  gap: 7px;
  flex-wrap: wrap;
}

@media (max-width: 520px) {
  .diagnostics-overlay {
    padding: 12px;
  }

  .diagnostics-header,
  .diagnostics-body,
  .diagnostics-footer {
    padding-left: 16px;
    padding-right: 16px;
  }

  .diagnostics-footer {
    align-items: stretch;
    flex-direction: column;
  }

  .diagnostics-actions .btn {
    flex: 1;
    justify-content: center;
  }
}

@keyframes diagnosticsIn {
  from {
    opacity: 0;
    transform: translateY(8px) scale(0.985);
  }
  to {
    opacity: 1;
    transform: none;
  }
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
