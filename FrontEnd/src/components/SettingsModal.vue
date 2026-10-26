<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import {
  Check,
  FolderOpen,
  HeartPulse,
  MonitorCog,
  Moon,
  Palette as PaletteIcon,
  SlidersHorizontal,
  Sun,
  X,
} from '@lucide/vue'
import pkg from '../../package.json'
import { useDashboardLog } from '../composables/useDashboardLog'
import { useSettings } from '../composables/useServices'
import { useTheme } from '../composables/useTheme'
import type { Settings } from '../types'

interface Emits {
  (e: 'close'): void
  (e: 'saved', settings: Settings): void
}

type SectionId = 'appearance' | 'general' | 'monitoring' | 'system'

const emit = defineEmits<Emits>()
const { theme, palette, palettes, setTheme, setPalette } = useTheme()
const { load: loadSettings, save: saveSettings } = useSettings()
const { openFolder } = useDashboardLog()

const sections = [
  {
    id: 'appearance',
    label: 'Apariencia',
    description: 'Modo y paleta',
    icon: PaletteIcon,
  },
  {
    id: 'general',
    label: 'General',
    description: 'Comportamiento',
    icon: SlidersHorizontal,
  },
  {
    id: 'monitoring',
    label: 'Monitoreo',
    description: 'Salud y chequeos',
    icon: HeartPulse,
  },
  {
    id: 'system',
    label: 'Sistema',
    description: 'Historial y versión',
    icon: MonitorCog,
  },
] as const

const activeSection = ref<SectionId>('appearance')
const closeButton = ref<HTMLButtonElement | null>(null)
const settingsContent = ref<HTMLElement | null>(null)
const settings = ref<Settings>({
  confirmOnExit: true,
  logRetentionRuns: 10,
  logRetentionDays: 30,
  healthCheckEnabled: true,
  healthCheckIntervalSecs: 10,
  healthCheckTimeoutSecs: 3,
  healthCheckFailures: 3,
  healthCheckStartupGraceSecs: 15,
})
const settingsLoading = ref(true)
const settingsSaving = ref(false)
const settingsStatus = ref<'idle' | 'saved' | 'error'>('idle')
const settingsError = ref('')
const version = pkg.version
let saveTimer: number | undefined
let statusTimer: number | undefined

const saveLabel = computed(() => {
  if (settingsSaving.value) return 'Guardando cambios...'
  if (settingsStatus.value === 'saved') return 'Cambios guardados'
  if (settingsStatus.value === 'error') return settingsError.value || 'No se pudieron guardar los cambios'
  return ''
})

