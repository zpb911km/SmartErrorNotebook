// src/types/index.ts

// ==================== 枚举类型 ====================

/**
 * 题型枚举
 */
enum QuestionType {
  MultipleChoice = "多选题",
  ShortAnswer = "简答题",
  TrueFalse = "判断题",
  SigleChoice = "单选题",
  FillInTheBlank = "填空题",
  Essay = "论述题",
  Calculation = "计算题",
}

/**
 * 附件类型枚举
 */
enum AttachmentType {
  Original = "original", // 原始题目图片
  Answer = "answer",     // 答案图片
}

/**
 * 文件类型枚举
 */
enum FileType {
  Image = "img", // 图片
  // 其他文件类型可以继续添加
}

// ==================== 错题相关类型 ====================

/**
 * 错题筛选条件
 */
interface QuestionFilter {
  /** 科目 ID（可选） */
  subject_id?: string;
  /** 搜索关键词（可选，会在题干、解析、错题笔记中搜索） */
  search?: string;
  /** 返回数量限制（可选） */
  limit?: number;
  /** 偏移量（可选，用于分页） */
  offset?: number;
}

/**
 * 更新错题的输入参数
 */
interface UpdateQuestionInput {
  /** 错题 ID */
  id: string;
  /** 科目 ID（可选） */
  subject_id?: string;
  /** 来源 ID（可选） */
  source_id?: string;
  /** 题干（可选） */
  prompt?: string;
  /** 题型（可选） */
  type?: string;
  /** 标准答案（可选） */
  answer?: string;
  /** 解析（可选） */
  analysis?: string;
  /** 错题笔记（可选） */
  error_note?: string;
}

/**
 * 错题统计信息
 */
interface QuestionStats {
  /** 错题总数 */
  total: number;
}

// ==================== 来源相关类型 ====================

/**
 * 创建来源的输入参数
 */
interface CreateSourceInput {
  /** 科目 ID（可选） */
  subject_id?: string;
  /** 书名（可选） */
  book?: string;
  /** 章节名（可选） */
  chapter?: string;
  /** 知识点名（可选） */
  knowledge?: string;
}

/**
 * 更新来源的输入参数
 */
interface UpdateSourceInput {
  /** 来源 ID */
  id: string;
  /** 科目 ID（可选） */
  subject_id?: string;
  /** 书名（可选） */
  book?: string;
  /** 章节名（可选） */
  chapter?: string;
  /** 知识点名（可选） */
  knowledge?: string;
}

/**
 * 来源筛选条件
 */
interface SourceFilter {
  /** 科目 ID（可选） */
  subject_id?: string;
}

// ==================== 实体接口 ====================

/**
 * 错题实体
 *
 * FIXME: 字段名与后端 Model 不匹配
 *   user_id → userid
 *   subject_id → subjectid
 *   source_id → sourceid
 *   type → type_
 * 后端 error_question::Model 返回的是 userid/subjectid/sourceid/type_
 */
interface ErrorQuestion {
  /** 错题 ID（uuid）*/
  id: string;
  /** 用户 ID（uuid）*/
  user_id: string;
  /** 科目 ID（uuid）*/
  subject_id: string;
  /** 来源 ID（uuid，可选）*/
  source_id?: string;
  /** 题干 */
  prompt: string;
  /** 题型 */
  type: QuestionType;
  /** 标准答案（可选） */
  answer?: string;
  /** 解析（可选） */
  analysis?: string;
  /** 错题笔记（可选） */
  error_note?: string;
}

/**
 * SRS（间隔重复学习）数据实体 - 基于连续反馈的 SDR 模型
 *
 * FIXME: last_review_at 字段名不匹配后端
 *   SRSCardOutput 返回 last_review_at
 *   srs_data::Model 返回 lastreviewed_at（create_srs_data 和 reset_srs_progress）
 *   视图用 srs.last_review_at ?? srs.lastreviewed_at 兼容
 */
interface SRSData {
  /** SRS 数据 ID（uuid）*/
  id: string;
  /** 错题 ID（uuid）*/
  question_id: string;
  /** 稳定性（Stability）：单位天，表示记忆强度 */
  stability: number;
  /** 难度（Difficulty）：[1.0, 10.0] */
  difficulty: number;
  /** 预测召回率 */
  recall_rate: number;
  /** 下次复习时间戳（秒，可选）*/
  next_review_at: number | null;
  /** 上次复习时间戳（秒，可选）*/
  last_review_at: number | null;
  /** 复习次数 */
  review_count: number;
  /** 最近最多 5 次反馈记录（JSON 数组字符串）*/
  feedback_history: string;
}

/**
 * 来源实体
 */
interface Source {
  /** 来源 ID（uuid）*/
  id: string;
  /** 错题 ID（uuid，可选） */
  question_id?: string;
  /** 科目 ID（uuid，可选） */
  subject_id?: string;
  /** 书名 */
  book?: string;
  /** 章节名 */
  chapter?: string;
  /** 知识点名 */
  knowledge?: string;
  /** 科目信息（可选） */
  subject?: Subject;
}

/**
 * 错因标签实体
 */
interface ErrorTags {
  /** 标签 ID（uuid）*/
  id: string;
  /** 错题 ID（uuid）*/
  question_id: string;
  /** 错因名称 */
  name: string;
  /** 标签颜色 */
  color: string;
}

/**
 * 附件实体
 *
 * FIXME: type → type_，后端 AttachmentInterface 的字段是 type_ 没有 rename
 */
interface Attachment {
  /** 附件 ID（uuid）*/
  id: string;
  /** 错题 ID（uuid）*/
  question_id: string;
  /** 附件类型（original/answer） */
  type: string;
  /** 文件类型（img 等）*/
  file_type: string;
  /** base64 编码的文件数据 */
  base64_data: string;
  /** 文件哈希 */
  hash: string;
}

/**
 * 科目实体
 */
interface Subject {
  /** 科目 ID（uuid，主键）*/
  id: string;
  /** 科目名称（同一用户下唯一） */
  name: string;
  /** 科目标识色（可选） */
  color?: string;
}

/**
 * 用户配置实体
 */
interface UserConfig {
  /** 用户名 */
  username: string;
  /** 邮箱 */
  email: string;
  /** 学号（长度为 8）*/
  student_num: string;
  /** 手机号（可选） */
  phone?: string;
  /** 头像（base64 图像，可选） */
  avatar?: string;
  /** 主题（可选） */
  theme?: string;
  /** 密码哈希（可选） */
  password_hash?: string;
  /** AI 接口地址（可选） */
  ai_base_url?: string;
  /** AI 密钥（可选） */
  ai_key?: string;
  /** 是否同步 */
  sync: boolean;
}

// ==================== 附件相关类型 ====================

/**
 * 创建附件的输入参数
 */
interface CreateAttachmentInput {
  /** 错题 ID */
  question_id: string;
  /** 附件类型 */
  type_: string;
  /** 文件类型 */
  file_type: string;
  /** base64 数据 */
  base64_data: string;
}

// ==================== 导出 ====================

export {
  QuestionType,
  AttachmentType,
  FileType,
};

export type {
  ErrorQuestion,
  SRSData,
  Source,
  ErrorTags,
  Attachment,
  Subject,
  UserConfig,
  QuestionFilter,
  UpdateQuestionInput,
  QuestionStats,
  CreateSourceInput,
  UpdateSourceInput,
  SourceFilter,
  CreateAttachmentInput,
};
