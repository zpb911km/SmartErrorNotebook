// src/types/testdata.ts
// 测试用的完整假数据集合

import { UserConfig, Subject, Source, ErrorQuestion, SRSData, ErrorTags, Attachment, QuestionType } from './index'

// 用户配置数据
export const testUsers: UserConfig[] = [
  {
    username: '张三',
    email: 'zhangsan@example.com',
    student_num: '20240001',
    phone: '13800138001',
    avatar: 'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTAwIiBoZWlnaHQ9IjEwMCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48Y2lyY2xlIGN4PSI1MCIgY3k9IjUwIiByPSI1MCIgZmlsbD0iIzQyODVGNCIvPjx0ZXh0IHg9IjUwIiB5PSI1NSIgZm9udC1zaXplPSIzMCIgZmlsbD0id2hpdGUiIHRleHQtYW5jaG9yPSJtaWRkbGUiPkE8L3RleHQ+PC9zdmc+',
    theme: 'light',
    password_hash: '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/RK.PZvO.S',
    ai_base_url: 'https://api.openai.com/v1',
    ai_key: 'sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
    sync: true
  },
  {
    username: '李四',
    email: 'lisi@example.com',
    student_num: '20240002',
    phone: '13800138002',
    avatar: 'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTAwIiBoZWlnaHQ9IjEwMCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48Y2lyY2xlIGN4PSI1MCIgY3k9IjUwIiByPSI1MCIgZmlsbD0iI0ZGNThCMiIvPjx0ZXh0IHg9IjUwIiB5PSI1NSIgZm9udC1zaXplPSIzMCIgZmlsbD0id2hpdGUiIHRleHQtYW5jaG9yPSJtaWRkbGUiPkI8L3RleHQ+PC9zdmc+',
    theme: 'dark',
    password_hash: '$2b$12$abcd1234EFGH5678IJKL90mnopqrSTUVWXYZabcdefghijk.lmn',
    ai_base_url: 'https://api.deepseek.com/v1',
    ai_key: 'sk-deepseek-xxxxxxxxxxxxxxxxxxxxxxxx',
    sync: false
  }
]

// 科目数据
export const testSubjects: Subject[] = [
  {
    id: 'sub_math_001',
    name: '高等数学',
    color: '#4285F4'
  },
  {
    id: 'sub_physics_001',
    name: '大学物理',
    color: '#FF5800'
  },
  {
    id: 'sub_chemistry_001',
    name: '无机化学',
    color: '#7B1FA2'
  },
  {
    id: 'sub_english_001',
    name: '大学英语',
    color: '#43A047'
  },
  {
    id: 'sub_computer_001',
    name: '计算机科学',
    color: '#F44336'
  }
]

// 题目来源数据
export const testSources: Source[] = [
  {
    id: 'src_001',
    question_id: 'eq_001',
    subject_id: 'sub_math_001',
    book: '高等数学（第七版）',
    chapter: '第一章 函数与极限',
    knowledge: '二次函数最值问题'
  },
  {
    id: 'src_002',
    question_id: 'eq_002',
    subject_id: 'sub_physics_001',
    book: '大学物理学（第四版）',
    chapter: '第二章 运动学',
    knowledge: '自由落体运动'
  },
  {
    id: 'src_003',
    question_id: 'eq_003',
    subject_id: 'sub_english_001',
    book: '新视野大学英语读写教程',
    chapter: 'Unit 3 Sports',
    knowledge: '英语全字母句翻译'
  },
  {
    id: 'src_004',
    question_id: 'eq_004',
    subject_id: 'sub_chemistry_001',
    book: '无机化学（第五版）',
    chapter: '第八章 酸碱平衡',
    knowledge: '强酸的性质'
  },
  {
    id: 'src_005',
    question_id: 'eq_005',
    subject_id: 'sub_computer_001',
    book: '数据结构与算法分析',
    chapter: '第三章 栈和队列',
    knowledge: '栈的应用'
  },
  // 新增的无机化学题目来源
  {
    id: 'src_006',
    question_id: 'eq_006',
    subject_id: 'sub_chemistry_001',
    book: '无机化学（第五版）',
    chapter: '第六章 金属元素化学',
    knowledge: '金属与酸的置换反应'
  },
  {
    id: 'src_007',
    question_id: 'eq_007',
    subject_id: 'sub_chemistry_001',
    book: '无机化学（第五版）',
    chapter: '第四章 元素周期律',
    knowledge: '金属性递变规律'
  },
  {
    id: 'src_008',
    question_id: 'eq_008',
    subject_id: 'sub_chemistry_001',
    book: '无机化学（第五版）',
    chapter: '第八章 酸碱平衡',
    knowledge: '强碱溶液pH计算'
  }
]

