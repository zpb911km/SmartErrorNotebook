import { invoke } from '@tauri-apps/api/core';
import { Attachment } from '../types';

export interface CreateAttachmentInput {
  question_id: string;
  type_: string;
  file_type: string;
  base64_data: string;
}

export async function createAttachment(
  input: CreateAttachmentInput
): Promise<Attachment> {
  return await invoke('create_attachment', { input });
}

export async function createAttachmentsForQuestion(
  questionId: string,
  attachments: Omit<CreateAttachmentInput, 'question_id'>[]
): Promise<Attachment[]> {
  return await invoke('create_attachments_for_question', {
    questionId,
    attachments,
  });
}

export async function getAttachmentsByQuestion(
  questionId: string
): Promise<Attachment[]> {
  return await invoke('get_attachments_by_question', { questionId });
}

export async function deleteAttachment(id: string): Promise<void> {
  return await invoke('delete_attachment', { id });
}

// 工具函数：将File对象转换为base64
export async function fileToBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.readAsDataURL(file);
    reader.onload = () => {
      const result = reader.result as string;
      // 移除data:image/xxx;base64,前缀，只保留base64数据
      const base64Data = result.split(',')[1];
      resolve(base64Data);
    };
    reader.onerror = (error) => reject(error);
  });
}

// 工具函数：将blob URL转换为base64
export async function blobUrlToBase64(blobUrl: string): Promise<string> {
  const response = await fetch(blobUrl);
  const blob = await response.blob();
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.readAsDataURL(blob);
    reader.onload = () => {
      const result = reader.result as string;
      const base64Data = result.split(',')[1];
      resolve(base64Data);
    };
    reader.onerror = (error) => reject(error);
  });
}

// 工具函数：将base64转换为blob URL
export function base64ToBlobUrl(base64: string, mimeType: string = 'image/png'): string {
  const byteCharacters = atob(base64);
  const byteArrays = [];
  
  for (let offset = 0; offset < byteCharacters.length; offset += 512) {
    const slice = byteCharacters.slice(offset, offset + 512);
    const byteNumbers = new Array(slice.length);
    
    for (let i = 0; i < slice.length; i++) {
      byteNumbers[i] = slice.charCodeAt(i);
    }
    
    const byteArray = new Uint8Array(byteNumbers);
    byteArrays.push(byteArray);
  }
  
  const blob = new Blob(byteArrays, { type: mimeType });
  return URL.createObjectURL(blob);
}

// 工具函数：从data URL提取base64数据
export function extractBase64FromDataUrl(dataUrl: string): string {
  return dataUrl.split(',')[1];
}

// 工具函数：构建data URL
export function buildDataUrl(base64: string, mimeType: string): string {
  return `data:${mimeType};base64,${base64}`;
}