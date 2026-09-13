// @vitest-environment happy-dom
import { describe, expect, it } from 'vitest'

import { renderMarkdown } from '../src/utils/markdown'

describe('shared Markdown rendering boundary', () => {
  it('renders math and highlighted code consistently across calls', () => {
    const input = '求 \\(x^2\\)\n\n```javascript\nconst x = 2\n```'
    const html = renderMarkdown(input)
    expect(html).toContain('class="katex"')
    expect(html).toContain('hljs-keyword')
    expect(renderMarkdown(input)).toBe(html)
  })

  it('preserves TeX delimiters inside code', () => {
    const element = document.createElement('div')
    element.innerHTML = renderMarkdown(
      '`\\(x\\)`\n\n```text\n\\[x\\]\n```\n\n    \\(y\\)'
    )
    expect(
      [...element.querySelectorAll('code')].map((code) =>
        code.textContent?.trim()
      )
    ).toEqual(['\\(x\\)', '\\[x\\]', '\\(y\\)'])
    expect(element.querySelector('.katex')).toBeNull()
  })

  // Sanitizer security is tested in a real browser in tests/browser/ui.spec.ts.
  it.each([
    ['``a ` \\(x\\)``', 'a ` \\(x\\)'],
    ['``a\n\\(x\\)``', 'a \\(x\\)'],
    ['```text\n```not-a-close\n\\(x\\)\n```', '```not-a-close\n\\(x\\)'],
    ['````text\n```\n\\[x\\]\n````', '```\n\\[x\\]'],
    ['~~~text\n~~~not-a-close\n\\(x\\)\n~~~', '~~~not-a-close\n\\(x\\)'],
    ['> ```text\n> \\(x\\)\n> ```', '\\(x\\)'],
    ['- item\n\n  ```text\n  \\(x\\)\n  ```', '\\(x\\)']
  ])('preserves code in %s', (input, expected) => {
    const element = document.createElement('div')
    element.innerHTML = renderMarkdown(input + '\n\nOutside \\(y\\)')
    expect(element.querySelector('code')?.textContent).toBe(expected)
    expect(element.querySelectorAll('.katex')).toHaveLength(1)
  })

  it('renders inline, multiline display and dollar math', () => {
    const element = document.createElement('div')
    element.innerHTML = renderMarkdown(
      '\\(x\\)\n\n\\[\nx^2\n\\]\n\n$y$\n\n$$\nz^2\n$$'
    )
    expect(element.querySelectorAll('.katex')).toHaveLength(4)
    expect(element.querySelectorAll('.katex-display')).toHaveLength(2)
  })
  // happy-dom is only used here for rendering logic, not security guarantees.

  it('handles empty input and unknown code languages without injecting markup', () => {
    expect(renderMarkdown(null)).toBe('')
    const element = document.createElement('div')
    element.innerHTML = renderMarkdown(
      '```unknown\n<img src=x onerror=alert(1)>\n```'
    )
    expect(element.querySelector('img')).toBeNull()
    expect(element.querySelector('code')?.textContent).toBe(
      '<img src=x onerror=alert(1)>'
    )
  })
})
