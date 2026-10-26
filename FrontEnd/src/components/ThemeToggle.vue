<script setup lang="ts">
import { Moon, Sun } from '@lucide/vue'
import { useTheme } from '../composables/useTheme'

const { isDark, toggleTheme } = useTheme()
</script>

<template>
  <button
    class="icon-btn theme-toggle"
    type="button"
    :aria-label="isDark ? 'Cambiar a modo claro' : 'Cambiar a modo oscuro'"
    :aria-pressed="isDark"
    :title="isDark ? 'Modo claro' : 'Modo oscuro'"
    @click="toggleTheme"
  >
    <Sun v-if="isDark" :size="17" />
    <Moon v-else :size="17" />
  </button>
</template>

<style scoped>
.theme-toggle {
  transition: transform var(--transition), background var(--transition), color var(--transition);
}

.theme-toggle:hover {
  transform: rotate(-8deg);
}
</style>
