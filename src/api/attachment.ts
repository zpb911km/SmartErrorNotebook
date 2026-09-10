import { invoke } from '@tauri-apps/api/core'
import type {
  CreateAttachmentRequest,
  CreateAttachmentResponse,
  DeleteAttachmentResponse,
  GetAttachmentResponse
} from '../types/attachment'

export const createAttachment = async (request: CreateAttachmentRequest) =>
  (await invoke<CreateAttachmentResponse>('create_attachment', { request })).id

export const getAttachment = async (id: string) =>
  (
    await invoke<GetAttachmentResponse>('get_attachment', {
      request: { id }
    })
  ).attachment

export const deleteAttachment = async (id: string) => {
  await invoke<DeleteAttachmentResponse>('delete_attachment', {
    request: { id }
  })
}
