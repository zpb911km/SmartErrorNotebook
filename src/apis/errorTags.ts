import { invoke } from '@tauri-apps/api/core';
import { ErrorTags } from '../types';

// ==================== API 接口 ====================

/**
 * 获取所有错因标签
 * @returns 错因标签列表
 */
export async function getErrorTags(): Promise<ErrorTags[]> {
  return await invoke('get_error_tags');
}

/**
 * 为指定题目批量创建错因标签
 * @param questionId 错题ID
 * @param tags 标签数组，每个标签包含名称和颜色
 * @returns 创建的错因标签列表
 */
export async function createErrorTagsForQuestion(
  questionId: string,
  tags: Array<{ name: string; color: string }>
): Promise<ErrorTags[]> {
  return await invoke('create_error_tags_for_question', {
    input: {
      question_id: questionId,
      tags: tags
    }
  });
}

// 注意：以下接口后端暂未实现
// - deleteErrorTag