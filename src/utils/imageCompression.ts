/**
 * 将 base64 图片压缩到指定大小以下
 * @param base64String base64 格式的图片数据
 * @param maxSizeInBytes 最大文件大小（字节），默认 10MB
 * @returns 压缩后的 base64 字符串，如果无需压缩则返回原字符串
 */
export const compressImageIfTooLarge = async (
  base64String: string,
  maxSizeInBytes: number = 10 * 1024 * 1024
): Promise<string> => {
  // 提取 base64 数据部分（去除 data URI 前缀）
  let base64Data = base64String
  if (base64String.startsWith('data:image')) {
    base64Data = base64String.split(',')[1]
  }

  // 解码为二进制数据并计算大小
  const binaryData = atob(base64Data)
  const originalSize = binaryData.length

  // 如果原始大小已满足要求，直接返回
  if (originalSize <= maxSizeInBytes) {
    return base64String
  }

  // 加载图片
  const img = new Image()
  img.src = `data:image/png;base64,${base64Data}`
  await new Promise((resolve, reject) => {
    img.onload = resolve
    img.onerror = reject
  })

  // 逐步降低质量直到达到目标大小
  let quality = 0.9
  const step = 0.05

  const canvas = document.createElement('canvas')
  canvas.width = img.width
  canvas.height = img.height
  const ctx = canvas.getContext('2d')
  if (!ctx) throw new Error('无法创建 canvas 上下文')

  ctx.drawImage(img, 0, 0)

  let compressedBase64 = canvas.toDataURL('image/jpeg', quality)

  while (quality > 0.1) {
    const blob = await (await fetch(compressedBase64)).blob()
    const size = blob.size

    if (size <= maxSizeInBytes) {
      return compressedBase64.split(',')[1]
    }

    quality -= step
    compressedBase64 = canvas.toDataURL('image/jpeg', quality)
  }

  // 如果仍然太大，尝试调整尺寸
  let targetWidth = img.width
  let targetHeight = img.height

  while (targetWidth > 100 && targetHeight > 100) {
    targetWidth = Math.floor(targetWidth * 0.8)
    targetHeight = Math.floor(targetHeight * 0.8)

    canvas.width = targetWidth
    canvas.height = targetHeight

    ctx.clearRect(0, 0, targetWidth, targetHeight)
    ctx.drawImage(img, 0, 0, targetWidth, targetHeight)

    const scaledDataUrl = canvas.toDataURL('image/jpeg', 0.8)
    const blob = await (await fetch(scaledDataUrl)).blob()

    if (blob.size <= maxSizeInBytes) {
      return scaledDataUrl.split(',')[1]
    }
  }

  // 最后兜底：返回当前压缩结果
  return compressedBase64
}
