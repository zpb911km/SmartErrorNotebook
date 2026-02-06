import { invoke } from '@tauri-apps/api/core';
import { ErrorTags } from '../types';

export async function getErrorTags(): Promise<ErrorTags[]> {
  return await invoke('get_error_tags');
}

export async function createErrorTag(errorTag: Omit<ErrorTags, 'id'>): Promise<ErrorTags> {
  return await invoke('create_error_tag', { errorTag });
}

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

export async function updateErrorTag(errorTag: ErrorTags): Promise<ErrorTags> {
  return await invoke('update_error_tag', { errorTag });
}

export async function deleteErrorTag(errorTag: ErrorTags): Promise<void> {
  return await invoke('delete_error_tag', { errorTag });
}