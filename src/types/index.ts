// src/types/index.ts

// 定义枚举类型
enum QuestionType {
  MultipleChoice = "multiple_choice",
  ShortAnswer = "short_answer",
  TrueFalse = "true_false",
  // 其他题型可以继续添加
}

enum AttachmentType {
  Original = "original",
  Answer = "answer",
}

enum FileType {
  Image = "img",
  // 其他文件类型可以继续添加
}

// 定义各个接口
interface ErrorQuestion {
  id: string; // uuid
  userid: string; // uuid
  subjectid: string; // uuid
  prompt: string; // 题干
  type: QuestionType; // 题型
  answer: string; // 标准答案
  analysis?: string; // 解析
  error_note?: string; // 错题笔记
  created_at: number; // 创建时间戳
  updated_at: number; // 修改时间戳
  sources?: Source[]; // 出题来源
  tags?: ErrorTags[]; // 错题标签
  attachments?: Attachment[]; // 附件
  srs_data?: SRSData; // 单词记忆数据
}

interface SRSData {
  id: string; // uuid
  question_id: string; // uuid
  difficulty: number; // 难度
  mastery: number; // 掌握程度，百分比
  lastreviewed_at: number; // 最后复习时间戳
  review_count: number; // 复习次数
}

interface Source {
  id: string; // uuid
  question_id: string; // uuid
  subject_id?: string; // uuid
  book?: string; // 书名
  chapter?: string; // 章节名
  knowledge?: string; // 知识点名
  subject?: Subject; // 科目
}

interface ErrorTags {
  id: string; // uuid
  question_id: string; // uuid
  name: string; // 错因名称
  color: string; // 标签颜色
}

interface Attachment {
  id: string; // uuid
  question_id: string; // uuid
  type: AttachmentType; // 附件意义
  file_type: FileType; // 文件类型
  path: string; // 文件相对路径
  hash: string; // 文件哈希
  created_at: number; // 创建时间戳
  updated_at: number; // 修改时间戳
}

interface Subject {
  id: string; // uuid，主键
  name: string; // 科目名称，同一用户下唯一
  color?: string; // 科目标识色
  created_at: number; // 创建时间戳
  updated_at: number; // 修改时间戳
}

interface UserConfig {
  username: string; // 用户名
  email: string; // 邮箱
  student_num: string; // 学号，长度为8
  phone?: string; // 手机号
  avatar?: string; // base64图像
  theme?: string; // 主题
  password_hash?: string; // 密码哈希
  ai_base_url?: string; // ai链接
  ai_key?: string; // ai密钥
  sync: boolean; // 是否同步
}

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
};
