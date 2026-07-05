import { invoke } from '@tauri-apps/api/core'
import { Attachment, CreateAttachmentInput } from '../types'
import { compressImageIfTooLarge } from '../utils/imageCompression'

// ==================== API 接口 ====================

/**
 * 创建附件
 * @param input 附件数据（包含题目ID、类型、文件类型、base64数据）
 * @returns 创建的附件对象
 */
export async function createAttachment(
  input: CreateAttachmentInput
): Promise<Attachment> {
  // 阻塞式图片压缩（等待压缩完成）
  let compressedBase64Data = input.base64_data
  const base64 = input.base64_data.startsWith('data:')
    ? input.base64_data.split(',')[1]
    : input.base64_data

  if (base64) {
    const compressed = await compressImageIfTooLarge(input.base64_data)
    if (compressed !== input.base64_data) {
      console.log(
        `[图片压缩] 题目 ${input.question_id} 的图片从 ${Math.round(input.base64_data.length / 1024)}KB 压缩至 ${Math.round(compressed.length / 1024)}KB`
      )
      compressedBase64Data = compressed
    }
  }

  // 使用压缩后的数据创建附件
  const updatedInput: CreateAttachmentInput = {
    ...input,
    base64_data: compressedBase64Data
  }

  return await invoke('create_attachment', { input: updatedInput })
}

/**
 * 批量为指定题目创建附件
 * @param questionId 错题ID
 * @param attachments 附件数据数组
 * @returns 创建的附件列表
 */
export async function createAttachmentsForQuestion(
  questionId: string,
  attachments: CreateAttachmentInput[]
): Promise<Attachment[]> {
  console.log('========== createAttachmentsForQuestion ==========')
  console.log('questionId:', questionId)
  console.log('附件数量:', attachments.length)

  // 阻塞式批量图片压缩（等待所有压缩完成）
  const compressedAttachments = await Promise.all(
    attachments.map(async (att, index) => {
      console.log(`\n--- 处理附件 ${index + 1} ---`)
      console.log('原始 base64_data 长度:', att.base64_data?.length || 0)

      const base64 = att.base64_data.startsWith('data:')
        ? att.base64_data.split(',')[1]
        : att.base64_data

      if (base64) {
        const compressed = await compressImageIfTooLarge(att.base64_data)
        if (compressed !== att.base64_data) {
          console.log(
            `[图片压缩] 题目 ${questionId} 的附件从 ${Math.round(att.base64_data.length / 1024)}KB 压缩至 ${Math.round(compressed.length / 1024)}KB`
          )
          return { ...att, base64_data: compressed }
        } else {
          console.log('图片未压缩（大小合适）')
        }
      }
      console.log(
        '最终 base64_data 长度:',
        (att.base64_data || base64)?.length || 0
      )
      return att
    })
  )

  console.log('\n========== 准备发送给后端的附件数据 ==========')
  console.log('questionId:', questionId)
  console.log(
    'attachments:',
    compressedAttachments.map((att, i) => ({
      index: i + 1,
      question_id: att.question_id,
      type_: att.type_,
      file_type: att.file_type,
      base64_data_length: att.base64_data?.length || 0
    }))
  )

  return await invoke('create_attachments_for_question', {
    questionId: questionId,
    attachments: compressedAttachments
  })
}

/**
 * 获取指定题目的所有附件
 * @param questionId 错题ID
 * @returns 附件列表（已过滤软删除的记录）
 */
export async function getAttachmentsByQuestion(
  questionId: string
): Promise<Attachment[]> {
  return await invoke('get_attachments_by_question', { questionId })
}

/**
 * 删除附件（软删除）
 * @param id 附件ID
 * @returns 无返回值
 */
export async function deleteAttachment(id: string): Promise<void> {
  return await invoke('delete_attachment', { id })
}

// ==================== 工具函数 ====================

/**
 * 将 base64 字符串转换为 Uint8Array
 * @param base64 base64 编码的字符串
 * @returns Uint8Array 字节数组
 */
export function base64ToArrayBuffer(base64: string): Uint8Array {
  const binaryString = atob(base64)
  const bytes = new Uint8Array(binaryString.length)
  for (let i = 0; i < binaryString.length; i++) {
    bytes[i] = binaryString.charCodeAt(i)
  }
  return bytes
}

/**
 * 将 File 对象转换为 base64 字符串
 * @param file 文件对象
 * @returns base64 编码的字符串（不包含 data URL 前缀）
 */
export async function fileToBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.readAsDataURL(file)
    reader.onload = () => {
      const result = reader.result as string
      // 移除 data:image/xxx;base64, 前缀，只保留 base64 数据
      const base64Data = result.split(',')[1]
      resolve(base64Data)
    }
    reader.onerror = (error) => reject(error)
  })
}

/**
 * 将 blob URL 或 data URL 转换为 base64 字符串
 * @param url blob URL 或 data URL
 * @returns base64 编码的字符串（不包含 data URL 前缀）
 */
export async function blobUrlToBase64(url: string): Promise<string> {
  // 如果已经是 data URL，直接提取 base64 部分，避免 fetch 的额外开销和潜在兼容性问题
  if (url.startsWith('data:')) {
    return url.split(',')[1]
  }
  const response = await fetch(url)
  const blob = await response.blob()
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.readAsDataURL(blob)
    reader.onload = () => {
      const result = reader.result as string
      const base64Data = result.split(',')[1]
      resolve(base64Data)
    }
    reader.onerror = (error) => reject(error)
  })
}

/**
 * 将 base64 字符串转换为 blob URL
 * @param base64 base64 编码的字符串
 * @param mimeType MIME 类型（默认为 image/png）
 * @returns blob URL
 */
export function base64ToBlobUrl(
  base64: string,
  mimeType: string = 'image/png'
): string {
  const byteCharacters = atob(base64)
  const byteArrays = []

  for (let offset = 0; offset < byteCharacters.length; offset += 512) {
    const slice = byteCharacters.slice(offset, offset + 512)
    const byteNumbers = new Array(slice.length)

    for (let i = 0; i < slice.length; i++) {
      byteNumbers[i] = slice.charCodeAt(i)
    }

    const byteArray = new Uint8Array(byteNumbers)
    byteArrays.push(byteArray)
  }

  const blob = new Blob(byteArrays, { type: mimeType })
  return URL.createObjectURL(blob)
}

/**
 * 从 data URL 提取 base64 数据
 * @param dataUrl data URL（格式为 data:mimeType;base64,base64Data）
 * @returns base64 数据（不包含前缀）
 */
export function extractBase64FromDataUrl(dataUrl: string): string {
  return dataUrl.split(',')[1]
}

/**
 * 构建 data URL
 * @param base64 base64 数据
 * @param mimeType MIME 类型
 * @returns data URL
 */
export function buildDataUrl(base64: string, mimeType: string): string {
  return `data:${mimeType};base64,${base64}`
}
