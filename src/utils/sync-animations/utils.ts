/**
 * 同步动画工具函数
 *
 * 包含：Perlin Noise、确定性随机、HSL 颜色工具
 */

/* ── 确定性随机（基于种子） ── */

export class SeededRandom {
  private s: number

  constructor(seed: number) {
    this.s = seed % 2147483647
    if (this.s <= 0) this.s += 2147483646
  }

  /** 返回 [0, 1) */
  next(): number {
    this.s = (this.s * 16807) % 2147483647
    return (this.s - 1) / 2147483646
  }

  /** 返回 [min, max) */
  range(min: number, max: number): number {
    return min + this.next() * (max - min)
  }

  /** 返回 [min, max] 整数 */
  int(min: number, max: number): number {
    return Math.floor(this.range(min, max + 1))
  }

  /** 从数组中随机选一个 */
  pick<T>(arr: T[]): T {
    return arr[this.int(0, arr.length - 1)]
  }
}

/* ── 简易 Perlin Noise ── */

const PERM: number[] = []
;(function initPerm() {
  const p = []
  for (let i = 0; i < 256; i++) p[i] = i
  // Fisher-Yates
  for (let i = 255; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1))
    ;[p[i], p[j]] = [p[j], p[i]]
  }
  for (let i = 0; i < 512; i++) PERM[i] = p[i & 255]
})()

function fade(t: number): number {
  return t * t * t * (t * (t * 6 - 15) + 10)
}

function lerp(a: number, b: number, t: number): number {
  return a + t * (b - a)
}

function grad(hash: number, x: number, y: number): number {
  const h = hash & 3
  const u = h < 2 ? x : y
  const v = h < 2 ? y : x
  return ((h & 1) === 0 ? u : -u) + ((h & 2) === 0 ? v : -v)
}

/** 2D Perlin Noise，返回 [-1, 1] */
export function perlin2D(x: number, y: number): number {
  const xi = Math.floor(x) & 255
  const yi = Math.floor(y) & 255
  const xf = x - Math.floor(x)
  const yf = y - Math.floor(y)
  const u = fade(xf)
  const v = fade(yf)

  const aa = PERM[PERM[xi] + yi]
  const ab = PERM[PERM[xi] + yi + 1]
  const ba = PERM[PERM[xi + 1] + yi]
  const bb = PERM[PERM[xi + 1] + yi + 1]

  const x1 = lerp(grad(aa, xf, yf), grad(ba, xf - 1, yf), u)
  const x2 = lerp(grad(ab, xf, yf - 1), grad(bb, xf - 1, yf - 1), u)
  return lerp(x1, x2, v)
}

/** 多层 Perlin Noise（分形叠加），返回 [0, 1] */
export function fbm(x: number, y: number, octaves = 3): number {
  let value = 0
  let amplitude = 0.5
  let frequency = 1
  for (let i = 0; i < octaves; i++) {
    value += amplitude * perlin2D(x * frequency, y * frequency)
    amplitude *= 0.5
    frequency *= 2
  }
  return (value + 1) / 2 // 映射到 [0, 1]
}

/* ── 颜色工具 ── */

export function hslToString(h: number, s: number, l: number, a = 1): string {
  return `hsla(${h}, ${s}%, ${l}%, ${a})`
}

/** 将 hex 颜色解析为 RGB 对象 */
export function hexToRgb(
  hex: string
): { r: number; g: number; b: number } | null {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex)
  return result
    ? {
        r: parseInt(result[1], 16),
        g: parseInt(result[2], 16),
        b: parseInt(result[3], 16)
      }
    : null
}

/** 读取 CSS 变量值（从 document.body 计算样式） */
export function getCSSVar(name: string): string {
  return getComputedStyle(document.body).getPropertyValue(name).trim()
}

/** 获取当前主题色（primary color）作为 RGB 字符串 */
export function getThemeColorRGB(): string {
  const color = getCSSVar('--primary-color')
  if (!color) return '25, 118, 210' // fallback #1976d2
  const rgb = hexToRgb(color)
  return rgb ? `${rgb.r}, ${rgb.g}, ${rgb.b}` : '25, 118, 210'
}

/** 检测当前是否为暗色主题 */
export function isDarkTheme(): boolean {
  return document.body.classList.contains('dark-theme')
}

/* ── 数学工具 ── */

/** 将值从一个范围映射到另一个范围 */
export function mapRange(
  value: number,
  inMin: number,
  inMax: number,
  outMin: number,
  outMax: number
): number {
  return outMin + ((value - inMin) / (inMax - inMin)) * (outMax - outMin)
}

/** 将弧度制角度归一化到 [0, 2PI) */
export function normalizeAngle(angle: number): number {
  while (angle < 0) angle += Math.PI * 2
  while (angle >= Math.PI * 2) angle -= Math.PI * 2
  return angle
}

/** 两点距离 */
export function dist(x1: number, y1: number, x2: number, y2: number): number {
  const dx = x2 - x1
  const dy = y2 - y1
  return Math.sqrt(dx * dx + dy * dy)
}
