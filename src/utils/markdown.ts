import DOMPurify from 'dompurify'
import hljs from 'highlight.js/lib/core'
import bash from 'highlight.js/lib/languages/bash'
import cpp from 'highlight.js/lib/languages/cpp'
import css from 'highlight.js/lib/languages/css'
import java from 'highlight.js/lib/languages/java'
import javascript from 'highlight.js/lib/languages/javascript'
import json from 'highlight.js/lib/languages/json'
import markdown from 'highlight.js/lib/languages/markdown'
import python from 'highlight.js/lib/languages/python'
import rust from 'highlight.js/lib/languages/rust'
import sql from 'highlight.js/lib/languages/sql'
import typescript from 'highlight.js/lib/languages/typescript'
import xml from 'highlight.js/lib/languages/xml'
import yaml from 'highlight.js/lib/languages/yaml'
import katex from 'katex'
import { Marked } from 'marked'
import markedKatex from 'marked-katex-extension'

const highlighter = hljs.newInstance()
for (const [name, language] of Object.entries({
  javascript,
  typescript,
  python,
  bash,
  json,
  css,
  sql,
  java,
  cpp,
  rust,
  xml,
  yaml,
  markdown
})) {
  highlighter.registerLanguage(name, language)
}

const escapeHtml = (value: string) =>
  value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
const parser = new Marked(
  markedKatex({
    throwOnError: false,
    trust: false,
    output: 'htmlAndMathml',
    nonStandard: true,
    strict: 'ignore'
  }),
  {
    extensions: [
      {
        name: 'texDelimitedMath',
        level: 'inline',
        start(src) {
          const index = src.search(/\\[([]/)
          return index < 0 ? undefined : index
        },
        tokenizer(src) {
          const match = /^(?:\\\(([\s\S]*?)\\\)|\\\[([\s\S]*?)\\\])/.exec(src)
          if (!match) return
          return {
            type: 'texDelimitedMath',
            raw: match[0],
            text: (match[1] ?? match[2]).trim(),
            displayMode: match[2] !== undefined
          }
        },
        renderer(token) {
          return katex.renderToString(token.text, {
            displayMode: token.displayMode,
            throwOnError: false,
            trust: false,
            output: 'htmlAndMathml',
            strict: 'ignore'
          })
        }
      }
    ],
    async: false,
    gfm: true,
    breaks: true,
    renderer: {
      // Notebook content is Markdown, not an executable HTML document.
      html({ text }) {
        return escapeHtml(text)
      },
      code({ text, lang }) {
        const language = lang?.split(/\s/)[0] || ''
        const known = !!highlighter.getLanguage(language)
        const content = known
          ? highlighter.highlight(text, { language }).value
          : escapeHtml(text)
        return (
          '<pre><code class="hljs' +
          (known ? ' language-' + escapeHtml(language) : '') +
          '">' +
          content +
          '</code></pre>'
        )
      }
    }
  }
)

/** Shared browser/WebView rendering boundary for previews and HTML exports. */
export function renderMarkdown(value: string | null | undefined): string {
  if (!value) return ''
  return DOMPurify.sanitize(parser.parse(value) as string)
}
