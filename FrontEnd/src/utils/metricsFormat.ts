export function formatCpuPercent(value: number): string {
  if (value < 0.1) return '0%'
  if (value < 10) return `${value.toFixed(1)}%`
  return `${Math.round(value)}%`
}

export function formatBytes(bytes: number): string {
  if (bytes <= 0) return '0 MB'
  const gigabyte = 1024 ** 3
  const megabyte = 1024 ** 2
  if (bytes >= gigabyte) {
    return `${(bytes / gigabyte).toFixed(1)} GB`
  }
  const megabytes = bytes / megabyte
  if (megabytes >= 100) return `${Math.round(megabytes)} MB`
  if (megabytes >= 10) return `${megabytes.toFixed(1)} MB`
  return `${Math.round(megabytes)} MB`
}
