<script setup lang="ts">
import { computed, nextTick, ref, watch, type Component } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import {
  AlertTriangle,
  ArrowLeft,
  ArrowRight,
  Check,
  CheckCircle2,
  ClipboardCheck,
  FolderOpen,
  Loader2,
  RefreshCw,
  ScanSearch,
  Settings2,
  Sparkles,
  Wrench,
  X,
} from '@lucide/vue'
import { logDashboardWarn } from '../composables/useDashboardLog'
import { useEnvironment } from '../composables/useEnvironment'
import {
  PROJECT_TYPES,
  projectTypeForPreset,
  projectTypeForStack,
  type ProjectTypeOption,
} from '../utils/projectTypes'
import { defaultPortForPreset, normalizeUrl, portFromCommand, portFromUrl } from '../utils/serviceDefaults'
import EnvironmentSetupPanel from './EnvironmentSetupPanel.vue'
import type {
  EnvironmentInfo,
  ServiceCategory,
  ServiceInput,
  ServicePreset,
  Workspace,
} from '../types'

interface Props {
  workspaces: Workspace[]
  activeWorkspaceId: string
}

interface Emits {
  (e: 'save', input: ServiceInput): void
  (e: 'cancel'): void
}

type WizardStepId = 'folder' | 'type' | 'config' | 'environment' | 'review'