function errorText(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

function selectSection(id: SectionId) {
  activeSection.value = id
}

function onNavKeydown(event: KeyboardEvent, index: number) {
  if (!['ArrowDown', 'ArrowUp', 'ArrowRight', 'ArrowLeft', 'Home', 'End'].includes(event.key)) {
    return
  }

  event.preventDefault()
  let nextIndex = index
  if (event.key === 'Home') nextIndex = 0
  else if (event.key === 'End') nextIndex = sections.length - 1
  else if (event.key === 'ArrowDown' || event.key === 'ArrowRight') nextIndex = (index + 1) % sections.length
  else nextIndex = (index - 1 + sections.length) % sections.length

  const next = sections[nextIndex]
  selectSection(next.id)
  document.getElementById(`settings-tab-${next.id}`)?.focus()
}

function onThemeKeydown(event: KeyboardEvent, index: number) {
  if (!['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) return

  event.preventDefault()
  let nextIndex = index
  if (event.key === 'Home') nextIndex = 0
  else if (event.key === 'End') nextIndex = palettes.length - 1
  else if (event.key === 'ArrowDown') nextIndex = (index + 1) % palettes.length
  else nextIndex = (index - 1 + palettes.length) % palettes.length

  const next = palettes[nextIndex]
  setPalette(next.id)
  document.getElementById(`theme-option-${next.id}`)?.focus()
}

async function loadPreferences() {
  settingsLoading.value = true
  settingsStatus.value = 'idle'
  settingsError.value = ''

  try {
    settings.value = await loadSettings()
  } catch (error) {
    settingsStatus.value = 'error'
    settingsError.value = errorText(error)
  } finally {
    settingsLoading.value = false
  }
}

async function persistSettings() {
  if (settingsLoading.value) return
  if (saveTimer !== undefined) {
    window.clearTimeout(saveTimer)
    saveTimer = undefined
  }
  if (statusTimer !== undefined) {
    window.clearTimeout(statusTimer)
    statusTimer = undefined
  }

  settingsSaving.value = true
  settingsStatus.value = 'idle'
  settingsError.value = ''

  try {
    const snapshot = { ...settings.value }
    await saveSettings(snapshot)
    emit('saved', snapshot)
    settingsStatus.value = 'saved'
    statusTimer = window.setTimeout(() => {
      if (settingsStatus.value === 'saved') settingsStatus.value = 'idle'
    }, 2200)
  } catch (error) {
    settingsStatus.value = 'error'
    settingsError.value = errorText(error)
  } finally {
    settingsSaving.value = false
  }
}

function queueSave() {
  if (settingsLoading.value) return
  settingsStatus.value = 'idle'
  settingsError.value = ''
  if (saveTimer !== undefined) window.clearTimeout(saveTimer)
  saveTimer = window.setTimeout(() => {
    saveTimer = undefined
    void persistSettings()
  }, 280)
}

function toggleConfirmExit() {
  settings.value.confirmOnExit = !settings.value.confirmOnExit
  queueSave()
}

function toggleHealthCheck() {
  settings.value.healthCheckEnabled = !settings.value.healthCheckEnabled
  queueSave()
}

function updateNumber(
  field:
    | 'logRetentionRuns'
    | 'logRetentionDays'
    | 'healthCheckIntervalSecs'
    | 'healthCheckTimeoutSecs'
    | 'healthCheckFailures'
    | 'healthCheckStartupGraceSecs',
  event: Event,
  min: number,
  max: number,
) {
  const input = event.target as HTMLInputElement
  const parsed = Number.parseInt(input.value, 10)
  const next = Number.isFinite(parsed) ? Math.min(Math.max(parsed, min), max) : min
  settings.value[field] = next
  input.value = String(next)
  queueSave()
}

function updateRetention(field: 'logRetentionRuns' | 'logRetentionDays', event: Event) {
  updateNumber(field, event, 1, field === 'logRetentionRuns' ? 1000 : 3650)
}

async function openLogsFolder() {
  settingsStatus.value = 'idle'
  settingsError.value = ''

  try {
    await openFolder()
  } catch (error) {
    settingsStatus.value = 'error'
    settingsError.value = errorText(error)
  }
}

async function revealActiveTheme() {
  await nextTick()
  await new Promise<void>((resolve) => {
    requestAnimationFrame(() => requestAnimationFrame(() => resolve()))
  })
  const container = settingsContent.value
  const target = document.getElementById(`theme-option-${palette.value}`)
  if (!container || !target) return

  const containerRect = container.getBoundingClientRect()
  const targetRect = target.getBoundingClientRect()
  const targetTop = targetRect.top - containerRect.top + container.scrollTop
  const targetBottom = targetTop + target.offsetHeight
  const padding = 18

  if (targetTop < container.scrollTop + padding) {
    container.scrollTop = Math.max(0, targetTop - padding)
  } else if (targetBottom > container.scrollTop + container.clientHeight - padding) {
    container.scrollTop = targetBottom - container.clientHeight + padding
  }
}

watch(activeSection, (section) => {
  if (section === 'appearance') void revealActiveTheme()
})

onMounted(async () => {
  closeButton.value?.focus()
  await loadPreferences()
  void revealActiveTheme()
})

onBeforeUnmount(() => {
  if (saveTimer !== undefined) {
    window.clearTimeout(saveTimer)
    saveTimer = undefined
    void persistSettings()
  }
  if (statusTimer !== undefined) window.clearTimeout(statusTimer)
})
</script>

<template>
  <div class="settings-overlay" @click="$emit('close')">
    <div
      class="settings-panel"
      role="dialog"
      aria-modal="true"
      aria-labelledby="settings-title"
      @click.stop
    >
      <header class="settings-header">
        <div class="settings-title">
          <div class="settings-icon"><MonitorCog :size="19" /></div>
          <div>
            <h2 id="settings-title">Configuración</h2>
            <p>Apariencia, comportamiento y sistema de la aplicación.</p>
          </div>
        </div>

        <button
          ref="closeButton"
          class="settings-close"
          type="button"
          title="Cerrar"
          aria-label="Cerrar configuración"
          @click="$emit('close')"
        >
          <X :size="17" />
        </button>
      </header>

      <div class="settings-shell">
        <aside class="settings-nav" role="tablist" aria-label="Categorías de configuración" aria-orientation="vertical">
          <button
            v-for="(section, index) in sections"
            :id="`settings-tab-${section.id}`"
            :key="section.id"
            class="settings-nav-item"
            :class="{ 'is-active': activeSection === section.id }"
            type="button"
            role="tab"
            :aria-selected="activeSection === section.id"
            :aria-controls="`settings-panel-${section.id}`"
            :tabindex="activeSection === section.id ? 0 : -1"
            @click="selectSection(section.id)"
            @keydown="onNavKeydown($event, index)"
          >
            <span class="settings-nav-icon">
              <component :is="section.icon" :size="16" />
            </span>
            <span class="settings-nav-copy">
              <strong>{{ section.label }}</strong>
              <small>{{ section.description }}</small>
            </span>
          </button>
        </aside>

        <div ref="settingsContent" class="settings-content">
          <section
            v-if="activeSection === 'appearance'"
            id="settings-panel-appearance"
            class="settings-section"
            role="tabpanel"
            aria-labelledby="settings-tab-appearance"
          >
            <header class="settings-section-head">
              <div class="settings-section-icon"><PaletteIcon :size="18" /></div>
              <div>
                <h3>Apariencia</h3>
                <p>Elige un tema y el modo; se aplica al fondo, paneles y tarjetas.</p>
              </div>
            </header>

            <div class="appearance-panel">
              <div class="mode-row">
                <div class="mode-copy">
                  <span class="appearance-label">Modo</span>
                  <p>Define el contraste general de la interfaz.</p>
                </div>
                <div class="mode-switch" role="group" aria-label="Modo de color">
                  <button
                    class="mode-button"
                    :class="{ 'is-active': theme === 'light' }"
                    type="button"
                    :aria-pressed="theme === 'light'"
                    @click="setTheme('light')"
                  >
                    <Sun :size="14" />
                    Claro
                  </button>
                  <button
                    class="mode-button"
                    :class="{ 'is-active': theme === 'dark' }"
                    type="button"
                    :aria-pressed="theme === 'dark'"
                    @click="setTheme('dark')"
                  >
                    <Moon :size="14" />
                    Oscuro
                  </button>
                </div>
              </div>

              <div class="theme-section">
                <div class="theme-section-head">
                  <span class="appearance-label">Temas</span>
                  <span class="theme-count">{{ palettes.length }} disponibles</span>
                </div>

                <div class="theme-list" role="radiogroup" aria-label="Temas de color">
                  <button
                    v-for="(option, index) in palettes"
                    :id="`theme-option-${option.id}`"
                    :key="option.id"
                    class="theme-option"
                    :class="{ 'is-active': palette === option.id }"
                    type="button"
                    role="radio"
                    :aria-checked="palette === option.id"
                    :aria-label="`Tema ${option.label}: ${option.description}`"
                    :title="option.description"
                    :tabindex="palette === option.id ? 0 : -1"
                    @click="setPalette(option.id)"
                    @keydown="onThemeKeydown($event, index)"
                  >
                    <span class="theme-preview" aria-hidden="true">
                      <span
                        v-for="(color, swatchIndex) in option.swatches"
                        :key="swatchIndex"
                        class="theme-swatch"
                        :style="{ background: color }"
                      ></span>
                    </span>

                    <span class="theme-copy">
                      <strong>{{ option.label }}</strong>
                      <small>{{ option.description }}</small>
                    </span>

                    <span class="theme-check" :class="{ 'is-selected': palette === option.id }" aria-hidden="true">
                      <Check v-if="palette === option.id" :size="13" />
                    </span>
                  </button>
                </div>
              </div>
            </div>
          </section>

          <section
            v-else-if="activeSection === 'general'"
            id="settings-panel-general"
            class="settings-section"
            role="tabpanel"
            aria-labelledby="settings-tab-general"
          >
            <header class="settings-section-head">
              <div class="settings-section-icon"><SlidersHorizontal :size="18" /></div>
              <div>
                <h3>General</h3>
                <p>Preferencias de comportamiento que se guardan automáticamente.</p>
              </div>
            </header>

            <div class="setting-list">
              <div class="setting-row">
                <div class="setting-copy">
                  <strong>Confirmar al salir</strong>
                  <p>Pide confirmación antes de cerrar la aplicación.</p>
                </div>
                <button
                  class="switch"
                  :class="{ 'is-on': settings.confirmOnExit }"
                  type="button"
                  role="switch"
                  :aria-checked="settings.confirmOnExit"
                  :disabled="settingsLoading"
                  aria-label="Confirmar al salir"
                  @click="toggleConfirmExit"
                >
                  <span class="switch-thumb"></span>
                </button>
              </div>
            </div>
          </section>

          <section
            v-else-if="activeSection === 'monitoring'"
            id="settings-panel-monitoring"
            class="settings-section"
            role="tabpanel"
            aria-labelledby="settings-tab-monitoring"
          >
            <header class="settings-section-head">
              <div class="settings-section-icon"><HeartPulse :size="18" /></div>
              <div>
                <h3>Monitoreo</h3>
                <p>Comprueba si los servicios responden de verdad, no solo si el proceso sigue vivo.</p>
              </div>
            </header>

            <div class="setting-list">
              <div class="setting-row">
                <div class="setting-copy">
                  <strong>Comprobar salud</strong>
                  <p>Activa los chequeos periódicos de disponibilidad.</p>
                </div>
                <button
                  class="switch"
                  :class="{ 'is-on': settings.healthCheckEnabled }"
                  type="button"
                  role="switch"
                  :aria-checked="settings.healthCheckEnabled"
                  :disabled="settingsLoading"
                  aria-label="Comprobar salud de los servicios"
                  @click="toggleHealthCheck"
                >
                  <span class="switch-thumb"></span>
                </button>
              </div>

              <div class="setting-row">
                <div class="setting-copy">
                  <strong>Intervalo</strong>
                  <p>Segundos entre comprobaciones de cada servicio.</p>
                </div>
                <div class="input-with-unit">
                  <input
                    class="number-input"
                    type="number"
                    min="5"
                    max="300"
                    :value="settings.healthCheckIntervalSecs"
                    :disabled="settingsLoading || !settings.healthCheckEnabled"
                    aria-label="Intervalo de comprobación en segundos"
                    @change="updateNumber('healthCheckIntervalSecs', $event, 5, 300)"
                  />
                  <span>s</span>
                </div>
              </div>

              <div class="setting-row">
                <div class="setting-copy">
                  <strong>Tiempo de espera</strong>
                  <p>Máximo que puede tardar una comprobación.</p>
                </div>
                <div class="input-with-unit">
                  <input
                    class="number-input"
                    type="number"
                    min="1"
                    max="60"
                    :value="settings.healthCheckTimeoutSecs"
                    :disabled="settingsLoading || !settings.healthCheckEnabled"
                    aria-label="Tiempo de espera en segundos"
                    @change="updateNumber('healthCheckTimeoutSecs', $event, 1, 60)"
                  />
                  <span>s</span>
                </div>
              </div>

              <div class="setting-row">
                <div class="setting-copy">
                  <strong>Fallos permitidos</strong>
                  <p>Fallos consecutivos antes de marcar el servicio como caído.</p>
                </div>
                <div class="input-with-unit">
                  <input
                    class="number-input"
                    type="number"
                    min="1"
                    max="20"
                    :value="settings.healthCheckFailures"
                    :disabled="settingsLoading || !settings.healthCheckEnabled"
                    aria-label="Fallos consecutivos permitidos"
                    @change="updateNumber('healthCheckFailures', $event, 1, 20)"
                  />
                  <span>fallos</span>
                </div>
              </div>

              <div class="setting-row">
                <div class="setting-copy">
                  <strong>Gracia de arranque</strong>
                  <p>Tiempo antes de empezar a comprobar un servicio recién iniciado.</p>
                </div>
                <div class="input-with-unit">
                  <input
                    class="number-input"
                    type="number"
                    min="0"
                    max="300"
                    :value="settings.healthCheckStartupGraceSecs"
                    :disabled="settingsLoading || !settings.healthCheckEnabled"
                    aria-label="Gracia de arranque en segundos"
                    @change="updateNumber('healthCheckStartupGraceSecs', $event, 0, 300)"
                  />
                  <span>s</span>
                </div>
              </div>
            </div>

            <p class="monitoring-note">
              Los servicios con URL se comprueban por HTTP; los que solo tienen puerto, por TCP; el resto,
              por proceso.
            </p>
          </section>

          <section
            v-else
            id="settings-panel-system"
            class="settings-section"
            role="tabpanel"
            aria-labelledby="settings-tab-system"
          >
            <header class="settings-section-head">
              <div class="settings-section-icon"><MonitorCog :size="18" /></div>
              <div>
                <h3>Sistema</h3>
                <p>Controla el historial de ejecuciones y consulta la versión instalada.</p>
              </div>
            </header>

            <div class="setting-list">
              <div class="setting-row">
                <div class="setting-copy">
                  <strong>Máximo de ejecuciones</strong>
                  <p>Se conservan las ejecuciones más recientes de cada servicio.</p>
                </div>
                <input
                  class="number-input"
                  type="number"
                  min="1"
                  max="1000"
                  :value="settings.logRetentionRuns"
                  :disabled="settingsLoading"
                  aria-label="Máximo de ejecuciones por servicio"
                  @change="updateRetention('logRetentionRuns', $event)"
                />
              </div>

              <div class="setting-row">
                <div class="setting-copy">
                  <strong>Días de retención</strong>
                  <p>Las ejecuciones más antiguas se eliminan al detener un servicio.</p>
                </div>
                <input
                  class="number-input"
                  type="number"
                  min="1"
                  max="3650"
                  :value="settings.logRetentionDays"
                  :disabled="settingsLoading"
                  aria-label="Días de retención de registros"
                  @change="updateRetention('logRetentionDays', $event)"
                />
              </div>
            </div>

            <div class="system-meta">
              <div>
                <span>Versión</span>
                <strong>Cardumen Deck v{{ version }}</strong>
              </div>
              <button class="btn btn-secondary btn-sm" type="button" @click="openLogsFolder">
                <FolderOpen :size="14" />
                Abrir carpeta de registros
              </button>
            </div>
          </section>

          <p
            v-if="saveLabel"
            class="settings-save-state"
            :class="{ 'is-error': settingsStatus === 'error', 'is-saved': settingsStatus === 'saved' }"
            role="status"
          >
            {{ saveLabel }}
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-overlay {
  position: fixed;
  inset: 0;
  z-index: 1100;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  overflow: auto;
  background: var(--overlay);
  backdrop-filter: blur(6px);
}

.settings-panel {
  display: flex;
  flex-direction: column;
  width: 100%;
  max-width: min(980px, calc(100vw - 48px));
  height: min(720px, calc(100vh - 48px));
  max-height: calc(100vh - 48px);
  min-width: 0;
  overflow: hidden;
  border: 1px solid var(--border);
  border-radius: var(--radius-xl);
  background: var(--surface);
  box-shadow: var(--shadow-lg);
  animation: settingsIn 180ms ease-out;
}

.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  flex-shrink: 0;
  padding: 18px 20px;
  border-bottom: 1px solid var(--border);
}

.settings-title {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.settings-icon {
  display: grid;
  place-items: center;
  width: 40px;
  height: 40px;
  flex-shrink: 0;
  border-radius: 12px;
  background: var(--primary-soft);
  color: var(--primary);
}

.settings-title h2 {
  margin: 0;
  color: var(--text);
  font-size: 17px;
  letter-spacing: -0.01em;
}

.settings-title p {
  margin: 3px 0 0;
  color: var(--text-muted);
  font-size: 12px;
}

.settings-close {
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

.settings-close:hover {
  border-color: var(--border-strong);
  background: var(--surface-2);
  color: var(--text);
}

.settings-close:focus-visible {
  outline: none;
  box-shadow: var(--focus-ring);
}

.settings-shell {
  display: grid;
  grid-template-columns: 230px minmax(0, 1fr);
  min-width: 0;
  min-height: 0;
  flex: 1;
}

.settings-nav {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-height: 0;
  padding: 14px;
  overflow-y: auto;
  border-right: 1px solid var(--border);
  background: var(--surface-2);
}

.settings-nav-item {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  width: 100%;
  padding: 10px;
  border: 1px solid transparent;
  border-radius: 10px;
  background: transparent;
  color: var(--text-muted);
  text-align: left;
  cursor: pointer;
  transition:
    background var(--transition),
    border-color var(--transition),
    color var(--transition);
}

.settings-nav-item:hover {
  border-color: var(--border);
  background: var(--surface);
  color: var(--text);
}

.settings-nav-item.is-active {
  border-color: var(--border);
  background: var(--primary-soft);
  color: var(--text);
  box-shadow: inset 3px 0 0 var(--primary);
}

.settings-nav-item:focus-visible {
  outline: none;
  box-shadow: var(--focus-ring);
}

.settings-nav-icon {
  display: grid;
  place-items: center;
  width: 30px;
  height: 30px;
  flex-shrink: 0;
  border-radius: 8px;
  background: var(--surface);
  color: var(--text-subtle);
}

.settings-nav-item.is-active .settings-nav-icon {
  background: var(--surface);
  color: var(--primary);
}

.settings-nav-copy {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.settings-nav-copy strong {
  font-size: 12px;
  font-weight: 700;
}

.settings-nav-copy small {
  color: var(--text-subtle);
  font-size: 10px;
  line-height: 1.35;
}

.settings-content {
  min-width: 0;
  min-height: 0;
  padding: 24px 26px;
  overflow-y: auto;
}

.settings-section {
  display: flex;
  flex-direction: column;
  gap: 20px;
  animation: sectionIn 160ms ease-out;
}

.settings-section-head {
  display: flex;
  align-items: center;
  gap: 12px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--border);
}

.settings-section-icon {
  display: grid;
  place-items: center;
  width: 40px;
  height: 40px;
  flex-shrink: 0;
  border-radius: 12px;
  background: var(--primary-soft);
  color: var(--primary);
}

.settings-section-head h3 {
  margin: 0;
  color: var(--text);
  font-size: 16px;
  letter-spacing: -0.01em;
}

.settings-section-head p {
  margin: 3px 0 0;
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.45;
}

.appearance-panel {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.mode-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 18px;
  padding: 12px 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface-2);
}

.mode-copy {
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
}

.mode-copy p {
  margin: 0;
  color: var(--text-muted);
  font-size: 11px;
  line-height: 1.4;
}

.appearance-label {
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 700;
}

.mode-switch {
  display: flex;
  width: 210px;
  flex-shrink: 0;
  gap: 4px;
  padding: 4px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--surface);
}

.mode-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  flex: 1;
  padding: 7px 8px;
  border: 0;
  border-radius: 7px;
  background: transparent;
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
}

.mode-button:hover {
  color: var(--text);
}

.mode-button.is-active {
  background: var(--primary-soft);
  color: var(--primary);
  box-shadow: var(--shadow-sm);
}

.mode-button:focus-visible {
  outline: none;
  box-shadow: var(--focus-ring);
}

.theme-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.theme-section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 0 2px;
}

