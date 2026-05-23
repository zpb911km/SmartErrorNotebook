/**
 * 格式化
 * @param input
 * @returns
 */
export function format(input: string): string {
  let text = input
  console.log('format input:', input)
  text = mathFormat(text)
  console.log('formatted math:', text)
  text = removeMarkdownOuterTags(text)
  console.log('format output:', text)
  return text
}


/**
 * 去除外层```markdown...所有的内容...```标签
 * @param input
 * @returns
 */
function removeMarkdownOuterTags(input: string): string {
  return input.replace(/```[markdown|Markdown]*?```/g, '').trim()
}


/**
 * 格式化 llm 返回的文本，主要是数学公式
 * 1. 识别公式定界符：$$,$,\(\),\[\]
 * 2. 归一化 LaTeX 风格为 KaTeX 风格
 * 3. 逐字符扫描提取公式内容
 * 4. 行内公式去除换行并保留单空格间距
 * @param input
 * @returns
 */
function mathFormat(input: string): string {
  if (!input) return input

  let text = input

  // ── 1. 转换 LaTeX 风格为 KaTeX 风格 ──
  text = text.replace(/\\\[/g, '$$').replace(/\\\]/g, '$$')
  text = text.replace(/\\\(/g, '$').replace(/\\\)/g, '$')

  const result: string[] = []
  let i = 0

  // ── 2. 逐字符扫描提取公式 ──
  while (i < text.length) {
    const ch = text[i]

    // 显示公式 $$...$$
    if (ch === '$' && text[i + 1] === '$') {
      i += 2
      const close = text.indexOf('$$', i)
      if (close === -1) {
        // 未闭合，原样输出剩余内容
        result.push(text.slice(i - 2))
        break
      }
      const content = text.slice(i, close)
      result.push(`\n$$\n${content}\n$$\n`)
      i = close + 2
      continue
    }

    // 行内公式 $...$
    if (ch === '$' && text[i + 1] !== '$') {
      i++
      let content = ''
      while (i < text.length && text[i] !== '$') {
        content += text[i]
        i++
      }
      if (text[i] === '$') {
        i++ // 跳过结束的 $
        // 去除换行和多余空格
        const cleaned = content.trim().replace(/[\r\n]+/g, ' ').replace(/  +/g, ' ')
        result.push(` $${cleaned}$ `)
      } else {
        // 未闭合，原样输出
        result.push(text.slice(i - 1))
        break
      }
      continue
    }

    // 普通字符
    result.push(ch)
    i++
  }

  return result.join('')
}
