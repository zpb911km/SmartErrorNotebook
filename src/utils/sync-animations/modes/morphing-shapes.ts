/**
 * 形态变形 (Morphing Shapes)
 *
 * 生成艺术灵感: 几何形状平滑过渡
 * 在三角形→正方形→五边形→六边形之间循环变形，
 * 颜色渐变填充，边缘发光。
 * 每次随机参数: 形状序列、颜色、变形速度
 */

import type { AnimationMode } from '../types'
import { SeededRandom, hslToString } from '../utils'

/** 获取正 N 边形的顶点 */
function getPolygonVertices(
  sides: number,
  cx: number,
  cy: number,
  radius: number,
  rotation: number
): { x: number; y: number }[] {
  const verts: { x: number; y: number }[] = []
  for (let i = 0; i < sides; i++) {
    const angle = rotation + (Math.PI * 2 * i) / sides - Math.PI / 2
    verts.push({
      x: cx + Math.cos(angle) * radius,
      y: cy + Math.sin(angle) * radius
    })
  }
  return verts
}

/** 在两个多边形顶点之间进行线性插值（M 对 N 映射为 M*N 对） */
function interpolatePolygons(
  vertsA: { x: number; y: number }[],
  vertsB: { x: number; y: number }[],
  t: number
): { x: number; y: number }[] {
  const maxLen = Math.max(vertsA.length, vertsB.length)
  const result: { x: number; y: number }[] = []

  for (let i = 0; i < maxLen; i++) {
    const aIdx = Math.floor((i / maxLen) * vertsA.length)
    const bIdx = Math.floor((i / maxLen) * vertsB.length)
    result.push({
      x: vertsA[aIdx].x + (vertsB[bIdx].x - vertsA[aIdx].x) * t,
      y: vertsA[aIdx].y + (vertsB[bIdx].y - vertsA[aIdx].y) * t
    })
  }
  return result
}

export class MorphingShapes implements AnimationMode {
  name = 'morphing-shapes'

  private width = 0
  private height = 0
  private cx = 0
  private cy = 0
  private time = 0
  private dark = false

  private sidesSequence: number[] = [3, 4, 5, 6]
  private morphDuration = 90 // 帧数
  private baseHue = 200
  private hueShift = 30
  private radius = 0
  private rotationSpeed = 0.01
  private innerShapes = 3

  init(
    _ctx: CanvasRenderingContext2D,
    width: number,
    height: number,
    seed: number
  ): void {
    this.width = width
    this.height = height
    this.cx = width / 2
    this.cy = height / 2
    this.time = 0

    const rng = new SeededRandom(seed)
    this.baseHue = rng.range(0, 360)
    this.hueShift = rng.range(20, 60)
    this.morphDuration = rng.int(60, 120)
    this.rotationSpeed = rng.range(0.005, 0.02)
    this.innerShapes = rng.int(2, 5)
    this.radius = Math.min(width, height) * 0.3

    // 随机排列形状顺序
    const baseSides = [3, 4, 5, 6]
    const shuffled: number[] = []
    const copy = [...baseSides]
    for (let i = copy.length - 1; i >= 0; i--) {
      const idx = Math.floor(rng.next() * (i + 1))
      shuffled.push(copy[idx])
      copy.splice(idx, 1)
    }
    this.sidesSequence = shuffled
  }

  update(_progress: number, isDarkTheme: boolean): void {
    this.time++
    this.dark = isDarkTheme
  }

  draw(ctx: CanvasRenderingContext2D): void {
    // 半透明叠加制造拖尾
    ctx.fillStyle = this.dark ? 'rgba(33,33,33,0.06)' : 'rgba(255,255,255,0.06)'
    ctx.fillRect(0, 0, this.width, this.height)

    const totalFrames = this.morphDuration * this.sidesSequence.length
    const cycleTime = this.time % totalFrames
    const morphIndex = Math.floor(cycleTime / this.morphDuration)
    const morphT = (cycleTime % this.morphDuration) / this.morphDuration

    // 缓动
    const easedT =
      morphT < 0.5 ? 2 * morphT * morphT : 1 - Math.pow(-2 * morphT + 2, 2) / 2

    const sA = this.sidesSequence[morphIndex % this.sidesSequence.length]
    const sB = this.sidesSequence[(morphIndex + 1) % this.sidesSequence.length]

    const rotation = this.time * this.rotationSpeed
    const l = this.dark ? 65 : 40
    const s = this.dark ? 50 : 60

    // 绘制多层嵌套形状
    for (let layer = 0; layer < this.innerShapes; layer++) {
      const layerT = layer / Math.max(this.innerShapes - 1, 1)
      const scale = 1 - layerT * 0.35
      const r =
        this.radius * scale * (0.85 + 0.15 * Math.sin(this.time * 0.01 + layer))
      const dir = layer % 2 === 0 ? 1 : -1
      const rot = rotation * dir * (1 + layer * 0.3)
      const hue = (this.baseHue + layer * this.hueShift + this.time * 0.5) % 360

      const vertsA = getPolygonVertices(sA, this.cx, this.cy, r, rot)
      const vertsB = getPolygonVertices(sB, this.cx, this.cy, r, rot + 0.3)

      const verts = interpolatePolygons(vertsA, vertsB, easedT)

      // 绘制填充
      ctx.beginPath()
      ctx.moveTo(verts[0].x, verts[0].y)
      for (let i = 1; i < verts.length; i++) {
        ctx.lineTo(verts[i].x, verts[i].y)
      }
      ctx.closePath()

      ctx.fillStyle = hslToString(hue, s, l + layerT * 15, 0.25 + layerT * 0.15)
      ctx.fill()

      // 绘制边缘
      ctx.strokeStyle = hslToString(
        hue + 20,
        s + 10,
        l + 10,
        0.6 + layerT * 0.2
      )
      ctx.lineWidth = 1 + (1 - layerT) * 2
      ctx.stroke()
    }

    // 中心亮点
    const glowHue = (this.baseHue + this.time * 0.5) % 360
    ctx.beginPath()
    ctx.arc(
      this.cx,
      this.cy,
      2 + Math.sin(this.time * 0.05) * 1.5,
      0,
      Math.PI * 2
    )
    ctx.fillStyle = hslToString(glowHue, 80, 70, 0.8)
    ctx.fill()
  }

  destroy(): void {
    // nothing to clean
  }
}
