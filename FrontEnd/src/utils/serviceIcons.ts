import { Box, Monitor, Server } from '@lucide/vue'

export function categoryIconFor(category: string) {
  if (category === 'frontend') return Monitor
  if (category === 'backend') return Server
  return Box
}