interface WizardStep {
  id: WizardStepId
  title: string
  description: string
  icon: Component
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()
const { detect } = useEnvironment()

const steps: WizardStep[] = [
  { id: 'folder', title: 'Carpeta', description: 'Selecciona el proyecto', icon: FolderOpen },
  { id: 'type', title: 'Tipo', description: 'Detectado o manual', icon: ScanSearch },
  { id: 'config', title: 'Configuración', description: 'Comando y red', icon: Settings2 },
  { id: 'environment', title: 'Entorno', description: 'Dependencias opcionales', icon: Wrench },
  { id: 'review', title: 'Revisión', description: 'Confirma y crea', icon: ClipboardCheck },
]

const categoryLabels: Record<ServiceCategory, string> = {
  frontend: 'Frontend',
  backend: 'Backend',
  custom: 'Personalizado',
}

const form = ref({
  workspaceId: props.activeWorkspaceId,
  name: '',
  category: 'custom' as ServiceCategory,
  preset: 'custom' as ServicePreset,
  cwd: '',
  command: '',
  args: '',
  setupCommand: '',
  env: '{}',
  url: 'http://localhost:3000',
  port: '3000',
  color: '#64748b',
})

const currentStep = ref<WizardStepId>('folder')
const detection = ref<EnvironmentInfo | null>(null)
const detecting = ref(false)
const detectionError = ref('')
const selectedTypeId = ref<ServicePreset | null>(null)
const manualMode = ref(false)
const stepError = ref('')
const contentRef = ref<HTMLElement | null>(null)
let detectToken = 0

const stepIndex = computed(() => steps.findIndex((step) => step.id === currentStep.value))
const currentStepData = computed(() => steps[stepIndex.value] ?? steps[0])
const isFirstStep = computed(() => stepIndex.value === 0)
const isReviewStep = computed(() => currentStep.value === 'review')
const selectedType = computed(() => projectTypeForPreset(selectedTypeId.value))
const detectedType = computed(() =>
  detection.value
    ? projectTypeForStack(detection.value.stack, detection.value.suggestedPreset)
    : undefined,
)
const detectedAutomatically = computed(
  () => Boolean(detection.value && detection.value.stack !== 'custom' && detection.value.runCommand),
)
const summaryCommand = computed(() =>
  [form.value.command, form.value.args].filter(Boolean).join(' '),
)
const envCount = computed(() => {
  try {
    const parsed = JSON.parse(form.value.env || '{}')
    return parsed && typeof parsed === 'object' ? Object.keys(parsed).length : 0
  } catch {
    return 0
  }
})
const canContinue = computed(() => {
  if (currentStep.value === 'folder') return Boolean(form.value.cwd.trim()) && !detecting.value
  if (currentStep.value === 'type') return Boolean(selectedTypeId.value)
  return true
})

watch(
  form,
  () => {
    if (stepError.value) stepError.value = ''
  },
  { deep: true },
)

function folderName(path: string): string {
  const trimmed = path.replace(/[\\/]+$/, '')
  return trimmed.split(/[\\/]/).pop() || 'Mi servicio'
}

function resetForFolder(path: string) {
  const port = defaultPortForPreset('custom')
  form.value.cwd = path
  form.value.name = folderName(path)
  form.value.category = 'custom'
  form.value.preset = 'custom'
  form.value.command = ''
  form.value.args = ''
  form.value.setupCommand = ''
  form.value.env = '{}'
  form.value.port = String(port)
  form.value.url = `http://localhost:${port}`
  form.value.color = '#64748b'
  detection.value = null
  detectionError.value = ''
  selectedTypeId.value = null
  manualMode.value = false
  stepError.value = ''
}

function applyEnvironment(info: EnvironmentInfo) {
  const type = projectTypeForStack(info.stack, info.suggestedPreset)

  if (type) {
    selectedTypeId.value = type.id
    manualMode.value = false
    form.value.category = type.category
    form.value.preset = type.id
    form.value.color = type.color
    form.value.command = info.runCommand ?? type.command
    form.value.setupCommand = info.setupCommand ?? type.setupCommand ?? ''
  } else {
    selectedTypeId.value = null
    manualMode.value = true
    form.value.category = 'custom'
    form.value.preset = 'custom'
    form.value.color = '#64748b'
    form.value.command = info.runCommand ?? ''
    form.value.setupCommand = info.setupCommand ?? ''
  }

  form.value.args = ''
  const port = info.suggestedPort ?? type?.port ?? defaultPortForPreset(type?.id ?? 'custom')
  form.value.port = String(port)
  form.value.url = `http://localhost:${port}`
}

async function detectProject(path: string) {
  const token = ++detectToken
  detecting.value = true
  detectionError.value = ''

  try {
    const info = await detect(path)
    if (token !== detectToken) return
    detection.value = info
    applyEnvironment(info)
  } catch (error) {
    if (token !== detectToken) return
    detection.value = null
    selectedTypeId.value = null
    manualMode.value = true
    detectionError.value = error instanceof Error ? error.message : String(error)
    void logDashboardWarn('serviceWizard.detect', error, {
      throttleMs: 10000,
      context: { cwd: path },
    })
  } finally {
    if (token === detectToken) detecting.value = false
  }
}

async function pickFolder() {
  try {
    const path = await open({
      title: 'Seleccionar carpeta del proyecto',
      directory: true,
      multiple: false,
    })
    if (typeof path !== 'string') return
    resetForFolder(path)
    await detectProject(path)
  } catch (error) {
    stepError.value = 'No se pudo abrir el selector de carpetas'
    void logDashboardWarn('serviceWizard.pickFolder', error, { throttleMs: 5000 })
  }
}

async function retryDetection() {
  if (!form.value.cwd.trim()) return
  await detectProject(form.value.cwd)
}

function applyProjectType(type: ProjectTypeOption) {
  selectedTypeId.value = type.id
  manualMode.value = true
  form.value.category = type.category
  form.value.preset = type.id
  form.value.command = type.command
  form.value.args = ''
  form.value.setupCommand = type.setupCommand ?? ''
  form.value.port = String(type.port)
  form.value.url = `http://localhost:${type.port}`
  form.value.color = type.color
  stepError.value = ''
}

function useDetectedType() {
  if (!detection.value) return
  applyEnvironment(detection.value)
  stepError.value = ''
}

function validateEnv(): boolean {
  try {
    JSON.parse(form.value.env || '{}')
    return true
  } catch {
    stepError.value = 'Variables de entorno: JSON inválido'
    return false
  }
}

function validateConfig(): boolean {
  stepError.value = ''

  if (!form.value.cwd.trim()) {
    stepError.value = 'Selecciona una carpeta de proyecto'
    return false
  }
  if (!form.value.name.trim()) {
    stepError.value = 'El nombre es obligatorio'
    return false
  }
  if (!form.value.command.trim()) {
    stepError.value = 'El comando es obligatorio'
    return false
  }
  if (!validateEnv()) return false

  const portInput = form.value.port ? Number(form.value.port) : undefined
  if (portInput !== undefined && (!Number.isInteger(portInput) || portInput < 1 || portInput > 65535)) {
    stepError.value = 'El puerto debe estar entre 1 y 65535'
    return false
  }

  return true
}

async function focusStep() {
  await nextTick()
  window.setTimeout(() => {
    const element = contentRef.value?.querySelector<HTMLElement>(
      'input:not([type="hidden"]), select, textarea, button',
    )
    element?.focus()
  }, 160)
}

function goToStep(id: WizardStepId) {
  const index = steps.findIndex((step) => step.id === id)
  if (index < 0 || index >= stepIndex.value) return
  currentStep.value = id
  stepError.value = ''
  void focusStep()
}

function nextStep() {
  stepError.value = ''

  if (currentStep.value === 'folder') {
    if (!form.value.cwd.trim()) {
      stepError.value = 'Selecciona una carpeta de proyecto'
      return
    }
    if (detecting.value) return
    currentStep.value = 'type'
  } else if (currentStep.value === 'type') {
    if (!selectedTypeId.value) {
      stepError.value = 'Selecciona un tipo de proyecto'
      return
    }
    currentStep.value = 'config'
  } else if (currentStep.value === 'config') {
    if (!validateConfig()) return
    currentStep.value = 'environment'
  } else if (currentStep.value === 'environment') {
    currentStep.value = 'review'
  } else {
    submit()
    return
  }

  void focusStep()
}

function previousStep() {
  if (isFirstStep.value) return
  currentStep.value = steps[stepIndex.value - 1].id
  stepError.value = ''
  void focusStep()
}

function submit() {
  if (!validateConfig()) {
    currentStep.value = 'config'
    return
  }

  const command = form.value.command.trim()
  const args = form.value.args.trim() ? form.value.args.trim().split(/\s+/) : []
  const portInput = form.value.port ? Number(form.value.port) : undefined
  const port =
    portInput ??
    portFromUrl(form.value.url) ??
    portFromCommand(command, args) ??
    defaultPortForPreset(form.value.preset)
  const url = normalizeUrl(form.value.url) || `http://localhost:${port}`

  emit('save', {
    workspaceId: form.value.workspaceId || props.activeWorkspaceId,
    name: form.value.name.trim(),
    category: form.value.category,
    preset: form.value.preset,
    cwd: form.value.cwd.trim(),
    command,
    args,
    setupCommand: form.value.setupCommand.trim() || undefined,
    env: JSON.parse(form.value.env || '{}'),
    url,
    port,
    color: form.value.color.trim() || undefined,
  })
}
</script>

<template>
  <div class="wizard-overlay" @click="emit('cancel')">
    <div
      class="wizard-panel"
      role="dialog"
      aria-modal="true"
      aria-labelledby="service-wizard-title"
      @click.stop
    >
      <header class="wizard-header">
        <div class="wizard-heading">
          <div class="wizard-icon"><Sparkles :size="19" /></div>
          <div>
            <h2 id="service-wizard-title">Nuevo servicio</h2>
            <p>
              Paso {{ stepIndex + 1 }} de {{ steps.length }} ·
              {{ currentStepData.description }}
            </p>
          </div>
        </div>
        <button class="icon-btn" type="button" aria-label="Cerrar" @click="emit('cancel')">
          <X :size="18" />
        </button>
      </header>

