import { invoke } from '@tauri-apps/api/core';
import {
  ErrorQuestion,
  QuestionFilter,
  UpdateQuestionInput,
  QuestionStats,
} from '../types';

// ==================== API 接口 ====================

/**
 * 创建错题
 * @param request 错题数据（不包含 id）
 * @returns 创建的错题对象
 */
export async function createErrorQuestion(
  request: Omit<ErrorQuestion, 'id'>
): Promise<ErrorQuestion> {
  return invoke('create_question', {
    input: request,
  });
}

/**
 * 获取错题列表
 * @param filter 筛选条件（可选）
 * @returns 错题列表，按更新时间降序排列
 */
export async function getQuestions(
  filter?: QuestionFilter
): Promise<ErrorQuestion[]> {
  return invoke('get_questions', { filter });
}

/**
 * 获取单个错题
 * @param id 错题ID
 * @returns 错题对象
 * @throws 如果错题不存在则抛出错误
 */
export async function getQuestion(id: string): Promise<ErrorQuestion> {
  return invoke('get_question', { id });
}

/**
 * 更新错题
 * @param input 更新参数（包含 id 和要更新的字段）
 * @returns 更新后的错题对象
 */
export async function updateQuestion(
  input: UpdateQuestionInput
): Promise<ErrorQuestion> {
  return invoke('update_question', { input });
}

/**
 * 删除错题（软删除）
 * @param id 错题ID
 * @returns 无返回值
 */
export async function deleteQuestion(id: string): Promise<void> {
  return invoke('delete_question', { id });
}

/**
 * 获取错题统计信息
 * @returns 错题统计信息（包含总数等）
 */
export async function getQuestionStats(): Promise<QuestionStats> {
  return invoke('get_question_stats');
}