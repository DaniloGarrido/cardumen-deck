<script setup lang="ts">
import ServiceCard from './ServiceCard.vue'
import ServiceListItem from './ServiceListItem.vue'
import type { Service, ServiceRuntime, ServiceUsage, UsageHistory } from '../types'

interface Props {
  services: Service[]
  viewMode: 'grid' | 'list'
  getStatus: (id: string) => string
  getRuntime: (id: string) => ServiceRuntime | undefined
  getGitBranch: (cwd: string) => string | undefined
  getStatusClass: (status: string) => string
  categoryLabel: (cat: string) => string
  getUsage: (id: string) => ServiceUsage | undefined
  getUsageHistory: (id: string) => UsageHistory | undefined
  draggingId?: string | null
}

interface Emits {
  (e: 'start', id: string): void
  (e: 'stop', id: string): void
  (e: 'restart', id: string): void
  (e: 'logs', id: string): void
  (e: 'info', svc: Service): void
  (e: 'move', svc: Service): void
  (e: 'edit', svc: Service): void
  (e: 'delete', id: string): void
  (e: 'contextmenu', svc: Service, event: MouseEvent): void
  (e: 'git', svc: Service): void
  (e: 'drag-start', svc: Service): void
  (e: 'drag-end'): void
  (e: 'drop-service', targetId: string, after: boolean): void
}

defineProps<Props>()
const emit = defineEmits<Emits>()

function forwardContextMenu(service: Service, event: MouseEvent) {
  emit('contextmenu', service, event)
}

function forwardDrop(targetId: string, after: boolean) {
  emit('drop-service', targetId, after)
}
</script>

<template>
  <div v-if="viewMode === 'grid'" class="grid">
    <ServiceCard
      v-for="svc in services"
      :key="svc.id"
      :svc="svc"
      :status="getStatus(svc.id)"
      :pid-alive="getRuntime(svc.id)?.pidAlive ?? false"
      :git-branch="getGitBranch(svc.cwd)"
      :health="getRuntime(svc.id)?.health"
      :status-class="getStatusClass(getStatus(svc.id))"
      :category-label="categoryLabel(svc.category)"
      :usage="getUsage(svc.id)"
      :usage-history="getUsageHistory(svc.id)"
      :dragging="svc.id === draggingId"
      @start="$emit('start', svc.id)"
      @stop="$emit('stop', svc.id)"
      @restart="$emit('restart', svc.id)"
      @logs="$emit('logs', svc.id)"
      @info="$emit('info', svc)"
      @edit="$emit('edit', svc)"
      @delete="$emit('delete', svc.id)"
      @contextmenu="forwardContextMenu"
      @git="$emit('git', svc)"
      @drag-start="$emit('drag-start', svc)"
      @drag-end="$emit('drag-end')"
      @drop-service="forwardDrop"
    />
  </div>

  <div v-else class="list">
    <ServiceListItem
      v-for="svc in services"
      :key="svc.id"
      :svc="svc"
      :status="getStatus(svc.id)"
      :pid-alive="getRuntime(svc.id)?.pidAlive ?? false"
      :git-branch="getGitBranch(svc.cwd)"
      :health="getRuntime(svc.id)?.health"
      :status-class="getStatusClass(getStatus(svc.id))"
      :category-label="categoryLabel(svc.category)"
      :dragging="svc.id === draggingId"
      @start="$emit('start', svc.id)"
      @stop="$emit('stop', svc.id)"
      @restart="$emit('restart', svc.id)"
      @logs="$emit('logs', svc.id)"
      @info="$emit('info', svc)"
      @move="$emit('move', svc)"
      @edit="$emit('edit', svc)"
      @delete="$emit('delete', svc.id)"
      @contextmenu="forwardContextMenu"
      @git="$emit('git', svc)"
      @drag-start="$emit('drag-start', svc)"
      @drag-end="$emit('drag-end')"
      @drop-service="forwardDrop"
    />
  </div>
</template>

<style scoped>
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(min(320px, 100%), 1fr));
  gap: 18px;
  align-items: stretch;
}

.list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

@media (max-width: 640px) {
  .grid {
    grid-template-columns: 1fr;
  }
}
</style>
