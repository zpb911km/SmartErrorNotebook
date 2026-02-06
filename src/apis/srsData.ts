import { invoke } from '@tauri-apps/api/core';
import { SRSData } from '../types';

export async function createSRSData(
  questionId: string,
  difficulty: number,
  mastery: number
): Promise<SRSData> {
  return await invoke('create_srs_data', {
    input: {
      question_id: questionId,
      difficulty,
      mastery,
    },
  });
}