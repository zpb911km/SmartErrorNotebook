import { Notify } from 'quasar'

export interface NotificationOptions {
  title: string
  message: string
  duration?: number
}
const dismissals = new Set<() => void>()
function notify(type: string, title: string, message: string, duration = 4000) {
  const dismiss = Notify.create({
    type,
    message: title,
    caption: message,
    timeout: duration,
    actions: [{ icon: 'close', 'aria-label': '关闭通知' }],
    onDismiss: () => dismissals.delete(dismiss)
  })
  dismissals.add(dismiss)
}
export const showInfo = (title: string, message: string, duration?: number) =>
  notify('info', title, message, duration)
export const showSuccess = (
  title: string,
  message: string,
  duration?: number
) => notify('positive', title, message, duration)
export const showWarning = (
  title: string,
  message: string,
  duration?: number
) => notify('warning', title, message, duration)
export const showError = (title: string, message: string, duration?: number) =>
  notify('negative', title, message, duration)
export const showDebug = (
  title: string,
  message: string | object,
  duration?: number
) => notify('info', title, JSON.stringify(message), duration)
export function destroyNotification() {
  dismissals.forEach((dismiss) => dismiss())
  dismissals.clear()
}
