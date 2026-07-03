import { reactive } from 'vue'

/** 一道待导入题的原始数据 */
export interface PendingImportQuestion {
  prompt: string
  answer?: string
  analysis?: string
}

/** 待导入数据的结构 */
export interface PendingImportData {
  questions: PendingImportQuestion[]
  version?: string
}

/** 全局导入状态 */
export const importStore = reactive<{
  /** 待导入的预解析数据（有值则说明有文件关联导入待处理） */
  pendingData: PendingImportData | null
}>({
  pendingData: null
})

/** 清空待导入数据 */
export function clearPendingImport() {
  importStore.pendingData = null
}
