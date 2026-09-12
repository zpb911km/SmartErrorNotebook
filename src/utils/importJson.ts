import { loadQuestionLibrary } from '../services/questionQueries'
import {
  createQuestionEditor,
  type SaveState
} from '../services/questionEditor'
import { parseQuestionType } from './questionDisplay'
import { errorMessage } from './errors'
import type { ExportJSONSchema } from '../types/transfer'

export function parseImportFile(content: string): {
  questions: ExportJSONSchema['questions']
  version: string
  error?: string
} {
  let data: unknown
  try {
    data = JSON.parse(content)
  } catch {
    return { questions: [], version: '', error: '文件不是有效的 JSON 格式' }
  }
  if (
    !data ||
    typeof data !== 'object' ||
    !('version' in data) ||
    typeof data.version !== 'string' ||
    !data.version ||
    !('questions' in data) ||
    !Array.isArray(data.questions) ||
    !data.questions.length
  ) {
    return {
      questions: [],
      version: '',
      error: '文件必须包含版本号和非空 questions 数组'
    }
  }
  const questions: ExportJSONSchema['questions'] = []
  for (const [index, item] of data.questions.entries()) {
    if (
      !item ||
      typeof item !== 'object' ||
      typeof item.prompt !== 'string' ||
      !item.prompt ||
      (item.answer !== undefined && typeof item.answer !== 'string') ||
      (item.analysis !== undefined && typeof item.analysis !== 'string')
    ) {
      return {
        questions: [],
        version: '',
        error: `第 ${index + 1} 条记录的题干、答案或解析无效`
      }
    }
    questions.push({
      prompt: item.prompt,
      answer: item.answer ?? '',
      analysis: item.analysis ?? ''
    })
  }
  return { questions, version: data.version }
}

/** Each imported row owns its resource checkpoints for retries in this session. */
const sessions = new WeakMap<
  object,
  { editor: ReturnType<typeof createQuestionEditor>; state: SaveState }
>()
export async function importSingleQuestion(
  question: ExportJSONSchema['questions'][0],
  subjectId: string,
  typeName: string,
  tags: Array<{ name: string; color: string }> = []
): Promise<{ success: boolean; error?: string }> {
  let session = sessions.get(question)
  if (!session) {
    const state: SaveState = {
      stage: 'idle',
      committed: null,
      cleanupIds: [],
      error: null
    }
    session = { editor: createQuestionEditor(state), state }
    sessions.set(question, session)
  }
  if (session.state.committed) return { success: true }
  try {
    await session.editor.save({
      source: subjectId
        ? { kind: 'new', subjectId, book: null, chapter: null, knowledge: null }
        : { kind: 'none' },
      questionType: parseQuestionType(typeName),
      stem: question.prompt,
      correctAnswer: question.answer || '',
      explanation: question.analysis || null,
      note: null,
      tags: tags.map((tag) => ({ ...tag })),
      attachments: []
    })
    return { success: true }
  } catch (error) {
    return { success: false, error: errorMessage(error) }
  }
}

export async function getExistingPromptSet(): Promise<Set<string>> {
  // Failure must reach the importer: an empty fallback silently bypasses deduplication.
  const library = await loadQuestionLibrary()
  return new Set(library.items.map((question) => question.stem.trim()))
}
