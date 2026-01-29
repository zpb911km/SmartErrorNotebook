import { invoke } from '@tauri-apps/api/core';
import { Source } from '../types';

export async function getSources(subjectId?: string): Promise<Source[]> {
  const filter = subjectId ? { subject_id: subjectId } : undefined;
  return await invoke('get_sources', { filter });
}

export async function getSource(id: string): Promise<Source> {
  return await invoke('get_source', { id });
}

export async function getBooks(subjectId?: string): Promise<string[]> {
  return await invoke('get_books', { subjectId });
}

export async function getChapters(subjectId: string | undefined, book: string): Promise<string[]> {
  return await invoke('get_chapters', { subjectId, book });
}

export async function getKnowledges(subjectId: string | undefined, book: string, chapter: string): Promise<string[]> {
  return await invoke('get_knowledges', { subjectId, book, chapter });
}

export async function createSource(source: Omit<Source, 'id' | 'question_id'>): Promise<Source> {
  const input = {
    subject_id: source.subject_id,
    book: source.book,
    chapter: source.chapter,
    knowledge: source.knowledge,
  };
  return await invoke('create_source', { input });
}

export async function updateSource(source: Source): Promise<Source> {
  const input = {
    id: source.id,
    subject_id: source.subject_id,
    book: source.book,
    chapter: source.chapter,
    knowledge: source.knowledge,
  };
  return await invoke('update_source', { input });
}

export async function deleteSource(source: Source): Promise<void> {
  return await invoke('delete_source', { id: source.id });
}

export async function getOrCreateSourceId(source: Omit<Source, 'id' | 'question_id'>): Promise<string> {
  const input = {
    book: source.book,
    chapter: source.chapter,
    knowledge: source.knowledge,
  };
  return await invoke('get_or_create_source_id', { input });
}