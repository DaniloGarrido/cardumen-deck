<script setup lang="ts">
import { LayoutGrid, List } from '@lucide/vue'
import { useViewMode } from '../composables/useViewMode'

const { viewMode, setViewMode } = useViewMode()
</script>

<template>
  <div class="view-toggle" role="group" aria-label="Vista de servicios">
    <button
      class="view-toggle-btn"
      :class="{ active: viewMode === 'grid' }"
      type="button"
      aria-label="Vista de tarjetas"
      title="Vista de tarjetas"
      :aria-pressed="viewMode === 'grid'"
      @click="setViewMode('grid')"
    >
      <LayoutGrid :size="15" />
    </button>
    <button
      class="view-toggle-btn"
      :class="{ active: viewMode === 'list' }"
      type="button"
      aria-label="Vista de lista"
      title="Vista de lista"
      :aria-pressed="viewMode === 'list'"
      @click="setViewMode('list')"
    >
      <List :size="15" />
    </button>
  </div>
</template>

<style scoped>
.view-toggle {
  display: inline-flex;
  gap: 2px;
  padding: 2px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
}

.view-toggle-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  padding: 0;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--text-subtle);
  cursor: pointer;
  transition:
    background var(--transition),
    color var(--transition),
    box-shadow var(--transition);
}

.view-toggle-btn:hover {
  color: var(--text);
}

.view-toggle-btn.active {
  background: var(--surface);
  color: var(--primary);
  box-shadow: var(--shadow-sm);
}
</style>