// 错题数据（扩展版）
export const testErrorQuestions: ErrorQuestion[] = [
  {
    id: 'eq_001',
    user_id: 'user_001',
    subject_id: 'sub_math_001',
    source_id: 'src_001',
    prompt: '求函数 f(x) = x² - 2x + 1 的最小值',
    type: QuestionType.Calculation,
    answer: '最小值为0，当x=1时取得',
    analysis: '这是一个二次函数，可以通过配方法或求导法找到最小值。配方得f(x)=(x-1)²，所以最小值为0。',
    error_note: '刚开始没有注意到这是完全平方式，导致计算复杂化'
  },
  {
    id: 'eq_002',
    user_id: 'user_001',
    subject_id: 'sub_physics_001',
    source_id: 'src_002',
    prompt: '一个质量为 2kg 的物体从 10m 高处自由落下，求落地时的速度',
    type: QuestionType.Calculation,
    answer: 'v = √(2gh) = √(2×9.8×10) ≈ 14 m/s',
    analysis: '使用自由落体运动公式 v² = 2gh，其中g=9.8m/s²，h=10m',
    error_note: '忘记了重力加速度的数值，公式记错了'
  },
  {
    id: 'eq_003',
    user_id: 'user_001',
    subject_id: 'sub_english_001',
    source_id: 'src_003',
    prompt: '翻译句子：The quick brown fox jumps over the lazy dog ',
    type: QuestionType.Essay,
    answer: '敏捷的棕色狐狸跳过懒惰的狗',
    analysis: '这是一个著名的全字母句，包含了英语的所有26个字母',
    error_note: '词汇量不够，几个关键词汇翻译不准确'
  },
  {
    id: 'eq_004',
    user_id: 'user_001',
    subject_id: 'sub_chemistry_001',
    source_id: 'src_004',
    prompt: '写出硫酸的化学式并说明其性质',
    type: QuestionType.ShortAnswer,
    answer: '化学式：H₂SO₄。性质：强酸性，具有强腐蚀性，能与金属反应产生氢气，是重要的化工原料',
    analysis: '硫酸是最常见的强酸之一，在工业上应用广泛',
    error_note: '化学式书写不规范，性质描述不够全面'
  },
  {
    id: 'eq_005',
    user_id: 'user_001',
    subject_id: 'sub_computer_001',
    source_id: 'src_005',
    prompt: '用栈实现表达式求值：计算 (3+4)*5-6 的值',
    type: QuestionType.Calculation,
    answer: '结果为29。使用两个栈分别存储操作数和运算符，按照运算符优先级进行计算。',
    analysis: '这是栈的经典应用，通过维护操作数栈和运算符栈来实现表达式求值',
    error_note: '没有正确处理运算符优先级，导致计算顺序错误'
  },
  // 新增的无机化学题目
  {
    id: 'eq_006',
    user_id: 'user_001',
    subject_id: 'sub_chemistry_001',
    source_id: 'src_006',
    prompt: '写出下列反应的化学方程式：锌与稀盐酸反应制取氢气',
    type: QuestionType.ShortAnswer,
    answer: 'Zn + 2HCl → ZnCl₂ + H₂↑',
    analysis: '这是一个典型的金属与酸反应制取氢气的置换反应。锌失去电子变成Zn²⁺，H⁺得到电子变成H₂气体。',
    error_note: '忘记配平化学方程式，氢原子数目不守恒'
  },
  {
    id: 'eq_007',
    user_id: 'user_001',
    subject_id: 'sub_chemistry_001',
    source_id: 'src_007',
    prompt: '比较Na、Mg、Al三种元素的金属性强弱，并说明判断依据',
    type: QuestionType.ShortAnswer,
    answer: '金属性强弱：Na > Mg > Al。判断依据：在同一周期中，从左到右金属性逐渐减弱；原子半径越大，失电子能力越强。',
    analysis: '根据元素周期律，在同一周期中，随着原子序数增加，金属性逐渐减弱。这与原子结构中电子层数相同但核电荷数增加有关。',
    error_note: '混淆了金属性和非金属性的概念，判断依据不充分'
  },
  {
    id: 'eq_008',
    user_id: 'user_001',
    subject_id: 'sub_chemistry_001',
    source_id: 'src_008',
    prompt: '计算0.1mol/L的NaOH溶液的pH值',
    type: QuestionType.Calculation,
    answer: 'pOH = -log[OH⁻] = -log(0.1) = 1，pH = 14 - pOH = 13',
    analysis: 'NaOH是强碱，完全电离。先计算pOH，再利用pH + pOH = 14的关系求得pH值。',
    error_note: '忘记了强碱完全电离的特性，计算过程出现错误'
  }
]

