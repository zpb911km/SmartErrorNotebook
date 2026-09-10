import { invoke } from '@tauri-apps/api/core'
import type {
  CreateSourceRequest,
  CreateSourceResponse,
  DeleteSourceResponse,
  DeleteSourcesResponse,
  GetSourceResponse,
  ListSourcesResponse,
  UpdateSourceRequest,
  UpdateSourceResponse
} from '../types/source'

export const createSource = async (request: CreateSourceRequest) =>
  (await invoke<CreateSourceResponse>('create_source', { request })).source

export const updateSource = async (request: UpdateSourceRequest) =>
  (await invoke<UpdateSourceResponse>('update_source', { request })).source

export const deleteSource = async (id: string) => {
  await invoke<DeleteSourceResponse>('delete_source', { request: { id } })
}

export const deleteSources = async (ids: string[]) => {
  await invoke<DeleteSourcesResponse>('delete_sources', { request: { ids } })
}

export const getSource = async (id: string) =>
  (await invoke<GetSourceResponse>('get_source', { request: { id } })).source

export const listSources = async (subjectId?: string) =>
  (
    await invoke<ListSourcesResponse>('list_sources', {
      request: { subjectId }
    })
  ).sources
