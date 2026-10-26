<script setup lang="ts">
import { computed, useId } from 'vue'
import { Cpu, MemoryStick } from '@lucide/vue'
import { USAGE_HISTORY_LIMIT } from '../composables/useServiceUsage'
import { formatBytes, formatCpuPercent } from '../utils/metricsFormat'
import type { UsageHistory } from '../types'

interface Props {
  cpuPercent?: number
  memoryBytes?: number
  processCount?: number
  history?: UsageHistory
  running?: boolean
}

interface ChartPoint {
  x: number
  y: number
}

const props = withDefaults(defineProps<Props>(), {
  cpuPercent: 0,
  memoryBytes: 0,
  processCount: 0,
  history: undefined,
  running: false,
})

const chartWidth = 120
const chartHeight = 38
const paddingX = 2
const paddingY = 3
const uid = useId()
const cpuGradientId = `cpu-${uid}`
const ramGradientId = `ram-${uid}`

const cpu = computed(() => Math.max(0, props.cpuPercent ?? 0))
const memory = computed(() => Math.max(0, props.memoryBytes ?? 0))
const cpuHistory = computed(() => props.history?.cpu ?? [])
const memoryHistory = computed(() => props.history?.memory ?? [])
const hasData = computed(() => cpuHistory.value.length > 0)
const hasSignal = computed(() => props.running && hasData.value)

const ramMax = computed(() => {
  const peak = memoryHistory.value.reduce((max, value) => Math.max(max, value), 0)
  return niceMemoryMax(Math.max(peak, 64 * 1024 * 1024))
})

const cpuPoints = computed(() => buildPoints(cpuHistory.value, 100))
const memoryPoints = computed(() => buildPoints(memoryHistory.value, ramMax.value))
const cpuLine = computed(() => smoothPath(cpuPoints.value))
const memoryLine = computed(() => smoothPath(memoryPoints.value))
const cpuArea = computed(() => areaPath(cpuPoints.value, cpuLine.value))
const memoryArea = computed(() => areaPath(memoryPoints.value, memoryLine.value))

const lastCpuPoint = computed(() => lastPoint(cpuPoints.value))
const lastMemoryPoint = computed(() => lastPoint(memoryPoints.value))

const peakCpu = computed(() =>
  Math.max(0, ...cpuHistory.value),
)
const peakMemory = computed(() =>
  Math.max(0, ...memoryHistory.value),
)

const cpuTone = computed(() => {
  if (cpu.value >= 85) return 'is-critical'
  if (cpu.value >= 60) return 'is-high'
  return ''
})

const cpuValue = computed(() => formatCpuPercent(cpu.value))
const memoryValue = computed(() => formatBytes(memory.value))
const processLabel = computed(() => {
  const count = props.processCount ?? 0
  return `${count} ${count === 1 ? 'proceso' : 'procesos'}`
})
const cpuTitle = computed(
  () => `CPU: ${cpuValue.value} · Pico: ${formatCpuPercent(peakCpu.value)} · ${processLabel.value}`,
)
const memoryTitle = computed(
  () => `RAM: ${memoryValue.value} · Pico: ${formatBytes(peakMemory.value)} · ${processLabel.value}`,
)

function buildPoints(values: number[], max: number): ChartPoint[] {
  if (values.length === 0) return []

  const usableWidth = chartWidth - paddingX * 2
  const offset = Math.max(0, USAGE_HISTORY_LIMIT - values.length)
  const denominator = Math.max(1, USAGE_HISTORY_LIMIT - 1)
  const usableHeight = chartHeight - paddingY * 2

  return values.map((value, index) => {
    const ratio = Math.min(1, Math.max(0, value / Math.max(1, max)))
    return {
      x: paddingX + ((offset + index) / denominator) * usableWidth,
      y: chartHeight - paddingY - ratio * usableHeight,
    }
  })
}

function smoothPath(points: ChartPoint[]): string {
  if (points.length === 0) return ''
  if (points.length === 1) {
    return `M ${paddingX} ${points[0].y} L ${chartWidth - paddingX} ${points[0].y}`
  }

  let path = `M ${points[0].x.toFixed(2)} ${points[0].y.toFixed(2)}`
  for (let index = 1; index < points.length; index += 1) {
    const previous = points[index - 1]
    const current = points[index]
    const controlX = ((previous.x + current.x) / 2).toFixed(2)
    path += ` C ${controlX} ${previous.y.toFixed(2)}, ${controlX} ${current.y.toFixed(2)}, ${current.x.toFixed(2)} ${current.y.toFixed(2)}`
  }
  return path
}

function areaPath(points: ChartPoint[], line: string): string {
  if (points.length === 0 || !line) return ''
  const first = points[0]
  const last = points[points.length - 1]
  return `${line} L ${last.x.toFixed(2)} ${chartHeight} L ${first.x.toFixed(2)} ${chartHeight} Z`
}