      <div class="wizard-body">
        <nav class="wizard-steps" aria-label="Pasos para crear el servicio">
          <button
            v-for="(step, index) in steps"
            :key="step.id"
            class="wizard-step"
            :class="{ active: index === stepIndex, completed: index < stepIndex }"
            type="button"
            :disabled="index >= stepIndex"
            :aria-current="index === stepIndex ? 'step' : undefined"
            @click="goToStep(step.id)"
          >
            <span class="wizard-step-icon">
              <Check v-if="index < stepIndex" :size="14" />
              <component :is="step.icon" v-else :size="14" />
            </span>
            <span class="wizard-step-copy">
              <strong>{{ step.title }}</strong>
              <small>{{ step.description }}</small>
            </span>
          </button>
        </nav>

        <div ref="contentRef" class="wizard-content">
          <Transition name="wizard-fade" mode="out-in">
            <section :key="currentStep" class="wizard-step-content">
              <template v-if="currentStep === 'folder'">
                <div class="step-head">
                  <span class="step-kicker">Paso 1</span>
                  <h3>¿Dónde está tu proyecto?</h3>
                  <p>Selecciona la carpeta raíz. Analizaremos el proyecto automáticamente.</p>
                </div>

                <div class="folder-picker" :class="{ 'has-folder': form.cwd }">
                  <div class="folder-icon"><FolderOpen :size="28" /></div>
                  <div class="folder-copy">
                    <strong>{{ form.cwd ? folderName(form.cwd) : 'Selecciona una carpeta' }}</strong>
                    <span class="mono-path">
                      {{ form.cwd || 'Ejemplo: C:\\Proyectos\\mi-app' }}
                    </span>
                  </div>
                  <button class="btn btn-secondary btn-sm" type="button" @click="pickFolder">
                    {{ form.cwd ? 'Cambiar' : 'Elegir carpeta' }}
                  </button>
                </div>

