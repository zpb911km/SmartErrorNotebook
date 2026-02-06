import { invoke } from '@tauri-apps/api/core';
import { Subject } from '../types';

export async function getSubjects(): Promise<Subject[]> {
  return await invoke("get_subjects");
}

export async function createSubject(subject: Subject): Promise<Subject> {
  return await invoke('create_subject', { input: subject });
}

export async function updateSubject(subject: Subject): Promise<Subject> {
  return await invoke('update_subject', { subject });
}

export async function deleteSubject(subject: Subject): Promise<void> {
  return await invoke('delete_subject', { subject });
}