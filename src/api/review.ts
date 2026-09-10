import { invoke } from '@tauri-apps/api/core'
import type {
  GetLibraryStatisticsResponse,
  GetSrsDataResponse,
  ListSrsDataResponse,
  ResetReviewProgressResponse,
  SubmitReviewRequest,
  SubmitReviewResponse
} from '../types/review'

export const submitReview = (request: SubmitReviewRequest) =>
  invoke<SubmitReviewResponse>('submit_review', { request })

export const resetReviewProgress = async (
  questionId: string,
  resetAt: string
) =>
  (
    await invoke<ResetReviewProgressResponse>('reset_review_progress', {
      request: { questionId, resetAt }
    })
  ).srs

export const getSrsData = async (questionId: string, at: string) =>
  (
    await invoke<GetSrsDataResponse>('get_srs_data', {
      request: { questionId, at }
    })
  ).srs

export const listSrsData = async (at: string) =>
  (await invoke<ListSrsDataResponse>('list_srs_data', { request: { at } }))
    .items

export const getLibraryStatistics = async (at: string) =>
  (
    await invoke<GetLibraryStatisticsResponse>('get_library_statistics', {
      request: { at }
    })
  ).statistics
