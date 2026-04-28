// SRS (间隔重复学习) API 接口

import { invoke } from '@tauri-apps/api/core';
import { SRSData } from '../types';

// ==================== Input/Output Types ====================

/**
 * 初始化 SRS 数据的输入参数
 */
export interface CreateSRSInput {
  /** 错题 ID */
  question_id: string;
  /** 初始难度（可选，默认 5.0）*/
  difficulty?: number;
}

/**
 * 提交复习结果的输入参数
 */
export interface SubmitReviewInput {
  /** 错题 ID */
  question_id: string;
  /** 用户反馈值 [0, 1] */
  /** 0: 清除所有 SRS 状态，当作新卡片从头来过 */
  /** 1: 直接视为熟记，以后不用再复习 */
  /** (0, 1): 连续反馈值 */
  feedback: number;
}

/**
 * SRS 卡片信息输出
 */
export interface SRSCardOutput extends Omit<SRSData, 'mastery' | 'lastreviewed_at'> {
  /** 稳定性（天）*/
  stability: number;
  /** 下次复习时间戳（秒）*/
  next_review_at: number | null;
  /** 上次复习时间戳（秒）*/
  last_review_at: number | null;
}

/**
 * 复习结果输出
 */
export interface ReviewOutput {
  /** 下次复习间隔（天）*/
  next_interval_days: number;
  /** 更新后的稳定性 */
  new_stability: number;
  /** 更新后的难度 */
  new_difficulty: number;
  /** 建议的下次复习时间戳（秒）*/
  next_review_at: number;
}

// ==================== API Functions ====================

/**
 * 为错题初始化 SRS 数据
 * 应在添加错题时调用，作为第一次复习
 * @param input 初始化参数
 * @returns 创建的 SRS 数据对象
 */
export async function initSRS(input: CreateSRSInput): Promise<SRSData> {
  return invoke('create_srs_data', { input });
}

/**
 * 获取待复习的题目列表
 * @param limit 返回数量限制（可选，默认 30）
 * @returns 待复习题目列表
 */
export async function getDueQuestions(limit?: number): Promise<SRSCardOutput[]> {
  return invoke('get_due_questions', { limit });
}

/**
 * 提交复习结果并更新 SRS 状态
 * @param input 复习结果参数
 * @returns 复习结果（包含新的 SRS 状态）
 */
export async function submitReviewResult(input: SubmitReviewInput): Promise<ReviewOutput> {
  return invoke('submit_review_result', { input });
}

/**
 * 获取单个题目的 SRS 状态
 * @param questionId 错题 ID
 * @returns SRS 数据对象，不存在则返回 undefined
 */
export async function getSRSStatus(questionId: string): Promise<SRSCardOutput | undefined> {
  return invoke('get_question_srs_status', { questionId });
}

/**
 * 重置单个题目的 SRS 进度（当作新卡片）
 * @param questionId 错题 ID
 * @returns 更新后的 SRS 数据对象
 */
export async function resetSRSProgress(questionId: string): Promise<SRSData> {
  return invoke('reset_srs_progress', { questionId });
}

/**
 * 获取所有题目的 SRS 状态
 * @returns 所有题目的 SRS 数据对象
 */
export async function getAllSRSStatus(): Promise<SRSData[]> {
  return invoke('get_all_cards');
}

export async function getDueCount(): Promise<number> {
  return invoke('get_due_count');
}

export async function getSRSStatics(): Promise<{
  total: number;
  due_count: number;
  new_cards: number;
  avg_stability: number;
  avg_difficulty: number;
  total_reviews: number;
}> {
  return invoke('get_srs_statics');
}

