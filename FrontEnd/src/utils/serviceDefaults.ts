const PRESET_DEFAULT_PORTS: Record<string, number> = {
  vue: 5173,
  'spring-boot': 8080,
  python: 8000,
  dotnet: 5000,
  rust: 3000,
  go: 3000,
  custom: 3000,
}

const FALLBACK_PORT = 3000

interface ServiceUrlSource {
  url?: string | null
  port?: number | null
  preset?: string | null
  command?: string | null
  args?: string[] | null
}

export function defaultPortForPreset(preset?: string | null): number {
  return (preset && PRESET_DEFAULT_PORTS[preset]) || FALLBACK_PORT
}

export function normalizeUrl(url?: string | null): string {
  const trimmed = url?.trim() ?? ''
  if (!trimmed) return ''
  return /^[a-zA-Z][a-zA-Z\d+\-.]*:/.test(trimmed) ? trimmed : `http://${trimmed}`
}

function isPort(value: unknown): value is number {
  return typeof value === 'number' && Number.isInteger(value) && value > 0 && value <= 65535
}

export function portFromUrl(url?: string | null): number | undefined {
  const normalized = normalizeUrl(url)
  if (!normalized) return undefined

  try {
    const parsed = new URL(normalized)
    const port = Number(parsed.port)
    return isPort(port) ? port : undefined
  } catch {
    return undefined
  }
}

export function portFromCommand(command?: string | null, args?: string[] | null): number | undefined {
  const text = [command ?? '', ...(args ?? [])].join(' ')
  const patterns = [
    /--port(?:=|\s+)(\d{1,5})/i,
    /--server\.port(?:=|\s+)(\d{1,5})/i,
    /server\.port=(\d{1,5})/i,
    /runserver\s+(\d{1,5})/i,
  ]

  for (const pattern of patterns) {
    const match = text.match(pattern)
    if (!match) continue
    const port = Number(match[1])
    if (isPort(port)) return port
  }

  return undefined
}

export function resolvePort(source: ServiceUrlSource): number {
  const explicitPort = isPort(source.port) ? source.port : undefined
  return explicitPort
    ?? portFromUrl(source.url)
    ?? portFromCommand(source.command, source.args)
    ?? defaultPortForPreset(source.preset)
}

export function resolveServiceUrl(source: ServiceUrlSource): string {
  const url = normalizeUrl(source.url)
  return url || `http://localhost:${resolvePort(source)}`
}
