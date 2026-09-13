import { invoke } from '@tauri-apps/api/core'

import { SRSData } from '../../types/legacy'

// ==================== API 接口 ====================

/**
 * 为指定题目创建 SRS（间隔重复学习）数据
 * @param questionId 错题ID
 * @param difficulty 难度值
 * @param mastery 掌握程度（百分比）
 * @returns 创建的 SRS 数据对象
 */
export async function legacyCreateSRSData(
  questionId: string,
  difficulty?: number
): Promise<SRSData> {
  return await invoke('legacy_create_srs_data', {
    input: {
      question_id: questionId,
      difficulty
    }
  })
}

/**
 * 获取单个题目的 SRS 状态
 * @param questionId 错题ID
 * @returns SRS 数据对象，如果不存在则返回 null
 */
export async function legacyGetQuestionSRSStatus(
  questionId: string
): Promise<SRSData | null> {
  return await invoke('legacy_get_question_srs_status', { questionId })
}
