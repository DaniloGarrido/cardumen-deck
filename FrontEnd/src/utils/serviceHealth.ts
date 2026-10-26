import type { HealthState, ServiceHealth } from '../types'

export function healthLabel(state?: HealthState): string {
  switch (state) {
    case 'healthy':
      return 'Saludable'
    case 'degraded':
      return 'Inestable'
    case 'unhealthy':
      return 'Sin respuesta'
    case 'checking':
      return 'Comprobando'
    case 'stopped':
      return 'Detenido'
    default:
      return 'Sin datos'
  }
}

export function healthKindLabel(kind?: ServiceHealth['kind']): string {
  if (kind === 'http') return 'HTTP'
  if (kind === 'tcp') return 'TCP'
  return 'Proceso'
}

export function healthDetail(health?: ServiceHealth): string {
  if (!health) return 'Sin datos de salud'

  const parts = [healthLabel(health.state)]
  if (health.target) parts.push(`${healthKindLabel(health.kind)} · ${health.target}`)
  else if (health.kind === 'process') parts.push('Proceso')
  if (health.latencyMs != null) parts.push(`${health.latencyMs} ms`)
  if (health.consecutiveFailures > 0) {
    parts.push(`${health.consecutiveFailures} fallo${health.consecutiveFailures === 1 ? '' : 's'}`)
  }
  if (health.message) parts.push(health.message)
  return parts.join(' · ')
}

export function formatHealthTime(timestamp?: number): string {
  if (!timestamp) return 'Sin comprobar'
  const date = new Date(timestamp * 1000)
  if (Number.isNaN(date.getTime())) return 'Sin comprobar'
  return new Intl.DateTimeFormat('es', {
    dateStyle: 'short',
    timeStyle: 'medium',
  }).format(date)
}
