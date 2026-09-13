import { Dialog } from 'quasar'

export function confirmAction(message: string): Promise<boolean> {
  return new Promise((resolve) => {
    Dialog.create({
      title: '请确认',
      message,
      persistent: true,
      ok: { label: '确认', color: 'primary', unelevated: true },
      cancel: { label: '取消', flat: true, color: 'grey' }
    })
      .onOk(() => resolve(true))
      .onCancel(() => resolve(false))
      .onDismiss(() => resolve(false))
  })
}

export function showAlert(message: string) {
  Dialog.create({
    title: '提示',
    message,
    ok: { label: '知道了', color: 'primary' }
  })
}
