import { computed, ref } from 'vue'

export type Theme = 'light' | 'dark'
export type Palette = 'indigo' | 'blue' | 'cyan' | 'purple' | 'red' | 'orange' | 'gray'

export interface PaletteOption {
  id: Palette
  label: string
  description: string
  swatches: [string, string, string]
}

export const PALETTES: PaletteOption[] = [
  {
    id: 'indigo',
    label: 'Eclipse',
    description: 'Índigo profundo y nocturno',
    swatches: ['#4f46e5', '#818cf8', '#eef2ff'],
  },
  {
    id: 'blue',
    label: 'Marea',
    description: 'Azul frío, limpio y enfocado',
    swatches: ['#2563eb', '#60a5fa', '#eff6ff'],
  },
  {
    id: 'cyan',
    label: 'Cristal',
    description: 'Cian luminoso y preciso',
    swatches: ['#0e7490', '#22d3ee', '#ecfeff'],
  },
  {
    id: 'purple',
    label: 'Orquídea',
    description: 'Morado suave y creativo',
    swatches: ['#9333ea', '#c084fc', '#faf5ff'],
  },
  {
    id: 'red',
    label: 'Brasa',
    description: 'Rojo cálido y directo',
    swatches: ['#e11d48', '#fb7185', '#fff1f2'],
  },
  {
    id: 'orange',
    label: 'Cobre',
    description: 'Naranja terroso y acogedor',
    swatches: ['#c2410c', '#f59e0b', '#fff7ed'],
  },
  {
    id: 'gray',
    label: 'Ceniza',
    description: 'Gris neutro y discreto',
    swatches: ['#475569', '#94a3b8', '#f1f5f9'],
  },
]

const THEME_STORAGE_KEY = 'cardumen-deck-theme'
const PALETTE_STORAGE_KEY = 'cardumen-deck-palette'

const THEME_COLORS: Record<Palette, Record<Theme, string>> = {
  indigo: { light: '#f8fafc', dark: '#0b1120' },
  blue: { light: '#f3f8ff', dark: '#07111f' },
  cyan: { light: '#f2fdff', dark: '#06151a' },
  purple: { light: '#faf7ff', dark: '#120b1e' },
  red: { light: '#fff7f7', dark: '#160b0f' },
  orange: { light: '#fff8f1', dark: '#160d06' },
  gray: { light: '#f7f8fa', dark: '#0c0f14' },
}

const theme = ref<Theme>('light')
const palette = ref<Palette>('indigo')
let initialized = false

function isTheme(value: string | null): value is Theme {
  return value === 'light' || value === 'dark'
}

function isPalette(value: string | null): value is Palette {
  return PALETTES.some((option) => option.id === value)
}

function systemTheme(): Theme {
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

function syncThemeColor() {
  document
    .querySelector('meta[name="theme-color"]')
    ?.setAttribute('content', THEME_COLORS[palette.value][theme.value])
}

function applyTheme(next: Theme) {
  theme.value = next
  document.documentElement.dataset.theme = next
  document.documentElement.style.colorScheme = next
  syncThemeColor()
}

function applyPalette(next: Palette) {
  palette.value = next
  document.documentElement.dataset.palette = next
  syncThemeColor()
}

function initializeTheme() {
  if (initialized) return

  const storedTheme = localStorage.getItem(THEME_STORAGE_KEY)
  const currentTheme = document.documentElement.dataset.theme
  const initialTheme: Theme = isTheme(storedTheme)
    ? storedTheme
    : isTheme(currentTheme ?? null)
      ? (currentTheme as Theme)
      : systemTheme()

  const storedPalette = localStorage.getItem(PALETTE_STORAGE_KEY)
  const currentPalette = document.documentElement.dataset.palette
  const initialPalette: Palette = isPalette(storedPalette)
    ? storedPalette
    : isPalette(currentPalette ?? null)
      ? (currentPalette as Palette)
      : 'indigo'

  applyTheme(initialTheme)
  applyPalette(initialPalette)
  initialized = true
}

export function useTheme() {
  initializeTheme()

  const isDark = computed(() => theme.value === 'dark')

  function setTheme(next: Theme) {
    applyTheme(next)
    localStorage.setItem(THEME_STORAGE_KEY, next)
  }

  function setPalette(next: Palette) {
    applyPalette(next)
    localStorage.setItem(PALETTE_STORAGE_KEY, next)
  }

  function toggleTheme() {
    setTheme(theme.value === 'dark' ? 'light' : 'dark')
  }

  return { theme, palette, isDark, palettes: PALETTES, setTheme, setPalette, toggleTheme }
}
