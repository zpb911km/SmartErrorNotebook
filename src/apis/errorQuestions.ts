import { invoke } from '@tauri-apps/api/core';
import { ErrorQuestion } from '../types';

export async function createErrorQuestion(
  request: Omit<ErrorQuestion, 'id'>
): Promise<ErrorQuestion> {
  return invoke('create_question', {
    input: request,
  });
}

export async function getQuestions(filter?: any): Promise<ErrorQuestion[]> {
  return invoke('get_questions', {
    filter,
  });
}