.theme-count {
  color: var(--text-subtle);
  font-size: 10px;
}

.theme-list {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface);
}

.theme-option {
  display: grid;
  grid-template-columns: 70px minmax(0, 1fr) 22px;
  align-items: center;
  gap: 12px;
  width: 100%;
  padding: 10px 12px;
  border: 0;
  border-bottom: 1px solid var(--border);
  background: transparent;
  color: var(--text);
  text-align: left;
  cursor: pointer;
  scroll-margin-block: 18px;
  transition:
    background var(--transition),
    box-shadow var(--transition);
}

.theme-option:last-child {
  border-bottom: 0;
}

.theme-option:hover {
  background: var(--surface-2);
}

.theme-option.is-active {
  background: var(--primary-soft);
  box-shadow: inset 3px 0 0 var(--primary);
}

.theme-option:focus-visible {
  outline: none;
  box-shadow: inset 0 0 0 2px var(--primary);
}

.theme-option.is-active:focus-visible {
  box-shadow:
    inset 3px 0 0 var(--primary),
    inset 0 0 0 2px var(--primary);
}

.theme-preview {
  display: flex;
  width: 70px;
  height: 38px;
  flex-shrink: 0;
  gap: 3px;
  padding: 4px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--surface);
}

.theme-swatch {
  flex: 1;
  border: 1px solid rgba(15, 23, 42, 0.12);
  border-radius: 4px;
}

