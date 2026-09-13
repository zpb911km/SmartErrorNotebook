import { beforeEach, expect, it, vi } from 'vitest'

import { buildDataUrl, prepareAttachment } from '../src/utils/attachments'
import { compressImageIfTooLarge } from '../src/utils/imageCompression'

vi.mock('../src/utils/imageCompression', () => ({
  compressImageIfTooLarge: vi.fn(async (value: string) => value)
}))
beforeEach(() => {
  vi.clearAllMocks()
})

it('uses the actual compressed image signature for its MIME type', async () => {
  vi.mocked(compressImageIfTooLarge).mockResolvedValueOnce('/9j/compressed')
  expect(
    await prepareAttachment('data:image/png;base64,iVBORoriginal')
  ).toEqual({ mimeType: 'image/jpeg', base64Data: '/9j/compressed' })
})

it('strips data URL metadata from uploads and avoids duplicate display prefixes', async () => {
  expect(await prepareAttachment('data:image/png;base64,iVBORdata')).toEqual({
    mimeType: 'image/png',
    base64Data: 'iVBORdata'
  })
  expect(buildDataUrl('iVBORdata', 'image/png')).toBe(
    'data:image/png;base64,iVBORdata'
  )
  expect(buildDataUrl('data:image/png;base64,iVBORdata', 'image/png')).toBe(
    'data:image/png;base64,iVBORdata'
  )
})

it('rejects unsupported image data before upload', async () => {
  await expect(prepareAttachment('unknown')).rejects.toThrow('不支持的图片格式')
})
