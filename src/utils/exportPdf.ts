import { showError } from './notification'
import type { ErrorQuestion } from '../types'
import { jsPDF } from 'jspdf'
import { save } from '@tauri-apps/plugin-dialog'
import { writeFile } from '@tauri-apps/plugin-fs'
import { Marked } from 'marked'
import markedKatex from 'marked-katex-extension'
import html2canvas from 'html2canvas'
import 'katex/dist/katex.min.css'

const _marked = new Marked(
  markedKatex({ throwOnError: false, output: 'html', nonStandard: true, strict: 'ignore' }),
  { renderer: { code({ text, lang }) {
    const e = text.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;')
    return `<pre><code class="hljs ${lang?`language-${lang}`:''}">${e}</code></pre>`
  }}}
)

function renderHtml(t: string|undefined|null): string {
  if (!t) return ''
  const c = (t||'').replace(/[①-⑳]/g,m=>`(${m.charCodeAt(0)-0x245f})`)
  return _marked.parse(c.replace(/\\\[/g,'$$$$').replace(/\\\]/g,'$$$$').replace(/\\\(/g,'$').replace(/\\\)/g,'$'), {breaks:true,gfm:true}) as string
}

export async function exportQuestionsToPDF(questions: ErrorQuestion[]): Promise<boolean> {
  if (!questions.length) { showError('导出失败','没有可导出的错题'); return false }

  try {
    const cards = questions.map((q,i)=>buildCardHtml(q,i)).join('\n')
    const date = new Date().toLocaleDateString('zh-CN',{year:'numeric',month:'long',day:'numeric'})

    const container = document.createElement('div')
    container.style.cssText = 'position:fixed;top:0;left:0;width:800px;z-index:-1000;pointer-events:none'
    container.innerHTML = `<div id="pdfr" style="font-family:'Microsoft YaHei','PingFang SC','Noto Sans SC','KaTeX_Main',serif;color:#333;line-height:1.7;padding:0;background:#fff;width:800px;font-size:14px">
  <div style="text-align:center;padding:18px 0;border-bottom:2px solid #1976d2;margin-bottom:20px">
    <div style="font-size:24px;color:#1976d2;font-weight:700;margin-bottom:6px">错题集</div>
    <div style="font-size:13px;color:#555">导出日期：${date} | 共 ${questions.length} 题</div>
  </div>
  ${cards}
  <div style="text-align:center;font-size:11px;color:#999;padding:10px 0">由 SmartErrorNotebook 生成</div>
</div>`
    document.body.appendChild(container)

    const root = document.getElementById('pdfr') as HTMLElement
    if (!root) { document.body.removeChild(container); throw new Error('未找到根元素') }

    await document.fonts.ready

    const canvas = await html2canvas(root, {
      scale: 2, useCORS: true, logging: false, backgroundColor: '#ffffff',
      width: 800, height: root.scrollHeight,
      windowWidth: 800, windowHeight: root.scrollHeight
    })

    document.body.removeChild(container)
    if (!canvas.width || !canvas.height) throw new Error('截图为空')

    const pdf = new jsPDF({ unit:'mm', format:'a4', orientation:'portrait' })
    const CW = 186, CH = 273
    const pxPerMm = canvas.width / CW, pagePx = CH * pxPerMm
    const total = Math.ceil(canvas.height / pagePx)
    for (let p = 0; p < total; p++) {
      if (p > 0) pdf.addPage()
      const sy = Math.round(p * pagePx), sh = Math.min(Math.round(pagePx), canvas.height - sy)
      const t = document.createElement('canvas')
      t.width = canvas.width; t.height = sh
      t.getContext('2d')!.drawImage(canvas, 0, sy, canvas.width, sh, 0, 0, canvas.width, sh)
      pdf.addImage(t.toDataURL('image/jpeg', 0.95), 'JPEG', 12, 12, CW, sh / pxPerMm)
      pdf.setFontSize(9); pdf.setTextColor(100,100,100)
      pdf.text(`- ${p+1} / ${total} -`, 105, 291, { align:'center' })
    }

    // 弹出保存对话框 → 通过 Tauri fs 写入（移动端兼容）
    const filePath = await save({
      defaultPath: `错题集_${new Date().toISOString().slice(0,10)}.pdf`,
      filters: [{ name:'PDF 文件', extensions:['pdf'] }]
    })
    if (!filePath) return false

    // jsPDF blob → Uint8Array → writeFile
    const blob = pdf.output('blob')
    const arrayBuffer = await blob.arrayBuffer()
    const uint8Array = new Uint8Array(arrayBuffer)
    await writeFile(filePath, uint8Array)
    return true
  } catch(e) {
    console.error('导出 PDF 失败:', e)
    showError('导出失败', `PDF 导出过程中出现错误: ${String(e)}`)
    return false
  }
}

function buildCardHtml(q: ErrorQuestion, i: number): string {
  return `<div style="padding:14px 18px;margin-bottom:14px;border:1px solid #ccc;border-radius:6px;background:#fff">
  <div style="font-size:16px;font-weight:700;color:#1565c0;margin-bottom:10px;padding-bottom:6px;border-bottom:1px solid #e0e0e0">第 ${i+1} 题</div>
  <div style="margin-bottom:8px">
    <div style="font-size:12px;font-weight:600;color:#555;margin-bottom:3px;text-transform:uppercase;letter-spacing:0.5px">题目</div>
    <div style="color:#333;white-space:pre-wrap;word-wrap:break-word;line-height:1.7">${renderHtml(q.prompt)}</div>
  </div>
  ${q.answer?`<div style="margin-bottom:8px"><div style="font-size:12px;font-weight:600;color:#2e7d32;margin-bottom:3px">参考答案</div><div style="color:#333;white-space:pre-wrap;word-wrap:break-word;line-height:1.7">${renderHtml(q.answer)}</div></div>`:''}
  ${q.analysis?`<div style="margin-bottom:8px"><div style="font-size:12px;font-weight:600;color:#c62828;margin-bottom:3px">解析</div><div style="color:#333;white-space:pre-wrap;word-wrap:break-word;line-height:1.7">${renderHtml(q.analysis)}</div></div>`:''}
</div>`
}
