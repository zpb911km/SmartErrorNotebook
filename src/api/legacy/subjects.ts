import { invoke } from '@tauri-apps/api/core'

import { Subject } from '../../types/legacy'

// ==================== API 接口 ====================

/**
 * 获取所有科目
 * @returns 科目列表（已过滤软删除的记录）
 */
export async function legacyGetSubjects(): Promise<Subject[]> {
  return await invoke('legacy_get_subjects')
}

/**
 * 创建科目
 * @param subject 科目数据（不包含 id）
 * @returns 创建的科目对象
 */
export async function legacyCreateSubject(
  subject: Omit<Subject, 'id'>
): Promise<Subject> {
  return await invoke('legacy_create_subject', { input: subject })
}

/**
 * 更新科目
 * @param subject 科目数据（必须包含 id）
 * @returns 更新后的科目对象
 */
export async function legacyUpdateSubject(subject: Subject): Promise<Subject> {
  return await invoke('legacy_update_subject', { input: subject })
}

/**
 * 删除科目（软删除）
 * @param id 科目ID
 * @returns 无返回值
 */
export async function legacyDeleteSubject(id: string): Promise<void> {
  return await invoke('legacy_delete_subject', { id })
}
