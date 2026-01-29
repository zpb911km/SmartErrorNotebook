import { invoke } from '@tauri-apps/api/core';
import { Source } from '../types';

export async function getSources(): Promise<Source[]> {
  return await invoke('get_sources');
}

export async function createSource(source: Omit<Source, 'id' | 'question_id'>): Promise<Source> {
  return await invoke('create_source', { source });
}

export async function updateSource(source: Source): Promise<Source> {
  return await invoke('update_source', { source });
}

export async function deleteSource(source: Source): Promise<void> {
  return await invoke('delete_source', { source });
}