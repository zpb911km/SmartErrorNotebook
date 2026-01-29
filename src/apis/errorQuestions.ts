import { invoke } from '@tauri-apps/api/core';
import { ErrorQuestion, Source, ErrorTags, SRSData } from '../types';

export interface CreateErrorQuestionRequest {
  errorQuestion: Omit<ErrorQuestion, 'id'>;
  source?: Omit<Source, 'id' | 'question_id'>;
  errorTag?: Omit<ErrorTags, 'id' | 'question_id'>;
  srsData?: Omit<SRSData, 'id' | 'question_id'>;
  attachment?: string; // 图片的base64数据
}

export interface CreateErrorQuestionResponse {
  questionId: string;
}

export async function createErrorQuestion(request: CreateErrorQuestionRequest): Promise<CreateErrorQuestionResponse> {
  // 将camelCase转换为snake_case以匹配后端API
  const snakeCaseRequest = {
    error_question: {
      userid: request.errorQuestion.userid,
      subjectid: request.errorQuestion.subjectid,
      prompt: request.errorQuestion.prompt,
      type: request.errorQuestion.type,
      answer: request.errorQuestion.answer,
      analysis: request.errorQuestion.analysis,
      error_note: request.errorQuestion.error_note,
    },
    source: request.source ? {
      subject_id: request.source.subject_id,
      book: request.source.book,
      chapter: request.source.chapter,
      knowledge: request.source.knowledge,
    } : undefined,
    error_tag: request.errorTag ? {
      name: request.errorTag.name,
      color: request.errorTag.color,
    } : undefined,
    srs_data: request.srsData ? {
      difficulty: request.srsData.difficulty,
      mastery: request.srsData.mastery,
      lastreviewed_at: request.srsData.lastreviewed_at,
      review_count: request.srsData.review_count,
    } : undefined,
    attachment: request.attachment,
  };

  const response = await invoke('create_error_question', { request: snakeCaseRequest });
  return response as CreateErrorQuestionResponse;
}