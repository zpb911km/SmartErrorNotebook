import type { CreateAttachmentRequest } from '../types/attachment'
import { compressImageIfTooLarge } from './imageCompression'

export function fileToBase64(file: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => resolve(String(reader.result).split(',')[1])
    reader.onerror = () => reject(reader.error)
    reader.readAsDataURL(file)
  })
}

export async function blobUrlToBase64(url: string): Promise<string> {
  if (url.startsWith('data:')) return url.split(',')[1]
  const response = await fetch(url)
  if (!response.ok) throw new Error('读取图片失败')
  return fileToBase64(await response.blob())
}

export const buildDataUrl = (base64: string, mimeType: string) =>
  base64.startsWith('data:') ? base64 : `data:${mimeType};base64,${base64}`

export async function prepareAttachment(
  data: string
): Promise<CreateAttachmentRequest> {
  const compressed = await compressImageIfTooLarge(data)
  const base64Data = compressed.includes(',')
    ? compressed.split(',')[1]
    : compressed
  const mimeType = base64Data.startsWith('/9j/')
    ? 'image/jpeg'
    : base64Data.startsWith('iVBOR')
      ? 'image/png'
      : base64Data.startsWith('UklGR')
        ? 'image/webp'
        : base64Data.startsWith('R0lGOD')
          ? 'image/gif'
          : null
  if (!mimeType) throw new Error('不支持的图片格式')
  return { base64Data, mimeType }
}
