import { invoke } from '@tauri-apps/api/core';
import { ErrorTags } from '../types';

// ==================== API 接口 ====================

/**
 * 获取所有不重复的错因标签
 * @returns 错因标签列表
 */
export async function getErrorTags(): Promise<ErrorTags[]> {
  return await invoke('get_error_tags');
}

/**
 * 获取所有错因标签
 * @returns 错因标签列表
 */
export async function getFullErrorTags(): Promise<ErrorTags[]> {
  return await invoke('get_full_error_tags');
}

/**
 * 根据题目ID获取错因标签
 * @param questionId 错题ID
 * @returns 错因标签列表
 */
export async function getErrorTagByQuestionId(questionId: string): Promise<ErrorTags[]> {
  return await invoke("get_error_tags_for_question", {
    questionId: questionId
  });
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

export async function deleteErrorTagById(tagId: string): Promise<number> {
  return await invoke('delete_error_tag', {
    tagId: tagId
  });
}

export async function updateErrorTagById(tagId: string, name: string, color: string): Promise<number> {
  return await invoke('update_error_tag_by_id', {
    tag_id: tagId,
    new_tag_name: name,
    new_tag_color: color
  });
}
/*
 * 获取所有唯一的错因标签名称列表
 * @returns 不重复的标签名称数组
 */
export async function getAllUniqueTags(): Promise<string[]> {
  const allTags = await invoke('get_error_tags');
  const tagNames = (allTags as ErrorTags[]).map(tag => tag.name);
  // 去重
  return [...new Set(tagNames)];
}

// 注意：以下接口后端暂未实现
// - deleteErrorTag