// SRS数据
export const testSRSData: SRSData[] = [
  {
    id: 'srs_001',
    question_id: 'eq_001',
    difficulty: 2,
    mastery: 65,
    lastreviewed_at: Date.now() - 2 * 24 * 60 * 60 * 1000, // 2天前
    review_count: 3
  },
  {
    id: 'srs_002',
    question_id: 'eq_002',
    difficulty: 3,
    mastery: 40,
    lastreviewed_at: Date.now() - 5 * 24 * 60 * 60 * 1000, // 5天前
    review_count: 2
  },
  {
    id: 'srs_003',
    question_id: 'eq_003',
    difficulty: 1,
    mastery: 85,
    lastreviewed_at: Date.now() - 1 * 24 * 60 * 60 * 1000, // 1天前
    review_count: 4
  },
  {
    id: 'srs_004',
    question_id: 'eq_004',
    difficulty: 2,
    mastery: 30,
    lastreviewed_at: Date.now() - 7 * 24 * 60 * 60 * 1000, // 7天前
    review_count: 1
  },
  {
    id: 'srs_005',
    question_id: 'eq_005',
    difficulty: 4,
    mastery: 20,
    lastreviewed_at: Date.now() - 10 * 24 * 60 * 60 * 1000, // 10天前
    review_count: 1
  },
  // 新增的无机化学题目SRS数据
  {
    id: 'srs_006',
    question_id: 'eq_006',
    difficulty: 2,
    mastery: 55,
    lastreviewed_at: Date.now() - 3 * 24 * 60 * 60 * 1000, // 3天前
    review_count: 2
  },
  {
    id: 'srs_007',
    question_id: 'eq_007',
    difficulty: 3,
    mastery: 45,
    lastreviewed_at: Date.now() - 4 * 24 * 60 * 60 * 1000, // 4天前
    review_count: 2
  },
  {
    id: 'srs_008',
    question_id: 'eq_008',
    difficulty: 2,
    mastery: 70,
    lastreviewed_at: Date.now() - 1 * 24 * 60 * 60 * 1000, // 1天前
    review_count: 3
  }
]

// 错题标签数据
export const testErrorTags: ErrorTags[] = [
  {
    id: 'tag_001',
    question_id: 'eq_001',
    name: '概念理解错误',
    color: '#FF5722'
  },
  {
    id: 'tag_002',
    question_id: 'eq_001',
    name: '计算粗心',
    color: '#FF9800'
  },
  {
    id: 'tag_003',
    question_id: 'eq_002',
    name: '公式记忆错误',
    color: '#F44336'
  },
  {
    id: 'tag_004',
    question_id: 'eq_003',
    name: '词汇量不足',
    color: '#9C27B0'
  },
  {
    id: 'tag_005',
    question_id: 'eq_004',
    name: '基础知识薄弱',
    color: '#3F51B5'
  },
  {
    id: 'tag_006',
    question_id: 'eq_005',
    name: '算法理解不深',
    color: '#03A9F4'
  },
  // 新增的无机化学题目标签
  {
    id: 'tag_007',
    question_id: 'eq_006',
    name: '化学方程式配平错误',
    color: '#E91E63'
  },
  {
    id: 'tag_008',
    question_id: 'eq_007',
    name: '概念混淆',
    color: '#FF5722'
  },
  {
    id: 'tag_009',
    question_id: 'eq_007',
    name: '理论理解不透彻',
    color: '#9C27B0'
  },
  {
    id: 'tag_010',
    question_id: 'eq_008',
    name: '计算步骤错误',
    color: '#FF9800'
  }
]

// 附件数据
export const testAttachments: Attachment[] = [
  {
    id: 'att_001',
    question_id: 'eq_001',
    type: 'original',
    file_type: 'img',
    base64_data: 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/5+hHgAHggJ/PchI7wAAAABJRU5ErkJggg==',
    hash: 'abc123def456'
  },
  {
    id: 'att_002',
    question_id: 'eq_002',
    type: 'answer',
    file_type: 'img',
    base64_data: 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/5+hHgAHggJ/PchI7wAAAABJRU5ErkJggg==',
    hash: 'def456ghi789'
  },
  {
    id: 'att_003',
    question_id: 'eq_005',
    type: 'original',
    file_type: 'img',
    base64_data: 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/5+hHgAHggJ/PchI7wAAAABJRU5ErkJggg==',
    hash: 'ghi789jkl012'
  }
]

// 数据关联查询函数
export const getDataUtils = {
  // 根据用户ID获取用户信息
  getUserById: (userId: string) => {
    return testUsers.find(user => user.student_num === userId.replace('user_', '')) || null
  },
  
  // 根据科目ID获取科目信息
  getSubjectById: (subjectId: string) => {
    return testSubjects.find(subject => subject.id === subjectId) || null
  },
  
  // 根据题目ID获取来源信息
  getSourceByQuestionId: (questionId: string) => {
    return testSources.find(source => source.question_id === questionId) || null
  },
  
  // 根据题目ID获取SRS数据
  getSRSByQuestionId: (questionId: string) => {
    return testSRSData.find(srs => srs.question_id === questionId) || null
  },
  
  // 根据题目ID获取标签
  getTagsByQuestionId: (questionId: string) => {
    return testErrorTags.filter(tag => tag.question_id === questionId)
  },
  
  // 根据题目ID获取附件
  getAttachmentsByQuestionId: (questionId: string) => {
    return testAttachments.filter(att => att.question_id === questionId)
  },
  
  // 获取用户的全部错题
  getErrorQuestionsByUserId: (userId: string) => {
    return testErrorQuestions.filter(eq => eq.user_id === userId)
  },
  
  // 获取科目的全部错题
  getErrorQuestionsBySubjectId: (subjectId: string) => {
    return testErrorQuestions.filter(eq => eq.subject_id === subjectId)
  }
}