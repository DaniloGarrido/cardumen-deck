import { readFromClipboard, writeToClipboard } from './clipboard'

export type TextContextAction = 'copy' | 'cut' | 'paste' | 'select-all'

export interface TextContext {
  target: HTMLElement | null
  editable: boolean
  canCopy: boolean
  canCut: boolean
  canPaste: boolean
  canSelectAll: boolean
  selectedText: string
  selectionStart: number | null
  selectionEnd: number | null
  range: Range | null
  input: HTMLInputElement | HTMLTextAreaElement | null
  contentEditable: HTMLElement | null
}

const TEXT_INPUT_TYPES = new Set([
  'text',
  'search',
  'url',
  'email',
  'password',
  'tel',
])

function isTextInput(input: HTMLInputElement): boolean {
  return TEXT_INPUT_TYPES.has(input.type)
}

function editableElement(target: HTMLElement | null): HTMLElement | null {
  if (!target) return null
  if (target instanceof HTMLInputElement) return isTextInput(target) ? target : null
  if (target instanceof HTMLTextAreaElement) return target

  const content = target.closest('[contenteditable]:not([contenteditable="false"])')
  return content instanceof HTMLElement && content.isContentEditable ? content : null
}

function emptyContext(target: HTMLElement | null, selectedText = ''): TextContext {
  return {
    target,
    editable: false,
    canCopy: selectedText.length > 0,
    canCut: false,
    canPaste: false,
    canSelectAll: false,
    selectedText,
    selectionStart: null,
    selectionEnd: null,
    range: null,
    input: null,
    contentEditable: null,
  }
}

export function captureTextContext(target: HTMLElement | null): TextContext {
  const editable = editableElement(target)

  if (editable instanceof HTMLInputElement || editable instanceof HTMLTextAreaElement) {
    const start = editable.selectionStart
    const end = editable.selectionEnd
    const hasSelection = start !== null && end !== null && end > start
    const selectedText = hasSelection ? editable.value.slice(start, end) : ''
    const locked = editable.readOnly || editable.disabled

    return {
      target: editable,
      editable: true,
      canCopy: hasSelection,
      canCut: hasSelection && !locked,
      canPaste: !locked,
      canSelectAll: !editable.disabled,
      selectedText,
      selectionStart: start,
      selectionEnd: end,
      range: null,
      input: editable,
      contentEditable: null,
    }
  }

  if (editable) {
    const selection = window.getSelection()
    let range: Range | null = null
    let selectedText = ''

    if (selection && selection.rangeCount > 0) {
      const candidate = selection.getRangeAt(0)
      if (editable.contains(candidate.commonAncestorContainer)) {
        range = candidate.cloneRange()
        selectedText = selection.toString()
      }
    }

    return {
      target: editable,
      editable: true,
      canCopy: selectedText.length > 0,
      canCut: selectedText.length > 0,
      canPaste: true,
      canSelectAll: true,
      selectedText,
      selectionStart: null,
      selectionEnd: null,
      range,
      input: null,
      contentEditable: editable,
    }
  }

  const selectedText = window.getSelection()?.toString() ?? ''
  return emptyContext(target, selectedText)
}

function replaceInputSelection(
  input: HTMLInputElement | HTMLTextAreaElement,
  start: number,
  end: number,
  text: string,
) {
  const max = input.value.length
  const safeStart = Math.min(Math.max(0, start), max)
  const safeEnd = Math.min(Math.max(safeStart, end), max)
  input.focus({ preventScroll: true })
  input.setRangeText(text, safeStart, safeEnd, 'end')
  input.dispatchEvent(new Event('input', { bubbles: true }))
}

function restoreContentEditableSelection(context: TextContext) {
  const editable = context.contentEditable
  if (!editable) return
  editable.focus({ preventScroll: true })
  const selection = window.getSelection()
  if (!selection) return

  selection.removeAllRanges()
  if (context.range) {
    selection.addRange(context.range)
    return
  }

  const range = document.createRange()
  range.selectNodeContents(editable)
  range.collapse(false)
  selection.addRange(range)
}

export async function copyTextContext(context: TextContext): Promise<void> {
  if (!context.canCopy) return
  await writeToClipboard(context.selectedText)

  if (context.input) {
    context.input.focus({ preventScroll: true })
    if (context.selectionStart !== null && context.selectionEnd !== null) {
      context.input.setSelectionRange(context.selectionStart, context.selectionEnd)
    }
    return
  }

  if (context.contentEditable) {
    restoreContentEditableSelection(context)
  }
}

export async function cutTextContext(context: TextContext): Promise<void> {
  if (!context.canCut) return
  await writeToClipboard(context.selectedText)

  if (context.input) {
    const start = context.selectionStart ?? context.input.selectionStart ?? 0
    const end = context.selectionEnd ?? context.input.selectionEnd ?? start
    replaceInputSelection(context.input, start, end, '')
    return
  }

  if (context.contentEditable) {
    restoreContentEditableSelection(context)
    document.execCommand('delete')
  }
}

export async function pasteTextContext(context: TextContext): Promise<void> {
  if (!context.canPaste) return
  const text = await readFromClipboard()
  if (!text) return

  if (context.input) {
    const input = context.input
    const start = context.selectionStart ?? input.selectionStart ?? input.value.length
    const end = context.selectionEnd ?? input.selectionEnd ?? start
    replaceInputSelection(input, start, end, text)
    return
  }

  if (context.contentEditable) {
    restoreContentEditableSelection(context)
    document.execCommand('insertText', false, text)
  }
}

export function selectAllTextContext(context: TextContext): void {
  if (!context.canSelectAll) return

  if (context.input) {
    context.input.focus({ preventScroll: true })
    context.input.select()
    return
  }

  const editable = context.contentEditable
  if (!editable) return

  editable.focus({ preventScroll: true })
  const range = document.createRange()
  range.selectNodeContents(editable)
  const selection = window.getSelection()
  selection?.removeAllRanges()
  selection?.addRange(range)
}
