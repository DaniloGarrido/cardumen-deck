<script setup lang="ts">
import { computed } from 'vue'
import { Cpu, MemoryStick } from '@lucide/vue'
import { formatBytes, formatCpuPercent } from '../utils/metricsFormat'

interface Props {
  cpuPercent?: number
  memoryBytes?: number
  processCount?: number
  variant?: 'sidebar' | 'header'
  label?: string
}

const props = withDefaults(defineProps<Props>(), {
  cpuPercent: 0,
  memoryBytes: 0,
  processCount: 0,
  variant: 'sidebar',
  label: '',
})

const cpu = computed(() => Math.max(0, props.cpuPercent ?? 0))
const memory = computed(() => Math.max(0, props.memoryBytes ?? 0))
const processCount = computed(() => Math.max(0, props.processCount ?? 0))

const cpuTone = computed(() => {
  if (cpu.value >= 85) return 'is-critical'
  if (cpu.value >= 60) return 'is-high'
  return ''
})

const processLabel = computed(() => {
  const count = processCount.value
  return `${count} ${count === 1 ? 'proceso' : 'procesos'}`
})

const title = computed(() => {
  const prefix = props.label ? `${props.label} · ` : ''
  return `${prefix}CPU total: ${formatCpuPercent(cpu.value)} · RAM total: ${formatBytes(memory.value)} · ${processLabel.value}`
})
</script>

<template>
  <div
    class="workspace-usage"
    :class="[`variant-${variant}`, cpuTone, { 'is-idle': processCount === 0 }]"
    role="group"
    :title="title"
    :aria-label="title"
  >
    <span class="usage cpu">
      <Cpu :size="11" aria-hidden="true" />
      <span class="usage-value">{{ formatCpuPercent(cpu) }}</span>
    </span>
    <span class="usage ram">
      <MemoryStick :size="11" aria-hidden="true" />
      <span class="usage-value">{{ formatBytes(memory) }}</span>
    </span>
  </div>
</template>

<style scoped>
.workspace-usage {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  flex-wrap: wrap;
  color: var(--text-subtle);
  font-size: 10px;
  font-weight: 600;
  line-height: 1.2;
}

.usage {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  min-width: 0;
  white-space: nowrap;
}

.usage.cpu {
  color: var(--info);
}

.usage.ram {
  color: var(--external);
}

.usage-value {
  overflow: hidden;
  font-family: var(--mono);
  font-variant-numeric: tabular-nums;
  text-overflow: ellipsis;
}

.workspace-usage.is-high .usage.cpu {
  color: var(--warning);
}

.workspace-usage.is-critical .usage.cpu {
  color: var(--danger);
}

.workspace-usage.is-idle .usage {
  color: var(--text-subtle);
}

.variant-sidebar {
  margin-top: 2px;
  gap: 6px;
}

.variant-header {
  padding: 5px 9px;
  border: 1px solid var(--border);
  border-radius: 999px;
  background: var(--surface-2);
  font-size: 11px;
}
</style>
