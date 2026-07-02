import { save } from '@tauri-apps/plugin-dialog'
import { writeTextFile, writeFile } from '@tauri-apps/plugin-fs'

/**
 * 检测当前是否为 Tauri 移动端
 * 通过 UA 或 navigator.userAgent 判断
 */
function isMobile(): boolean {
  try {
    const ua = navigator.userAgent.toLowerCase()
    return /iphone|ipad|ipod|android/.test(ua)
  } catch {
    return false
  }
}

/**
 * 导出文件：移动端用 Web Share API，桌面端用 Tauri save dialog
 *
 * @param filename 文件名（含扩展名）
 * @param content 文本内容（JSON/HTML）或 Blob（PDF）
 * @param mimeType MIME 类型
 * @returns 是否成功
 */
export async function exportFile(
  filename: string,
  content: string | Blob,
  mimeType: string
): Promise<boolean> {
  // ===== 移动端：使用 Web Share API 分享到微信/QQ =====
  if (isMobile()) {
    try {
      if (typeof content === 'string') {
        // 文本内容（JSON/HTML）→ Blob → File
        const blob = new Blob([content], { type: mimeType })
        const file = new File([blob], filename, { type: mimeType })
        await navigator.share({
          title: filename,
          files: [file]
        })
      } else {
        // 已经是 Blob（PDF）
        const file = new File([content], filename, { type: mimeType })
        await navigator.share({
          title: filename,
          files: [file]
        })
      }
      return true
    } catch (e) {
      // 用户取消分享或 Web Share API 不支持 files
      console.warn('Web Share API 分享失败:', e)
      // fallback 到下载
    }
  }

  // ===== 桌面端 / fallback：Tauri save dialog + fs write =====
  try {
    const filePath = await save({
      defaultPath: filename,
      filters: [
        {
          name: mimeType.includes('json')
            ? 'JSON 文件'
            : mimeType.includes('html')
              ? 'HTML 文件'
              : mimeType.includes('pdf')
                ? 'PDF 文件'
                : '文件',
          extensions: [filename.split('.').pop() || '']
        }
      ]
    })
    if (!filePath) return false

    if (typeof content === 'string') {
      await writeTextFile(filePath, content)
    } else {
      const buf = await content.arrayBuffer()
      await writeFile(filePath, new Uint8Array(buf))
    }
    return true
  } catch (e) {
    console.error('保存文件失败:', e)
    return false
  }
}
