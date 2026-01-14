// src/types/mock.ts

import { v4 as uuidv4 } from 'uuid';
import { QuestionType, AttachmentType, FileType, ErrorQuestion, SRSData, Source, ErrorTags, Attachment, Subject, UserConfig } from './index';

export const mockErrorQuestion1: ErrorQuestion = {
  id: uuidv4(),
  userid: uuidv4(),
  subjectid: uuidv4(),
  prompt: "什么是JavaScript?",
  type: QuestionType.ShortAnswer,
  answer: "JavaScript是一种用于网页开发的脚本语言。",
  analysis: "JavaScript是一种轻量级的编程语言，广泛用于客户端网页开发。",
  error_note: "回答中没有提到JavaScript的用途。",
  created_at: Date.now(),
  updated_at: Date.now(),
};

export const mockErrorQuestion2: ErrorQuestion = {
  id: uuidv4(),
  userid: mockErrorQuestion1.userid,
  subjectid: mockErrorQuestion1.subjectid,
  prompt: "解释一下闭包。",
  type: QuestionType.ShortAnswer,
  answer: "闭包是函数和对其周围状态（词法环境）的引用组合。",
  analysis: "闭包允许函数访问其词法作用域中的变量，即使函数在其词法作用域之外执行。",
  error_note: "回答中没有提到闭包的实际用途。",
  created_at: Date.now(),
  updated_at: Date.now(),
};

export const mockSRSData1: SRSData = {
  id: uuidv4(),
  question_id: mockErrorQuestion1.id,
  difficulty: 2,
  mastery: 75,
  lastreviewed_at: Date.now(),
  review_count: 5,
};

export const mockSRSData2: SRSData = {
  id: uuidv4(),
  question_id: mockErrorQuestion2.id,
  difficulty: 3,
  mastery: 50,
  lastreviewed_at: Date.now(),
  review_count: 3,
};

export const mockSource1: Source = {
  id: uuidv4(),
  question_id: mockErrorQuestion1.id,
  subject_id: mockErrorQuestion1.subjectid,
  book: "JavaScript权威指南",
  chapter: "第2章",
  knowledge: "语言基础",
};

export const mockSource2: Source = {
  id: uuidv4(),
  question_id: mockErrorQuestion2.id,
  subject_id: mockErrorQuestion2.subjectid,
  book: "深入浅出JavaScript",
  chapter: "第5章",
  knowledge: "闭包",
};

export const mockErrorTags1: ErrorTags = {
  id: uuidv4(),
  question_id: mockErrorQuestion1.id,
  name: "概念理解不深刻",
  color: "#FF0000",
};

export const mockErrorTags2: ErrorTags = {
  id: uuidv4(),
  question_id: mockErrorQuestion2.id,
  name: "实际应用不清楚",
  color: "#FFA500",
};

export const mockAttachment1: Attachment = {
  id: uuidv4(),
  question_id: mockErrorQuestion1.id,
  type: AttachmentType.Original,
  file_type: FileType.Image,
  path: "attachments/mockErrorQuestion1.jpg",
  hash: "abcdef1234567890",
  created_at: Date.now(),
  updated_at: Date.now(),
};

export const mockAttachment2: Attachment = {
  id: uuidv4(),
  question_id: mockErrorQuestion2.id,
  type: AttachmentType.Answer,
  file_type: FileType.Image,
  path: "attachments/mockErrorQuestion2.jpg",
  hash: "0987654321abcdef",
  created_at: Date.now(),
  updated_at: Date.now(),
};

export const mockSubject1: Subject = {
  id: uuidv4(),
  name: "计算机科学",
  color: "#0000FF",
  created_at: Date.now(),
  updated_at: Date.now(),
};

export const mockSubject2: Subject = {
  id: uuidv4(),
  name: "数学",
  color: "#00FF00",
  created_at: Date.now(),
  updated_at: Date.now(),
};

export const mockUserConfig: UserConfig = {
  username: "johndoe",
  email: "johndoe@example.com",
  student_num: "12345678",
  phone: "13800138000",
  avatar: "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAUA...",
  theme: "dark",
  password_hash: "hashed_password",
  ai_base_url: "https://api.example.com",
  ai_key: "ai_key_here",
  sync: true,
};


