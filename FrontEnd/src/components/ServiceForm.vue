<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { AlertTriangle, FolderOpen, Globe, Info, Palette, Save, Settings2, Terminal, Wrench, X } from '@lucide/vue'
import EnvironmentSetupPanel from './EnvironmentSetupPanel.vue'
import { projectTypeForPreset } from '../utils/projectTypes'
import { defaultPortForPreset, normalizeUrl, portFromCommand, portFromUrl } from '../utils/serviceDefaults'
import type {
  EnvironmentInfo,
  Service,
  ServiceCategory,
  ServiceInput,
  ServicePreset,
  Workspace,
} from '../types'

interface Props {
  service: Service | null
  workspaces: Workspace[]
  activeWorkspaceId: string
}

interface Emits {
  (e: 'save', input: ServiceInput): void
  (e: 'cancel'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const form = ref({
  workspaceId: '',
  name: '',
  category: 'frontend' as ServiceCategory,
  preset: 'vue' as ServicePreset,
  cwd: '',
  command: '',
  args: '' as string,
  setupCommand: '',
  env: '' as string,
  url: '',
  port: '' as string,
  color: '#3b82f6',
})

const formError = ref('')
const lastDefaultPort = ref(defaultPortForPreset('vue'))

const presetOptions = [
  { value: 'vue', label: 'Node / Vue (npm/pnpm/yarn)' },
  { value: 'spring-boot', label: 'Spring Boot (mvnw/gradlew)' },
  { value: 'python', label: 'Python (Django/FastAPI/Flask)' },
  { value: 'dotnet', label: '.NET (dotnet CLI)' },
  { value: 'rust', label: 'Rust (Cargo)' },
  { value: 'go', label: 'Go (Go modules)' },
  { value: 'custom', label: 'Personalizado' },
]

const isEditing = computed(() => props.service !== null)

function resetForm() {
  formError.value = ''
  const port = defaultPortForPreset('vue')
  lastDefaultPort.value = port
  form.value = {
    workspaceId: props.activeWorkspaceId,
    name: '',
    category: 'frontend',
    preset: 'vue',
    cwd: '',
    command: '',
    args: '',
    setupCommand: '',
    env: '',
    url: `http://localhost:${port}`,
    port: String(port),
    color: '#3b82f6',
  }
}

watch(() => props.service, (service) => {
  formError.value = ''
  if (service) {
    form.value.workspaceId = service.workspaceId || props.activeWorkspaceId
    form.value.name = service.name
    form.value.category = service.category
    form.value.preset = service.preset
    form.value.cwd = service.cwd ?? ''
    form.value.command = service.command
    form.value.args = service.args?.join(' ') ?? ''
    form.value.setupCommand = service.setupCommand ?? ''
    form.value.env = JSON.stringify(service.env ?? {}, null, 2)
    form.value.color = service.color ?? '#3b82f6'

    const port = service.port
      ?? portFromUrl(service.url)
      ?? portFromCommand(service.command, service.args)
      ?? defaultPortForPreset(service.preset)
    form.value.port = String(port)
    form.value.url = normalizeUrl(service.url) || `http://localhost:${port}`
    lastDefaultPort.value = defaultPortForPreset(service.preset)
  } else {
    resetForm()
  }
}, { immediate: true, deep: true })

function onPresetChange() {
  const preset = form.value.preset

  if (preset === 'vue') {
    form.value.category = 'frontend'
    form.value.command = 'npm run dev'
    form.value.args = ''
  } else if (preset === 'spring-boot') {
    form.value.category = 'backend'
    form.value.command = 'mvnw.cmd spring-boot:run'
    form.value.args = ''
  } else if (preset === 'python') {
    form.value.category = 'backend'
    form.value.command = 'python manage.py runserver'
    form.value.args = ''
  } else if (preset === 'dotnet') {
    form.value.category = 'backend'
    form.value.command = 'dotnet run'
    form.value.args = ''
  } else if (preset === 'rust') {
    form.value.category = 'backend'
    form.value.command = 'cargo run'
    form.value.args = ''
  } else if (preset === 'go') {
    form.value.category = 'backend'
    form.value.command = 'go run .'
    form.value.args = ''
  } else {
    form.value.command = ''
    form.value.args = ''
  }

  const nextDefaultPort = defaultPortForPreset(preset)
  const currentPort = form.value.port ? Number(form.value.port) : undefined
  const shouldUpdatePort = !currentPort || currentPort === lastDefaultPort.value
  const nextPort = shouldUpdatePort ? nextDefaultPort : currentPort
  const currentUrl = form.value.url.trim()
  const shouldUpdateUrl = !currentUrl
    || normalizeUrl(currentUrl) === `http://localhost:${lastDefaultPort.value}`

  if (shouldUpdatePort) form.value.port = String(nextPort)
  if (shouldUpdateUrl) form.value.url = `http://localhost:${nextPort}`
  lastDefaultPort.value = nextDefaultPort
}

function onEnvironmentDetected(info: EnvironmentInfo) {
  if (form.value.command.trim()) return

  if (info.suggestedPreset) {
    const preset = info.suggestedPreset as ServicePreset
    form.value.preset = preset
    const type = projectTypeForPreset(preset)
    if (type) form.value.category = type.category
  }
  if (info.runCommand) {
    form.value.command = info.runCommand
    form.value.args = ''
  }
  if (info.suggestedPort) {
    form.value.port = String(info.suggestedPort)
    form.value.url = `http://localhost:${info.suggestedPort}`
    lastDefaultPort.value = info.suggestedPort
  }
}

function applyEnvironment(info: EnvironmentInfo) {
  if (info.runCommand) form.value.command = info.runCommand
  if (info.runCommand) form.value.args = ''
  if (info.setupCommand) form.value.setupCommand = info.setupCommand
  if (info.suggestedPreset) {
    const preset = info.suggestedPreset as ServicePreset
    form.value.preset = preset
    const type = projectTypeForPreset(preset)
    if (type) form.value.category = type.category
  }
  if (info.suggestedPort) {
    form.value.port = String(info.suggestedPort)
    form.value.url = `http://localhost:${info.suggestedPort}`
    lastDefaultPort.value = info.suggestedPort
  }
}

async function pickFolder() {
  try {
    const path = await open({
      title: 'Seleccionar carpeta del servicio',
      directory: true,
      multiple: false,
    })
    if (typeof path === 'string') form.value.cwd = path
  } catch (error) {
    console.error('Error seleccionando carpeta:', error)
  }
}

function validateEnv() {
  try {
    JSON.parse(form.value.env || '{}')
    return true
  } catch {
    return false
  }
}

function submit() {
  formError.value = ''
  if (!form.value.name.trim()) {
    formError.value = 'El nombre es obligatorio'
    return
  }
  if (!form.value.command.trim()) {
    formError.value = 'El comando es obligatorio'
    return
  }
  if (!validateEnv()) {
    formError.value = 'Variables de entorno: JSON inválido'
    return
  }

  const command = form.value.command.trim()
  const args = form.value.args.trim() ? form.value.args.trim().split(/\s+/) : []
  const portInput = form.value.port ? Number(form.value.port) : undefined
  if (portInput !== undefined && (!Number.isInteger(portInput) || portInput < 1 || portInput > 65535)) {
    formError.value = 'El puerto debe estar entre 1 y 65535'
    return
  }

  const port = portInput
    ?? portFromUrl(form.value.url)
    ?? portFromCommand(command, args)
    ?? defaultPortForPreset(form.value.preset)
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

function cancel() {
  emit('cancel')
}
</script>

<template>
  <div class="form-overlay" @click="cancel">
    <div class="form-panel" role="dialog" aria-modal="true" aria-labelledby="service-form-title" @click.stop>
      <div class="form-header">
        <div class="form-title">
          <div class="form-icon">
            <Settings2 :size="19" />
          </div>
          <div>
            <h3 id="service-form-title">{{ isEditing ? 'Editar servicio' : 'Nuevo servicio' }}</h3>
            <p>{{ isEditing ? 'Actualiza la configuración del servicio' : 'Solo necesitas un nombre y un comando' }}</p>
          </div>
        </div>
        <button class="icon-btn" type="button" aria-label="Cerrar" @click="cancel">
          <X :size="18" />
        </button>
      </div>

      <form autocomplete="off" @submit.prevent="submit">
        <section class="form-section">
          <div class="section-title">
            <Info :size="14" />
            Información general
          </div>

          <div class="field-row">
            <div class="field">
              <label for="service-name">Nombre *</label>
              <input id="service-name" v-model="form.name" class="input" placeholder="Mi servicio Vue" />
            </div>
            <div class="field">
              <label for="service-category">Categoría</label>
              <select id="service-category" v-model="form.category" class="select">
                <option value="frontend">Frontend</option>
                <option value="backend">Backend</option>
                <option value="custom">Personalizado</option>
              </select>
            </div>
          </div>

          <div class="field">
            <label for="service-workspace">Workspace</label>
            <select id="service-workspace" v-model="form.workspaceId" class="select">
              <option v-for="workspace in workspaces" :key="workspace.id" :value="workspace.id">
                {{ workspace.name }}
              </option>
            </select>
          </div>

          <div class="field">
            <label for="service-preset">Preset</label>
            <select id="service-preset" v-model="form.preset" class="select" @change="onPresetChange">
              <option v-for="option in presetOptions" :key="option.value" :value="option.value">
                {{ option.label }}
              </option>
            </select>
          </div>
        </section>

        <section class="form-section">
          <div class="section-title">
            <Terminal :size="14" />
            Ejecución
          </div>

          <div class="field">
            <label for="service-cwd">Carpeta de trabajo</label>
            <div class="input-with-btn">
              <input id="service-cwd" v-model="form.cwd" class="input" placeholder="C:\\Proyectos\\mi-app" readonly />
              <button class="btn btn-secondary btn-sm" type="button" @click="pickFolder">
                <FolderOpen :size="14" />
                Examinar
              </button>
            </div>
            <span class="hint">Opcional. Si se deja vacía, se usará tu carpeta de usuario.</span>
          </div>

          <div class="field">
            <label for="service-command">Comando *</label>
            <input id="service-command" v-model="form.command" class="input" placeholder="npm run dev" />
          </div>

          <div class="field">
            <label for="service-args">Argumentos</label>
            <input id="service-args" v-model="form.args" class="input" placeholder="--port 3000" />
            <span class="hint">Sepáralos con espacios.</span>
          </div>

          <div class="field">
            <label for="service-env">Variables de entorno (JSON)</label>
            <textarea
              id="service-env"
              v-model="form.env"
              class="textarea"
              rows="4"
              placeholder='{ "NODE_ENV": "development", "PORT": "3000" }'
            />
            <span class="hint">Ejemplo: { "JAVA_HOME": "C:\\Java\\jdk-21", "SPRING_PROFILES_ACTIVE": "dev" }</span>
          </div>
        </section>

        <section class="form-section">
          <div class="section-title">
            <Wrench :size="14" />
            Entorno
          </div>
          <EnvironmentSetupPanel
            v-model:setup-command="form.setupCommand"
            :cwd="form.cwd"
            @apply="applyEnvironment"
            @detected="onEnvironmentDetected"
          />
        </section>

        <section class="form-section">
          <div class="section-title">
            <Globe :size="14" />
            Red y URL
          </div>

          <div class="field-row">
            <div class="field">
              <label for="service-url">URL</label>
              <input id="service-url" v-model="form.url" class="input" type="text" placeholder="http://localhost:3000" />
            </div>
            <div class="field">
              <label for="service-port">Puerto</label>
              <input
                id="service-port"
                v-model="form.port"
                class="input"
                type="number"
                placeholder="Automático"
                min="1"
                max="65535"
              />
            </div>
          </div>
          <span class="hint">Si dejas URL o puerto vacíos, se completan automáticamente según el preset.</span>
        </section>

        <section class="form-section">
          <div class="section-title">
            <Palette :size="14" />
            Apariencia
          </div>

          <div class="field">
            <label for="service-color">Color del servicio</label>
            <div class="color-field">
              <input id="service-color" v-model="form.color" class="color-input" type="color" />
              <span class="color-value">{{ form.color }}</span>
            </div>
          </div>
        </section>

        <p v-if="formError" class="form-error" role="alert">
          <AlertTriangle :size="16" />
          {{ formError }}
        </p>

        <div class="form-actions">
          <button class="btn btn-secondary" type="button" @click="cancel">
            <X :size="15" />
            Cancelar
          </button>
          <button class="btn btn-primary" type="submit">
            <Save :size="15" />
            {{ isEditing ? 'Guardar cambios' : 'Crear servicio' }}
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<style scoped>
.form-overlay {
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

.form-panel {
  width: 100%;
  max-width: 660px;
  max-height: 90vh;
  overflow: auto;
  border: 1px solid var(--border);
  border-radius: var(--radius-xl);
  background: var(--surface);
  box-shadow: var(--shadow-lg);
  animation: modalIn 0.18s ease-out;
}

.form-header {
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

.form-title {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.form-icon {
  display: grid;
  place-items: center;
  width: 40px;
  height: 40px;
  border-radius: 13px;
  background: var(--primary-soft);
  color: var(--primary);
}

.form-title h3 {
  margin: 0;
  color: var(--text);
  font-size: 16px;
  letter-spacing: -0.01em;
}

.form-title p {
  margin: 3px 0 0;
  color: var(--text-muted);
  font-size: 12px;
}

form {
  display: flex;
  flex-direction: column;
  gap: 20px;
  padding: 20px;
}

.form-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.form-section + .form-section {
  padding-top: 18px;
  border-top: 1px solid var(--border);
}

.section-title {
  display: flex;
  align-items: center;
  gap: 7px;
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.field-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
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

.hint {
  color: var(--text-subtle);
  font-size: 11px;
  line-height: 1.45;
}

.input-with-btn {
  display: flex;
  gap: 8px;
}

.input-with-btn .input {
  flex: 1;
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

.form-error {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
  padding: 11px 12px;
  border: 1px solid var(--danger);
  border-radius: var(--radius-sm);
  background: var(--danger-soft);
  color: var(--danger);
  font-size: 12px;
  font-weight: 600;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding-top: 16px;
  border-top: 1px solid var(--border);
}

@media (max-width: 620px) {
  .form-overlay {
    padding: 12px;
  }

  .form-panel {
    max-height: 94vh;
  }

  .field-row {
    grid-template-columns: 1fr;
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
