import { invoke } from '@tauri-apps/api/core';
import { Subject } from '../types';

export async function getSubjects(): Promise<Subject[]> {
  // return await invoke("get_subjects");
  return [
    {
      id: '1',
      name: 'English',
      color: '#ff0000',
    },
    {
      id: '2',
      name: 'Math',
      color: '#00ff00',
    },
    {
      id: '3',
      name: 'Science',
      color: '#0000ff',
    },
  ];
}

export async function createSubject(subject: Subject): Promise<Subject> {
  return await invoke('create_subject', { subject });
}

export async function updateSubject(subject: Subject): Promise<Subject> {
  return await invoke('update_subject', { subject });
}

export async function deleteSubject(subject: Subject): Promise<void> {
  return await invoke('delete_subject', { subject });
}
