import { invoke } from '@tauri-apps/api/core'
import type {
  CreateSubjectRequest,
  CreateSubjectResponse,
  DeleteSubjectResponse,
  ListSubjectsResponse,
  UpdateSubjectRequest,
  UpdateSubjectResponse
} from '../types/subject'

export const createSubject = async (request: CreateSubjectRequest) =>
  (await invoke<CreateSubjectResponse>('create_subject', { request })).subject

export const updateSubject = async (request: UpdateSubjectRequest) =>
  (await invoke<UpdateSubjectResponse>('update_subject', { request })).subject

export const deleteSubject = async (id: string) => {
  await invoke<DeleteSubjectResponse>('delete_subject', { request: { id } })
}

export const listSubjects = async () =>
  (await invoke<ListSubjectsResponse>('list_subjects')).subjects
