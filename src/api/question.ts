import { invoke } from '@tauri-apps/api/core'

import type {
  CreateQuestionRequest,
  CreateQuestionResponse,
  DeleteQuestionResponse,
  GetQuestionResponse,
  ListQuestionsRequest,
  ListQuestionsResponse,
  UpdateQuestionRequest,
  UpdateQuestionResponse
} from '../types/question'

export const createQuestion = async (request: CreateQuestionRequest) =>
  (await invoke<CreateQuestionResponse>('create_question', { request }))
    .question

export const updateQuestion = async (request: UpdateQuestionRequest) =>
  (await invoke<UpdateQuestionResponse>('update_question', { request }))
    .question

export const deleteQuestion = async (id: string) => {
  await invoke<DeleteQuestionResponse>('delete_question', { request: { id } })
}

export const getQuestion = async (id: string) =>
  (
    await invoke<GetQuestionResponse>('get_question', {
      request: { id }
    })
  ).question

export const listQuestions = (request: ListQuestionsRequest) =>
  invoke<ListQuestionsResponse>('list_questions', { request })