                <div v-if="detecting" class="detect-state">
                  <Loader2 :size="16" class="spinning" />
                  Analizando proyecto...
                </div>

                <div v-else-if="detection && detectedAutomatically" class="detection-summary">
                  <div class="detection-title">
                    <CheckCircle2 :size="15" />
                    {{ detection.displayName }} detectado
                  </div>
                  <div class="detection-chips">
                    <span v-if="detection.packageManager">{{ detection.packageManager }}</span>
                    <span v-for="file in detection.detectedFiles" :key="file">{{ file }}</span>
                  </div>
                  <p v-if="detection.runCommand" class="hint">
                    Comando sugerido: <code>{{ detection.runCommand }}</code>
                  </p>
                  <div v-for="warning in detection.warnings" :key="warning" class="detect-warning">
                    <AlertTriangle :size="13" />
                    {{ warning }}
                  </div>
                </div>

                <div v-else-if="detection" class="detect-state warning">
                  <AlertTriangle :size="16" />
                  No se reconoció el tipo automáticamente. Podrás elegirlo en el siguiente paso.
                </div>

                <div v-else-if="detectionError" class="wizard-error">
                  <AlertTriangle :size="16" />
                  <span>{{ detectionError }}</span>
                  <button class="btn btn-ghost btn-sm" type="button" @click="retryDetection">
                    <RefreshCw :size="13" />
                    Reintentar
                  </button>
                </div>

                <p v-else-if="form.cwd" class="hint">
                  No se pudo analizar la carpeta. Puedes continuar y configurar el servicio manualmente.
                </p>
              </template>

              <template v-else-if="currentStep === 'type'">
                <div class="step-head">
                  <span class="step-kicker">Paso 2</span>
                  <h3>Tipo de proyecto</h3>
                  <p>Usa la detección automática o elige el tipo manualmente.</p>
                </div>

                <div v-if="detectedAutomatically && !manualMode" class="detected-type-card">
                  <div class="detected-type-head">
                    <div class="type-icon">
                      <component :is="detectedType?.icon" v-if="detectedType" :size="21" />
                      <ScanSearch v-else :size="21" />
                    </div>
                    <div>
                      <h4>{{ detection?.displayName }}</h4>
                      <p>{{ detectedType?.label }} · {{ detectedType?.description }}</p>
                    </div>
                    <span class="badge badge-active">Detectado</span>
                  </div>

                  <div class="detected-details">
                    <div>
                      <span>Comando</span>
                      <code>{{ detection?.runCommand }}</code>
                    </div>
                    <div>
                      <span>Preparación</span>
                      <code>{{ detection?.setupCommand || 'No necesaria' }}</code>
                    </div>
                    <div>
                      <span>Puerto sugerido</span>
                      <strong>{{ form.port }}</strong>
                    </div>
                    <div>
                      <span>Categoría</span>
                      <strong>{{ categoryLabels[form.category] }}</strong>
                    </div>
                  </div>

                  <div v-for="warning in detection?.warnings ?? []" :key="warning" class="detect-warning">
                    <AlertTriangle :size="13" />
                    {{ warning }}
                  </div>

                  <div class="detected-actions">
                    <button class="btn btn-secondary btn-sm" type="button" @click="manualMode = true">
                      <Settings2 :size="13" />
                      Cambiar tipo
                    </button>
                    <button class="btn btn-ghost btn-sm" type="button" @click="retryDetection">
                      <RefreshCw :size="13" />
                      Volver a detectar
                    </button>
                  </div>
                </div>

                <div v-else class="manual-types">
                  <div v-if="!detectedAutomatically" class="info-card">
                    <AlertTriangle :size="16" />
                    <span>
                      No pudimos detectar el tipo de proyecto. Elige una opción para continuar.
                    </span>
                  </div>

                  <div class="type-grid">
                    <button
                      v-for="type in PROJECT_TYPES"
                      :key="type.id"
                      class="type-card"
                      :class="{ selected: selectedTypeId === type.id }"
                      type="button"
                      @click="applyProjectType(type)"
                    >
                      <span class="type-card-icon">
                        <component :is="type.icon" :size="18" />
                      </span>
                      <span>
                        <strong>{{ type.label }}</strong>
                        <small>{{ type.description }}</small>
                      </span>
                    </button>
                  </div>

