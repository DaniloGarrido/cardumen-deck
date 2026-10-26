import { ref } from 'vue'
import type { AppNotification } from '../types'

const notifications = ref<AppNotification[]>([])
let sequence = 0

function newId(): string {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) {
    return crypto.randomUUID()
  }
  sequence += 1
  return `${Date.now()}-${sequence}`
}

export function useNotifications() {
  function notify(input: Omit<AppNotification, 'id' | 'createdAt'>) {
    const now = Date.now()
    const duplicate = notifications.value.find(
      (item) =>
        item.scope === input.scope
        && item.message === input.message
        && now - item.createdAt < 2500,
    )
    if (duplicate) return duplicate

    const notification: AppNotification = {
      ...input,
      id: newId(),
      createdAt: now,
    }
    notifications.value = [...notifications.value, notification].slice(-3)
    return notification
  }

  function dismiss(id: string) {
    notifications.value = notifications.value.filter((item) => item.id !== id)
  }

  function dismissScope(scope: string) {
    notifications.value = notifications.value.filter((item) => item.scope !== scope)
  }

  function clear() {
    notifications.value = []
  }

  return {
    notifications,
    notify,
    dismiss,
    dismissScope,
    clear,
  }
}