.theme-copy {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.theme-copy strong {
  color: var(--text);
  font-size: 12px;
  font-weight: 700;
}

.theme-copy small {
  overflow: hidden;
  color: var(--text-muted);
  font-size: 10px;
  line-height: 1.35;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.theme-check {
  display: grid;
  place-items: center;
  width: 22px;
  height: 22px;
  border: 1px solid var(--border);
  border-radius: 999px;
  background: var(--surface);
  color: transparent;
}

.theme-check.is-selected {
  border-color: var(--primary);
  background: var(--primary);
  color: var(--primary-contrast);
}

.setting-list {
  display: flex;
  flex-direction: column;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 18px;
  padding: 16px 0;
  border-bottom: 1px solid var(--border);
}

.setting-row:first-child {
  padding-top: 0;
}

.setting-row:last-child {
  border-bottom: 0;
}

.setting-copy {
  min-width: 0;
}

.setting-copy strong {
  display: block;
  color: var(--text);
  font-size: 13px;
  font-weight: 700;
}

.setting-copy p {
  margin: 3px 0 0;
  color: var(--text-muted);
  font-size: 11px;
  line-height: 1.45;
}

.switch {
  position: relative;
  display: inline-flex;
  align-items: center;
  width: 42px;
  height: 24px;
  flex-shrink: 0;
  padding: 2px;
  border: 1px solid var(--border);
  border-radius: 999px;
  background: var(--surface-2);
  cursor: pointer;
  transition:
    background var(--transition),
    border-color var(--transition);
}

.switch.is-on {
  border-color: var(--primary);
  background: var(--primary);
}

.switch:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.switch:focus-visible {
  outline: none;
  box-shadow: var(--focus-ring);
}

.switch-thumb {
  width: 18px;
  height: 18px;
  border-radius: 999px;
  background: var(--surface);
  box-shadow: var(--shadow-sm);
  transition: transform var(--transition);
}

.switch.is-on .switch-thumb {
  transform: translateX(18px);
}

.number-input {
  width: 92px;
  flex-shrink: 0;
  padding: 8px 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface);
  color: var(--text);
  font-size: 12px;
}

.number-input:focus {
  outline: none;
  border-color: var(--primary);
  box-shadow: var(--focus-ring);
}

.number-input:disabled {
  opacity: 0.5;
}

.input-with-unit {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
  color: var(--text-subtle);
  font-size: 10px;
}

.monitoring-note {
  margin: 0;
  padding: 11px 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  color: var(--text-muted);
  font-size: 11px;
  line-height: 1.5;
}

.system-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  flex-wrap: wrap;
  padding: 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface-2);
}

