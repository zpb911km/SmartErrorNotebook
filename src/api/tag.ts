import { invoke } from '@tauri-apps/api/core'

import type {
  CreateTagRequest,
  CreateTagResponse,
  DeleteTagResponse,
  ListTagsResponse,
  UpdateTagRequest,
  UpdateTagResponse
} from '../types/tag'

export const createTag = async (request: CreateTagRequest) =>
  (await invoke<CreateTagResponse>('create_tag', { request })).tag

export const updateTag = async (request: UpdateTagRequest) =>
  (await invoke<UpdateTagResponse>('update_tag', { request })).tag

export const deleteTag = async (id: string) => {
  await invoke<DeleteTagResponse>('delete_tag', { request: { id } })
}

export const listTags = async () =>
  (await invoke<ListTagsResponse>('list_tags')).tags
