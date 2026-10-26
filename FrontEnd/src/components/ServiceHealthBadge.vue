<script setup lang="ts">
import { computed } from 'vue'
import { HeartPulse } from '@lucide/vue'
import { healthDetail, healthLabel } from '../utils/serviceHealth'
import type { ServiceHealth } from '../types'

interface Props {
  health?: ServiceHealth
  compact?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  compact: true,
})

const state = computed(() => props.health?.state ?? 'unknown')
const label = computed(() => healthLabel(state.value))
const detail = computed(() => healthDetail(props.health))
const visible = computed(
  () => Boolean(props.health) && state.value !== 'unknown' && state.value !== 'stopped',
)
</script>

<template>
  <span
    v-if="visible"
    class="health-badge"
    :class="[`is-${state}`, { 'is-compact': compact }]"
    :title="detail"
    :aria-label="`Salud: ${detail}`"
  >
    <HeartPulse :size="12" />
    <span v-if="!compact">{{ label }}</span>
  </span>
</template>

<style scoped>
.health-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  min-height: 22px;
  padding: 3px 7px;
  border-radius: 999px;
  font-size: 10px;
  font-weight: 700;
  line-height: 1;
  white-space: nowrap;
}

.health-badge.is-compact {
  width: 22px;
  padding: 0;
}

.health-badge.is-healthy {
  background: var(--success-soft);
  color: var(--success);
}

.health-badge.is-degraded {
  background: var(--warning-soft);
  color: var(--warning);
}

.health-badge.is-unhealthy {
  background: var(--danger-soft);
  color: var(--danger);
}

.health-badge.is-checking {
  background: var(--info-soft);
  color: var(--info);
}

.health-badge.is-unknown,
.health-badge.is-stopped {
  background: var(--surface-2);
  color: var(--text-subtle);
}
</style>