                  <button
                    v-if="detectedAutomatically"
                    class="btn btn-ghost btn-sm"
                    type="button"
                    @click="useDetectedType"
                  >
                    <Sparkles :size="13" />
                    Usar detección automática
                  </button>
                </div>
              </template>

              <template v-else-if="currentStep === 'config'">
                <div class="step-head">
                  <span class="step-kicker">Paso 3</span>
                  <h3>Configura el servicio</h3>
                  <p>Ajusta el nombre, el comando y la red. Los valores detectados ya están aplicados.</p>
                </div>

                <div class="config-grid">
                  <div class="field">
                    <label for="wizard-name">Nombre *</label>
                    <input id="wizard-name" v-model="form.name" class="input" placeholder="Mi servicio" />
                  </div>
                  <div class="field">
                    <label for="wizard-workspace">Workspace</label>
                    <select id="wizard-workspace" v-model="form.workspaceId" class="select">
                      <option v-for="workspace in workspaces" :key="workspace.id" :value="workspace.id">
                        {{ workspace.name }}
                      </option>
                    </select>
                  </div>
                  <div class="field">
                    <label for="wizard-category">Categoría</label>
                    <select id="wizard-category" v-model="form.category" class="select">
                      <option value="frontend">Frontend</option>
                      <option value="backend">Backend</option>
                      <option value="custom">Personalizado</option>
                    </select>
                  </div>
                  <div class="field">
                    <label for="wizard-color">Color</label>
                    <div class="color-field">
                      <input id="wizard-color" v-model="form.color" class="color-input" type="color" />
                      <span class="color-value">{{ form.color }}</span>
                    </div>
                  </div>
                </div>

                <div class="config-section">
                  <h4>Ejecución</h4>
                  <div class="field">
                    <label for="wizard-command">Comando *</label>
                    <input id="wizard-command" v-model="form.command" class="input" placeholder="npm run dev" />
                  </div>
                  <div class="field">
                    <label for="wizard-args">Argumentos</label>
                    <input id="wizard-args" v-model="form.args" class="input" placeholder="--port 3000" />
                    <span class="hint">Sepáralos con espacios.</span>
                  </div>
                </div>

                <div class="config-section">
                  <h4>Red</h4>
                  <div class="field-row">
                    <div class="field">
                      <label for="wizard-url">URL</label>
                      <input
                        id="wizard-url"
                        v-model="form.url"
                        class="input"
                        placeholder="http://localhost:3000"
                      />
                    </div>
                    <div class="field">
                      <label for="wizard-port">Puerto</label>
                      <input
                        id="wizard-port"
                        v-model="form.port"
                        class="input"
                        type="number"
                        min="1"
                        max="65535"
                        placeholder="Automático"
                      />
                    </div>
                  </div>
                </div>

                <details class="advanced">
                  <summary>Avanzado: variables de entorno</summary>
                  <div class="field">
                    <textarea
                      v-model="form.env"
                      class="textarea"
                      rows="5"
                      placeholder='{ "NODE_ENV": "development", "PORT": "3000" }'
                    ></textarea>
                    <span class="hint">Debe ser un objeto JSON válido.</span>
                  </div>
                </details>
              </template>

              <template v-else-if="currentStep === 'environment'">
                <div class="step-head">
                  <span class="step-kicker">Paso 4</span>
                  <h3>Prepara el entorno</h3>
                  <p>Instala dependencias o crea el entorno virtual ahora. Este paso es opcional.</p>
                </div>

                <EnvironmentSetupPanel
                  v-model:setup-command="form.setupCommand"
                  :cwd="form.cwd"
                  @apply="applyEnvironment"
                />

                <p class="hint">
                  Puedes continuar sin ejecutar la preparación y hacerlo más tarde desde el servicio.
                </p>
              </template>

              <template v-else>
                <div class="step-head">
                  <span class="step-kicker">Paso 5</span>
                  <h3>Revisa y crea</h3>
                  <p>Verifica la configuración antes de agregar el servicio al dashboard.</p>
                </div>