.system-meta > div {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.system-meta span {
  color: var(--text-subtle);
  font-size: 10px;
}

.system-meta strong {
  color: var(--text);
  font-size: 12px;
}

.settings-save-state {
  display: flex;
  align-items: center;
  gap: 6px;
  min-height: 18px;
  margin: 18px 0 0;
  color: var(--text-subtle);
  font-size: 11px;
}

.settings-save-state.is-saved {
  color: var(--success);
}

.settings-save-state.is-error {
  color: var(--danger);
}

@media (max-width: 820px) {
  .settings-shell {
    grid-template-columns: 200px minmax(0, 1fr);
  }

  .settings-content {
    padding: 20px;
  }
}

@media (max-width: 700px) {
  .settings-panel {
    height: min(760px, calc(100vh - 24px));
    max-height: calc(100vh - 24px);
    max-width: calc(100vw - 24px);
  }

  .settings-shell {
    grid-template-columns: 1fr;
    grid-template-rows: auto minmax(0, 1fr);
  }

  .settings-nav {
    flex-direction: row;
    padding: 10px;
    overflow-x: auto;
    overflow-y: hidden;
    border-right: 0;
    border-bottom: 1px solid var(--border);
  }

  .settings-nav-item {
    flex: 0 0 auto;
    width: auto;
    min-width: 142px;
  }

  .settings-nav-item.is-active {
    box-shadow: inset 0 -3px 0 var(--primary);
  }

  .settings-content {
    padding: 18px 16px;
  }

  .mode-row {
    align-items: stretch;
    flex-direction: column;
    gap: 10px;
  }

  .mode-switch {
    width: 100%;
  }

  .theme-option {
    grid-template-columns: 60px minmax(0, 1fr) 22px;
    gap: 10px;
    padding: 9px 10px;
  }

  .theme-preview {
    width: 60px;
    height: 34px;
  }
}

@media (max-width: 620px) {
  .settings-overlay {
    padding: 12px;
  }

  .settings-header {
    padding: 16px;
  }

  .settings-nav-copy small {
    display: none;
  }

  .setting-row {
    align-items: flex-start;
  }
}

@media (prefers-reduced-motion: reduce) {
  .settings-panel,
  .settings-section {
    animation: none;
  }

  .switch,
  .switch-thumb {
    transition: none;
  }
}

@keyframes settingsIn {
  from {
    opacity: 0;
    transform: translateY(8px) scale(0.985);
  }
  to {
    opacity: 1;
    transform: none;
  }
}

@keyframes sectionIn {
  from {
    opacity: 0;
    transform: translateY(4px);
  }
  to {
    opacity: 1;
    transform: none;
  }
}
</style>
