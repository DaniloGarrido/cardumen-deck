import { isTauri } from '@tauri-apps/api/core'
import { readText, writeText } from '@tauri-apps/plugin-clipboard-manager'

export async function writeToClipboard(text: string): Promise<void> {
  if (isTauri()) {
    await writeText(text)
    return
  }

  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text)
    return
  }

  const textarea = document.createElement('textarea')
  textarea.value = text
  textarea.style.position = 'fixed'
  textarea.style.top = '-9999px'
  document.body.appendChild(textarea)
  textarea.focus()
  textarea.select()
  const success = document.execCommand('copy')
  document.body.removeChild(textarea)
  if (!success) throw new Error('No se pudo copiar al portapapeles')
}

export async function readFromClipboard(): Promise<string> {
  if (isTauri()) {
    return await readText()
  }

  if (navigator.clipboard?.readText) {
    return await navigator.clipboard.readText()
  }

  throw new Error('No se pudo leer el portapapeles')
}