function lastPoint(points: ChartPoint[]): ChartPoint {
  if (points.length === 0) {
    return { x: chartWidth - paddingX, y: chartHeight - paddingY }
  }
  return points[points.length - 1]
}

function niceMemoryMax(bytes: number): number {
  const megabytes = Math.max(64, Math.ceil((bytes / (1024 * 1024)) * 1.15))
  const magnitude = 10 ** Math.max(0, Math.floor(Math.log10(megabytes)))
  const rounded = Math.ceil(megabytes / magnitude) * magnitude
  return rounded * 1024 * 1024
}

</script>

<template>
  <div class="vitals" :class="{ 'is-idle': !running }">
    <div class="vital cpu" :class="cpuTone" :title="cpuTitle">
      <div class="vital-head">
        <span class="vital-label"><Cpu :size="12" /> CPU</span>
        <strong class="vital-value">{{ cpuValue }}</strong>
      </div>
      <svg
        class="sparkline"
        :viewBox="`0 0 ${chartWidth} ${chartHeight}`"
        preserveAspectRatio="none"
        role="img"
        :aria-label="cpuTitle"
      >
        <defs>
          <linearGradient :id="cpuGradientId" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="currentColor" stop-opacity="0.3" />
            <stop offset="100%" stop-color="currentColor" stop-opacity="0" />
          </linearGradient>
        </defs>
        <line class="baseline" x1="0" :y1="chartHeight - paddingY" :x2="chartWidth" :y2="chartHeight - paddingY" />
        <path v-if="cpuArea" class="spark-area" :d="cpuArea" :fill="`url(#${cpuGradientId})`" />
        <path v-if="cpuLine" class="spark-line" :d="cpuLine" />
        <circle
          v-if="hasSignal"
          class="live-dot"
          :cx="lastCpuPoint.x"
          :cy="lastCpuPoint.y"
          r="2"
        />
      </svg>
    </div>

    <div class="vital ram" :title="memoryTitle">
      <div class="vital-head">
        <span class="vital-label"><MemoryStick :size="12" /> RAM</span>
        <strong class="vital-value">{{ memoryValue }}</strong>
      </div>
      <svg
        class="sparkline"
        :viewBox="`0 0 ${chartWidth} ${chartHeight}`"
        preserveAspectRatio="none"
        role="img"
        :aria-label="memoryTitle"
      >
        <defs>
          <linearGradient :id="ramGradientId" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="currentColor" stop-opacity="0.3" />
            <stop offset="100%" stop-color="currentColor" stop-opacity="0" />
          </linearGradient>
        </defs>
        <line class="baseline" x1="0" :y1="chartHeight - paddingY" :x2="chartWidth" :y2="chartHeight - paddingY" />
        <path v-if="memoryArea" class="spark-area" :d="memoryArea" :fill="`url(#${ramGradientId})`" />
        <path v-if="memoryLine" class="spark-line" :d="memoryLine" />
        <circle
          v-if="hasSignal"
          class="live-dot"
          :cx="lastMemoryPoint.x"
          :cy="lastMemoryPoint.y"
          r="2"
        />
      </svg>
    </div>
  </div>
</template>

<style scoped>
.vitals {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}

.vital {
  min-width: 0;
  padding: 8px 9px 5px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  overflow: hidden;
}

.cpu {
  color: var(--info);
}

.ram {
  color: var(--external);
}

.cpu.is-high {
  color: var(--warning);
}

.cpu.is-critical {
  color: var(--danger);
}

.vital-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  min-width: 0;
}

.vital-label {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  color: var(--text-subtle);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.vital-label svg {
  color: currentColor;
}

.vital-value {
  overflow: hidden;
  color: currentColor;
  font-family: var(--mono);
  font-size: 12px;
  font-weight: 700;
  line-height: 1.2;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sparkline {
  display: block;
  width: 100%;
  height: 38px;
  margin-top: 4px;
  overflow: visible;
}

.baseline {
  stroke: var(--border);
  stroke-dasharray: 2 3;
  stroke-width: 1;
  vector-effect: non-scaling-stroke;
}

.spark-area {
  stroke: none;
}

.spark-line {
  fill: none;
  stroke: currentColor;
  stroke-linecap: round;
  stroke-linejoin: round;
  stroke-width: 1.7;
  vector-effect: non-scaling-stroke;
}

.live-dot {
  fill: currentColor;
  animation: livePulse 1.8s ease-in-out infinite;
}

.vitals.is-idle .spark-line {
  stroke: var(--text-subtle);
  stroke-dasharray: 3 3;
  opacity: 0.7;
}

.vitals.is-idle .spark-area {
  opacity: 0.35;
}

.vitals.is-idle .vital-value {
  color: var(--text-muted);
}

@keyframes livePulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.35;
  }
}

@media (prefers-reduced-motion: reduce) {
  .live-dot {
    animation: none;
  }
}

@media (max-width: 340px) {
  .vitals {
    grid-template-columns: 1fr;
  }
}
</style>
