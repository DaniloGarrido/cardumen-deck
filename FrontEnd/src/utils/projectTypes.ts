import type { Component } from 'vue'
import { Box, Braces, Cog, Monitor, Server, Terminal, Wrench } from '@lucide/vue'
import type { ServiceCategory, ServicePreset } from '../types'

export interface ProjectTypeOption {
  id: ServicePreset
  label: string
  description: string
  category: ServiceCategory
  command: string
  setupCommand?: string
  port: number
  color: string
  icon: Component
}

export const PROJECT_TYPES: ProjectTypeOption[] = [
  {
    id: 'vue',
    label: 'Node / Vue',
    description: 'npm, pnpm, yarn o bun',
    category: 'frontend',
    command: 'npm run dev',
    setupCommand: 'npm install',
    port: 5173,
    color: '#3b82f6',
    icon: Monitor,
  },
  {
    id: 'spring-boot',
    label: 'Spring Boot',
    description: 'Maven o Gradle',
    category: 'backend',
    command: 'mvnw.cmd spring-boot:run',
    setupCommand: 'mvnw.cmd -q -DskipTests dependency:go-offline',
    port: 8080,
    color: '#22c55e',
    icon: Server,
  },
  {
    id: 'python',
    label: 'Python',
    description: 'Django, FastAPI o Flask',
    category: 'backend',
    command: 'python manage.py runserver',
    setupCommand: 'python -m venv .venv',
    port: 8000,
    color: '#f59e0b',
    icon: Terminal,
  },
  {
    id: 'dotnet',
    label: '.NET',
    description: 'ASP.NET / C#',
    category: 'backend',
    command: 'dotnet run',
    setupCommand: 'dotnet restore',
    port: 5000,
    color: '#8b5cf6',
    icon: Box,
  },
  {
    id: 'rust',
    label: 'Rust',
    description: 'Cargo',
    category: 'backend',
    command: 'cargo run',
    setupCommand: 'cargo fetch',
    port: 3000,
    color: '#f97316',
    icon: Cog,
  },
  {
    id: 'go',
    label: 'Go',
    description: 'Go modules',
    category: 'backend',
    command: 'go run .',
    setupCommand: 'go mod download',
    port: 3000,
    color: '#06b6d4',
    icon: Braces,
  },
  {
    id: 'custom',
    label: 'Otro',
    description: 'Configurar manualmente',
    category: 'custom',
    command: '',
    port: 3000,
    color: '#64748b',
    icon: Wrench,
  },
]

const STACK_PRESETS: Record<string, ServicePreset> = {
  node: 'vue',
  'spring-boot': 'spring-boot',
  python: 'python',
  dotnet: 'dotnet',
  rust: 'rust',
  go: 'go',
}

export function projectTypeForPreset(preset?: string | null): ProjectTypeOption | undefined {
  return PROJECT_TYPES.find((type) => type.id === preset)
}

export function projectTypeForStack(
  stack?: string | null,
  suggestedPreset?: string | null,
): ProjectTypeOption | undefined {
  if (!stack || stack === 'custom') return undefined

  const preset = STACK_PRESETS[stack] ?? suggestedPreset ?? undefined
  return projectTypeForPreset(preset)
}
