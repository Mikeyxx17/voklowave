import { marked } from 'marked'
import DOMPurify from 'dompurify'
import katex from 'katex'

// ── 数学公式占位符 ──
const MATH_PLACEHOLDER = '%%MATH%%'

marked.setOptions({
  breaks: true,
  gfm: true,
})

/**
 * 将 Markdown 文本转为安全的 HTML。
 * 处理流程：提取公式 → marked 解析 → DOMPurify 清洗 → 还原公式。
 * 支持 $...$（行内）和 $$...$$（块级）。
 */
export function renderMarkdown(text) {
  if (!text) return ''

  // ── 步骤 1：提取所有公式，替换为占位符 ──
  const formulas = []
  let processed = text

  // 先提取块级公式 $$...$$（避免与行内 $ 冲突）
  processed = processed.replace(/\$\$([^$]+)\$\$/g, (_, formula) => {
    formulas.push({ type: 'block', formula: formula.trim() })
    return MATH_PLACEHOLDER
  })

  // 再提取行内公式 $...$（不跨行）
  processed = processed.replace(/\$([^$\n]+)\$/g, (_, formula) => {
    formulas.push({ type: 'inline', formula: formula.trim() })
    return MATH_PLACEHOLDER
  })

  // ── 步骤 2：marked 解析 ──
  const raw = marked.parse(processed)

  // ── 步骤 3：DOMPurify 清洗 ──
  const clean = DOMPurify.sanitize(raw)

  // ── 步骤 4：还原公式（将占位符替换为 KaTeX HTML） ──
  let idx = 0
  const result = clean.replace(
    new RegExp(MATH_PLACEHOLDER.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'g'),
    () => {
      if (idx >= formulas.length) return ''
      const f = formulas[idx++]
      try {
        return katex.renderToString(f.formula, {
          displayMode: f.type === 'block',
          throwOnError: false,
        })
      } catch {
        return `<em>[公式解析错误]</em>`
      }
    },
  )

  return result
}
