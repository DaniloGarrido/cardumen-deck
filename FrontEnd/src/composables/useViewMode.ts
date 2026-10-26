import { ref, watch } from 'vue'

export type ViewMode = 'grid' | 'list'

const STORAGE_KEY = 'cardumen-deck-view-mode'
const stored = localStorage.getItem(STORAGE_KEY)
const viewMode = ref<ViewMode>(stored === 'list' ? 'list' : 'grid')

watch(viewMode, (value) => {
  localStorage.setItem(STORAGE_KEY, value)
})

export function useViewMode() {
  function setViewMode(value: ViewMode) {
    viewMode.value = value
  }

  return { viewMode, setViewMode }
}
