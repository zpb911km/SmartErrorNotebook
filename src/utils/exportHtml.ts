import type { ErrorQuestion } from '../types'
import { Marked } from 'marked'
import markedKatex from 'marked-katex-extension'
import { showError } from './notification'
import { exportFile } from './exportFile'

const _marked = new Marked(
  markedKatex({
    throwOnError: false,
    output: 'html',
    nonStandard: true,
    strict: 'ignore'
  }),
  {
    renderer: {
      code({ text, lang }) {
        const e = text
          .replace(/&/g, '&amp;')
          .replace(/</g, '&lt;')
          .replace(/>/g, '&gt;')
        return `<pre><code class="hljs ${lang ? `language-${lang}` : ''}">${e}</code></pre>`
      }
    }
  }
)

function renderHtml(t: string | undefined | null): string {
  if (!t) return ''
  const c = (t || '').replace(/[①-⑳]/g, (m) => `(${m.charCodeAt(0) - 0x245f})`)
  return _marked.parse(
    c
      .replace(/\\\[/g, '$$$$')
      .replace(/\\\]/g, '$$$$')
      .replace(/\\\(/g, '$')
      .replace(/\\\)/g, '$'),
    { breaks: true, gfm: true }
  ) as string
}

/**
 * 构建独立 HTML 字符串（纯函数，不涉及 IO）
 *
 * 根据 localStorage 中 export_include_answer 的值决定是否包含答案和解析。
 * 默认只导出题目（prompt），可在设置页修改。
 *
 * @param questions     错题列表
 * @param includeAnswer 是否包含答案和解析（不传则从 localStorage 读取）
 * @returns HTML 字符串
 */
export function buildQuestionsHTML(
  questions: ErrorQuestion[],
  includeAnswer?: boolean
): string {
  // 读取是否包含答案和解析的设置（默认 false）
  const showAnswer =
    includeAnswer ?? localStorage.getItem('export_include_answer') === 'true'

  const cards = questions
    .map((q, i) => buildCardHtml(q, i, showAnswer))
    .join('\n')
  const date = new Date().toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'long',
    day: 'numeric'
  })

  return `<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>错题集</title>
  <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/katex.min.css"
        crossorigin="anonymous">
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body {
      font-family: "Microsoft YaHei", "PingFang SC", "Noto Sans SC", "KaTeX_Main", serif;
      color: #333;
      line-height: 1.7;
      padding: 20px;
      background: #fff;
      max-width: 800px;
      margin: 0 auto;
    }
    .header {
      text-align: center;
      padding: 20px 0;
      border-bottom: 2px solid #1976d2;
      margin-bottom: 24px;
    }
    .header h1 { font-size: 24px; color: #1976d2; margin: 0 0 8px; }
    .header .meta { font-size: 14px; color: #666; }
    .card {
      padding: 16px 20px;
      margin-bottom: 16px;
      border: 1px solid #e0e0e0;
      border-radius: 8px;
      background: #fff;
      page-break-inside: avoid;
    }
    .card-number {
      font-size: 17px;
      font-weight: 700;
      color: #1976d2;
      margin-bottom: 12px;
      padding-bottom: 8px;
      border-bottom: 1px solid #eee;
    }
    .card-section { margin-bottom: 10px; }
    .card-section:last-child { margin-bottom: 0; }
    .section-label {
      font-size: 13px;
      font-weight: 600;
      color: #888;
      margin-bottom: 4px;
    }
    .section-label.answer { color: #2e7d32; }
    .section-label.analysis { color: #c62828; }
    .content {
      color: #333;
      white-space: pre-wrap;
      word-wrap: break-word;
      line-height: 1.7;
      font-size: 14px;
    }
    .content p { margin: 0.5em 0; }
    .content h1, .content h2, .content h3,
    .content h4, .content h5, .content h6 {
      margin: 0.8em 0 0.4em;
      font-weight: 600;
      font-size: 1em;
    }
    .content pre {
      background: #f5f5f5;
      padding: 10px;
      border-radius: 6px;
      overflow-x: auto;
      margin: 0.5em 0;
    }
    .content pre code {
      background: transparent;
      padding: 0;
      color: #333;
      font-size: 13px;
      font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    }
    .content code {
      background: rgba(25, 118, 210, 0.12);
      padding: 2px 6px;
      border-radius: 3px;
      font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
      font-size: 0.9em;
    }
    .content ul, .content ol {
      padding-left: 20px;
      margin: 0.5em 0;
    }
    .content blockquote {
      margin: 0.5em 0;
      padding-left: 10px;
      border-left: 3px solid #e0e0e0;
      color: #666;
    }
    .content a { color: #1976d2; text-decoration: underline; }
    .content table {
      border-collapse: collapse;
      width: 100%;
      margin: 0.5em 0;
    }
    .content th, .content td {
      border: 1px solid #e0e0e0;
      padding: 6px 8px;
    }
    .content img { max-width: 100%; }
    .content .katex { font-size: 1.1em; }
    .content .katex-display {
      display: block;
      text-align: center;
      margin: 0.5em 0;
      overflow-x: auto;
      overflow-y: hidden;
    }
    .content .katex-display > .katex { font-size: 1.15em; }
    .footer {
      text-align: center;
      font-size: 12px;
      color: #999;
      padding: 20px 0 10px;
    }
    @media print {
      body { padding: 0; }
      .card { break-inside: avoid; }
    }
  </style>
</head>
<body>
  <div class="header">
    <h1>错题集</h1>
    <div class="meta">导出日期：${date} | 共 ${questions.length} 题</div>
  </div>
  ${cards}
  <div class="footer">由 SmartErrorNotebook 生成</div>
</body>
</html>`
}

/**
 * 将错题导出为独立 HTML 文件
 *
 * 根据 localStorage 中 export_include_answer 的值决定是否包含答案和解析。
 * 默认只导出题目（prompt），可在设置页修改。
 */
export async function exportQuestionsToHTML(
  questions: ErrorQuestion[]
): Promise<boolean> {
  if (questions.length === 0) {
    showError('导出失败', '当前筛选条件下没有可导出的错题')
    return false
  }

  try {
    const html = buildQuestionsHTML(questions)
    const filename = `错题集_${new Date().toISOString().slice(0, 10)}.html`
    return await exportFile(filename, html, 'text/html;charset=utf-8')
  } catch (e) {
    console.error('导出 HTML 失败:', e)
    showError('导出失败', `HTML 导出过程中出现错误: ${String(e)}`)
    return false
  }
}

function buildCardHtml(
  q: ErrorQuestion,
  i: number,
  includeAnswer: boolean
): string {
  let sections = `
    <div class="section-label">题目</div>
    <div class="content">${renderHtml(q.prompt)}</div>
  `

  if (includeAnswer) {
    if (q.answer) {
      sections += `
    <div class="section-label answer">参考答案</div>
    <div class="content">${renderHtml(q.answer)}</div>
      `
    }
    if (q.analysis) {
      sections += `
    <div class="section-label analysis">解析</div>
    <div class="content">${renderHtml(q.analysis)}</div>
      `
    }
  }

  return `
<div class="card">
  <div class="card-number">第 ${i + 1} 题</div>
  <div class="card-section">${sections}</div>
</div>`
}