                <div class="review-grid">
                  <section class="review-card">
                    <header>
                      <h4>Proyecto</h4>
                      <button type="button" @click="goToStep('folder')">Editar</button>
                    </header>
                    <dl class="review-list">
                      <div class="review-row"><dt>Carpeta</dt><dd class="mono-path">{{ form.cwd }}</dd></div>
                      <div class="review-row"><dt>Tipo</dt><dd>{{ selectedType?.label || 'Personalizado' }}</dd></div>
                      <div class="review-row"><dt>Nombre</dt><dd>{{ form.name }}</dd></div>
                      <div class="review-row">
                        <dt>Workspace</dt>
                        <dd>{{ workspaces.find((workspace) => workspace.id === form.workspaceId)?.name }}</dd>
                      </div>
                    </dl>
                  </section>

                  <section class="review-card">
                    <header>
                      <h4>Ejecución</h4>
                      <button type="button" @click="goToStep('config')">Editar</button>
                    </header>
                    <dl class="review-list">
                      <div class="review-row"><dt>Comando</dt><dd><code>{{ summaryCommand || 'Sin definir' }}</code></dd></div>
                      <div class="review-row"><dt>URL</dt><dd>{{ form.url }}</dd></div>
                      <div class="review-row"><dt>Puerto</dt><dd>{{ form.port }}</dd></div>
                      <div class="review-row"><dt>Categoría</dt><dd>{{ categoryLabels[form.category] }}</dd></div>
                    </dl>
                  </section>

                  <section class="review-card">
                    <header>
                      <h4>Entorno</h4>
                      <button type="button" @click="goToStep('environment')">Editar</button>
                    </header>
                    <dl class="review-list">
                      <div class="review-row">
                        <dt>Preparación</dt>
                        <dd><code>{{ form.setupCommand || 'No definida' }}</code></dd>
                      </div>
                      <div class="review-row"><dt>Variables</dt><dd>{{ envCount }}</dd></div>
                    </dl>
                  </section>

                  <section class="review-card">
                    <header>
                      <h4>Apariencia</h4>
                      <button type="button" @click="goToStep('config')">Editar</button>
                    </header>
                    <div class="review-color">
                      <span class="review-swatch" :style="{ background: form.color }"></span>
                      <span>{{ form.color }}</span>
                    </div>
                  </section>
                </div>
              </template>

              <p v-if="stepError" class="wizard-error" role="alert">
                <AlertTriangle :size="16" />
                {{ stepError }}
              </p>
            </section>
          </Transition>
        </div>
      </div>

