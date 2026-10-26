export const SERVICE_DRAG_MIME = 'application/x-cardumen-deck-service'

export function setServiceDragData(event: DragEvent, id: string) {
  if (!event.dataTransfer) return
  event.dataTransfer.setData(SERVICE_DRAG_MIME, id)
  event.dataTransfer.setData('text/plain', id)
  event.dataTransfer.effectAllowed = 'move'
}

export function isServiceDrag(event: DragEvent): boolean {
  return Array.from(event.dataTransfer?.types ?? []).includes(SERVICE_DRAG_MIME)
}
