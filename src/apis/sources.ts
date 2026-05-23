import { invoke } from '@tauri-apps/api/core'
import {
  Source,
  CreateSourceInput,
  UpdateSourceInput,
  SourceFilter
} from '../types'

// ==================== API 接口 ====================

/**
 * 获取所有来源
 * @param subjectId 科目ID（可选）
 * @returns 来源列表（已过滤软删除的记录）
 */
export async function getSources(subjectId?: string): Promise<Source[]> {
  const filter: SourceFilter = subjectId ? { subject_id: subjectId } : {}
  return await invoke('get_sources', { filter })
}

/**
 * 获取单个来源
 * @param id 来源ID
 * @returns 来源对象
 * @throws 如果来源不存在则抛出错误
 */
export async function getSource(id: string): Promise<Source> {
  return await invoke('get_source', { id })
}

/**
 * 获取指定科目下的所有书名
 * @param subjectId 科目ID（可选）
 * @returns 书名列表
 */
export async function getBooks(subjectId?: string): Promise<string[]> {
  return await invoke('get_books', { subjectId })
}

/**
 * 获取指定科目和书名下的所有章节
 * @param book 书名
 * @param subjectId 科目ID（可选）
 * @returns 章节列表
 */
export async function getChapters(
  book: string,
  subjectId?: string
): Promise<string[]> {
  return await invoke('get_chapters', { book, subjectId })
}

/**
 * 获取指定科目、书名和章节下的所有知识点
 * @param book 书名
 * @param chapter 章节名
 * @param subjectId 科目ID（可选）
 * @returns 知识点列表
 */
export async function getKnowledges(
  book: string,
  chapter: string,
  subjectId?: string
): Promise<string[]> {
  return await invoke('get_knowledges', { book, chapter, subjectId })
}

/**
 * 创建来源
 * @param source 来源数据（不包含 id 和 question_id）
 * @returns 创建的来源对象
 */
export async function createSource(source: CreateSourceInput): Promise<Source> {
  return await invoke('create_source', { input: source })
}

/**
 * 更新来源
 * @param source 来源数据（必须包含 id）
 * @returns 更新后的来源对象
 */
export async function updateSource(source: UpdateSourceInput): Promise<Source> {
  return await invoke('update_source', { input: source })
}

/**
 * 删除来源（软删除）
 * @param sourceId 来源ID
 * @returns 无返回值
 */
export async function deleteSource(sourceId: string): Promise<void> {
  return await invoke('delete_source', { id: sourceId })
}

/**
 * 获取或新增来源，并返回来源ID
 * 如果指定的来源已存在，则返回其ID；否则创建新来源并返回新ID
 * @param source 来源数据（不包含 id 和 question_id）
 * @returns 来源ID
 */
export async function getOrCreateSourceId(
  source: CreateSourceInput
): Promise<string> {
  return await invoke('get_or_create_source_id', { input: source })
}