      <footer class="wizard-footer">
        <button class="btn btn-secondary" type="button" @click="emit('cancel')">Cancelar</button>
        <div class="wizard-footer-actions">
          <button v-if="!isFirstStep" class="btn btn-secondary" type="button" @click="previousStep">
            <ArrowLeft :size="14" />
            Atrás
          </button>
          <button
            v-if="!isReviewStep"
            class="btn btn-primary"
            type="button"
            :disabled="!canContinue"
            @click="nextStep"
          >
            {{ currentStep === 'environment' ? 'Continuar' : 'Siguiente' }}
            <ArrowRight :size="14" />
          </button>
          <button v-else class="btn btn-primary" type="button" @click="submit">
            <Check :size="15" />
            Crear servicio
          </button>
        </div>
      </footer>
    </div>
  </div>
</template>

<style scoped>
.wizard-overlay {
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

.wizard-panel {
  display: flex;
  flex-direction: column;
  width: 100%;
  max-width: 920px;
  max-height: 90vh;
  overflow: hidden;
  border: 1px solid var(--border);
  border-radius: var(--radius-xl);
  background: var(--surface);
  box-shadow: var(--shadow-lg);
  animation: wizardIn 0.18s ease-out;
}

.wizard-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-shrink: 0;
  padding: 18px 20px;
  border-bottom: 1px solid var(--border);
}

.wizard-heading {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.wizard-icon {
  display: grid;
  place-items: center;
  width: 40px;
  height: 40px;
  flex-shrink: 0;
  border-radius: 13px;
  background: var(--primary-soft);
  color: var(--primary);
}

.wizard-heading h2 {
  margin: 0;
  color: var(--text);
  font-size: 17px;
  letter-spacing: -0.01em;
}

.wizard-heading p {
  margin: 3px 0 0;
  color: var(--text-muted);
  font-size: 12px;
}

.wizard-body {
  display: grid;
  grid-template-columns: 220px minmax(0, 1fr);
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.wizard-steps {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 16px 12px;
  overflow: auto;
  border-right: 1px solid var(--border);
  background: var(--surface-2);
}

.wizard-step {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 9px;
  border: 0;
  border-radius: 10px;
  background: transparent;
  color: var(--text-muted);
  text-align: left;
  cursor: pointer;
}

.wizard-step:disabled {
  cursor: default;
  opacity: 0.62;
}

.wizard-step.active {
  background: var(--primary-soft);
  color: var(--primary);
}

.wizard-step.completed {
  color: var(--text);
}

.wizard-step-icon {
  display: grid;
  place-items: center;
  width: 28px;
  height: 28px;
  flex-shrink: 0;
  border: 1px solid var(--border);
  border-radius: 9px;
  background: var(--surface);
}

.wizard-step.active .wizard-step-icon {
  border-color: var(--primary);
  background: var(--primary);
  color: var(--primary-contrast);
}

.wizard-step.completed .wizard-step-icon {
  border-color: var(--success);
  color: var(--success);
}

.wizard-step-copy {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}

.wizard-step-copy strong {
  font-size: 11px;
}

.wizard-step-copy small {
  overflow: hidden;
  color: var(--text-subtle);
  font-size: 9px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.wizard-content {
  min-height: 0;
  padding: 22px 24px;
  overflow: auto;
}

.wizard-step-content {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.step-head {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.step-kicker {
  color: var(--primary);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.step-head h3 {
  margin: 0;
  color: var(--text);
  font-size: 18px;
  letter-spacing: -0.02em;
}

.step-head p {
  margin: 0;
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.5;
}

.folder-picker {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 18px;
  border: 1px dashed var(--border-strong);
  border-radius: var(--radius-lg);
  background: var(--surface-2);
}

.folder-picker.has-folder {
  border-style: solid;
  border-color: var(--primary);
  background: var(--primary-soft);
}

.folder-icon {
  display: grid;
  place-items: center;
  width: 52px;
  height: 52px;
  flex-shrink: 0;
  border: 1px solid var(--border);
  border-radius: 14px;
  background: var(--surface);
  color: var(--primary);
}

.folder-copy {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
}

.folder-copy strong {
  color: var(--text);
  font-size: 14px;
}

.folder-copy span {
  overflow: hidden;
  color: var(--text-muted);
  font-size: 11px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.mono-path,
.folder-copy span {
  font-family: var(--mono);
}

.detect-state,
.wizard-error,
.info-card {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  color: var(--text-muted);
  font-size: 12px;
}

.detect-state.warning {
  border-color: var(--warning);
  background: var(--warning-soft);
  color: var(--warning);
}

.wizard-error {
  border-color: var(--danger);
  background: var(--danger-soft);
  color: var(--danger);
}

.wizard-error .btn {
  margin-left: auto;
}

.info-card {
  align-items: flex-start;
}

.detection-summary {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface);
}

.detection-title {
  display: flex;
  align-items: center;
  gap: 7px;
  color: var(--success);
  font-size: 12px;
  font-weight: 700;
}

.detection-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.detection-chips span {
  padding: 3px 8px;
  border-radius: 999px;
  background: var(--surface-2);
  color: var(--text-muted);
  font-size: 10px;
}

.detection-summary code {
  color: var(--text);
  font-family: var(--mono);
  font-size: 11px;
}

.detect-warning {
  display: flex;
  align-items: flex-start;
  gap: 6px;
  color: var(--warning);
  font-size: 11px;
}

.detected-type-card {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 16px;
  border: 1px solid var(--primary);
  border-radius: var(--radius-lg);
  background: var(--primary-soft);
}

.detected-type-head {
  display: flex;
  align-items: center;
  gap: 12px;
}

.type-icon {
  display: grid;
  place-items: center;
  width: 44px;
  height: 44px;
  flex-shrink: 0;
  border: 1px solid var(--border);
  border-radius: 12px;
  background: var(--surface);
  color: var(--primary);
}

.detected-type-head h4 {
  margin: 0;
  color: var(--text);
  font-size: 15px;
}

.detected-type-head p {
  margin: 2px 0 0;
  color: var(--text-muted);
  font-size: 11px;
}

.detected-type-head .badge {
  margin-left: auto;
}

.detected-details {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.detected-details > div {
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
}

.detected-details span {
  color: var(--text-subtle);
  font-size: 9px;
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

.detected-details code,
.detected-details strong {
  overflow-wrap: anywhere;
  color: var(--text);
  font-size: 11px;
}

.detected-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.type-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}

.type-card {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface);
  color: var(--text-muted);
  text-align: left;
  cursor: pointer;
}

.type-card:hover {
  border-color: var(--border-strong);
  background: var(--surface-2);
}

.type-card.selected {
  border-color: var(--primary);
  background: var(--primary-soft);
  color: var(--text);
  box-shadow: var(--focus-ring);
}

.type-card-icon {
  display: grid;
  place-items: center;
  width: 34px;
  height: 34px;
  flex-shrink: 0;
  border-radius: 9px;
  background: var(--surface-2);
  color: var(--primary);
}

.type-card strong {
  display: block;
  font-size: 12px;
}

.type-card small {
  display: block;
  margin-top: 2px;
  color: var(--text-subtle);
  font-size: 10px;
}

.config-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
}

.field-row {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;
}

.field label {
  color: var(--text);
  font-size: 12px;
  font-weight: 600;
}

.color-field {
  display: flex;
  align-items: center;
  gap: 10px;
}

.color-input {
  width: 46px;
  height: 38px;
  padding: 3px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface);
  cursor: pointer;
}

.color-value {
  color: var(--text-muted);
  font-family: var(--mono);
  font-size: 12px;
}

.config-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding-top: 16px;
  border-top: 1px solid var(--border);
}

.config-section h4 {
  margin: 0;
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.07em;
  text-transform: uppercase;
}

.advanced {
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface-2);
}

.advanced summary {
  padding: 11px 12px;
  color: var(--text);
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
}

.advanced[open] summary {
  border-bottom: 1px solid var(--border);
}

.advanced .field {
  padding: 12px;
}

.hint {
  color: var(--text-subtle);
  font-size: 11px;
  line-height: 1.45;
}

.review-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.review-card {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface);
}

.review-card header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.review-card h4 {
  margin: 0;
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

.review-card header button {
  padding: 0;
  border: 0;
  background: transparent;
  color: var(--primary);
  font-size: 10px;
  font-weight: 700;
  cursor: pointer;
}

.review-list {
  display: flex;
  flex-direction: column;
  gap: 7px;
  margin: 0;
}

.review-row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  font-size: 11px;
}

.review-row dt {
  flex-shrink: 0;
  color: var(--text-subtle);
}

.review-row dd {
  min-width: 0;
  margin: 0;
  overflow-wrap: anywhere;
  color: var(--text);
  text-align: right;
}

.review-row code {
  font-family: var(--mono);
  font-size: 10px;
}

.review-color {
  display: flex;
  align-items: center;
  gap: 9px;
  color: var(--text-muted);
  font-family: var(--mono);
  font-size: 11px;
}

.review-swatch {
  width: 26px;
  height: 26px;
  border: 1px solid var(--border);
  border-radius: 8px;
}

.wizard-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-shrink: 0;
  padding: 14px 20px;
  border-top: 1px solid var(--border);
  background: var(--surface);
}

.wizard-footer-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.spinning {
  animation: spin 1s linear infinite;
}

.wizard-fade-enter-active,
.wizard-fade-leave-active {
  transition:
    opacity 0.14s ease,
    transform 0.14s ease;
}

.wizard-fade-enter-from {
  opacity: 0;
  transform: translateX(6px);
}

.wizard-fade-leave-to {
  opacity: 0;
  transform: translateX(-6px);
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

@keyframes wizardIn {
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
  .wizard-overlay {
    padding: 12px;
  }

  .wizard-panel {
    max-height: 94vh;
  }

  .wizard-body {
    grid-template-columns: 1fr;
    overflow: auto;
  }

  .wizard-steps {
    flex-direction: row;
    overflow: auto;
    border-right: 0;
    border-bottom: 1px solid var(--border);
    padding: 10px;
  }

  .wizard-step {
    min-width: max-content;
    padding: 7px;
  }

  .wizard-step-copy small {
    display: none;
  }

  .wizard-content {
    padding: 18px 16px;
    overflow: visible;
  }

  .config-grid,
  .field-row,
  .review-grid,
  .detected-details {
    grid-template-columns: 1fr;
  }

  .type-grid {
    grid-template-columns: 1fr;
  }

  .wizard-footer {
    padding: 12px 16px;
  }

  .folder-picker {
    align-items: flex-start;
    flex-wrap: wrap;
  }

  .folder-picker .btn {
    width: 100%;
  }
}
</style>
