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

/**
 * 删除指定错因标签的全部记录
 * 注意,这会一次性删除所有名字相同的记录, 因此请谨慎使用
 * @param tagName 错因标签名称
 * @returns 删除的标签数量
 */
export async function deleteErrorTagByName(tagName: string): Promise<number> {
  return await invoke('delete_error_tag_by_name', {
    tag_name: tagName
  });
}

/**
 * 更新指定错因标签
 * 注意,这会一次性更新所有名字相同的记录, 因此请谨慎使用
 * @param tagName 错因标签名称
 * @param name 新的标签名称
 * @param color 新的标签颜色
 * @returns 更新的标签数量
 */
export async function updateErrorTagByName(tagName: string, name: string, color: string): Promise<number> {
  return await invoke('update_error_tag_by_name', {
    tag_name: tagName,
    new_tag_name: name,
    new_tag_color: color
  });
}

export async function deleteErrorTagById(tagId: string): Promise<number> {
  return await invoke('delete_error_tag_by_id', {
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
