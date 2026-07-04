/**
 * 几何分形旋转 (Fractal Rotation)
 *
 * 生成艺术灵感: 递归旋转几何图形 / 万花筒
 * 多层递归正方形/多边形，每层旋转不同角度并缩小，
 * 形成深邃的迷幻旋转效果。
 * 每次随机参数: 形状种类、旋转方向、颜色渐变、递归深度
 */

import type { AnimationMode } from '../types'
import { SeededRandom, mapRange, isDarkTheme, hslToString } from '../utils'

type ShapeType = 'square' | 'hexagon' | 'triangle' | 'diamond'

export class FractalRotation implements AnimationMode {
  name = 'fractal-rotation'

  private width = 0
  private height = 0
  private time = 0
  private depth = 8
  private rotationSpeed = 0.5
  private scaleFactor = 0.7
  private baseHue = 200
  private hueShift = 30
  private shapeType: ShapeType = 'square'
  private dark = false
  private progress = 0
  private cx = 0
  private cy = 0

  // 预计算顶点（避免每帧重复创建）
  private shapeVertices: Record<
    ShapeType,
    (cx: number, cy: number, r: number, rot: number) => [number, number][]
  > = {
    square: (cx, cy, r, rot) => {
      const cos = Math.cos(rot)
      const sin = Math.sin(rot)
      const hw = r * 0.707 // half-width at 45°
      return [
        [cx + cos * hw - sin * hw, cy + sin * hw + cos * hw],
        [cx + cos * hw + sin * hw, cy + sin * hw - cos * hw],
        [cx - cos * hw + sin * hw, cy - sin * hw - cos * hw],
        [cx - cos * hw - sin * hw, cy - sin * hw + cos * hw]
      ]
    },
    hexagon: (cx, cy, r, rot) => {
      const verts: [number, number][] = []
      for (let i = 0; i < 6; i++) {
        const a = rot + (Math.PI / 3) * i
        verts.push([cx + Math.cos(a) * r, cy + Math.sin(a) * r])
      }
      return verts
    },
    triangle: (cx, cy, r, rot) => {
      const verts: [number, number][] = []
      for (let i = 0; i < 3; i++) {
        const a = rot + ((Math.PI * 2) / 3) * i - Math.PI / 2
        verts.push([cx + Math.cos(a) * r, cy + Math.sin(a) * r])
      }
      return verts
    },
    diamond: (cx, cy, r, rot) => {
      const cos = Math.cos(rot)
      const sin = Math.sin(rot)
      return [
        [cx + cos * r, cy + sin * r], // 右
        [cx - sin * r * 0.7, cy + cos * r * 0.7], // 下
        [cx - cos * r, cy - sin * r], // 左
        [cx + sin * r * 0.7, cy - cos * r * 0.7] // 上
      ]
    }
  }

  init(
    ctx: CanvasRenderingContext2D,
    width: number,
    height: number,
    seed: number
  ): void {
    this.width = width
    this.height = height
    this.time = 0
    this.dark = isDarkTheme()
    this.progress = 0
    this.cx = width / 2
    this.cy = height / 2

    const rng = new SeededRandom(seed)

    this.depth = rng.int(6, 12)
    this.rotationSpeed = rng.range(0.3, 1.2)
    this.scaleFactor = rng.range(0.65, 0.82)
    this.baseHue = rng.int(0, 360)
    this.hueShift = rng.int(15, 50)
    this.shapeType = rng.pick<ShapeType>([
      'square',
      'hexagon',
      'triangle',
      'diamond'
    ])

    this.draw(ctx)
  }

  update(progress: number, dark: boolean): void {
    this.time += 0.02
    this.dark = dark
    this.progress = progress
  }

  draw(ctx: CanvasRenderingContext2D): void {
    const { width, height, cx, cy, depth, dark, progress } = this

    // 清空背景
    ctx.fillStyle = dark ? '#212121' : '#ffffff'
    ctx.fillRect(0, 0, width, height)

    // 进度影响：旋转速度减慢，形状趋向中心
    const speedFactor = 1 - progress * 0.4
    const rotation = this.time * this.rotationSpeed * speedFactor

    // 从外到内绘制
    const maxRadius = Math.min(width, height) * 0.48 * (1 + progress * 0.1)
    const hueProgOffset = progress * 60

    for (let i = depth; i >= 0; i--) {
      const t = i / depth // 0=最内, 1=最外
      const radius = maxRadius * Math.pow(this.scaleFactor, i)
      if (radius < 1) continue

      // 进度影响：最内层透明度降低（趋向光点）
      const innerFade = i <= 2 ? Math.min(progress * 3, 1) : 0
      const alpha = 0.15 + t * 0.6 + innerFade * 0.2

      // 颜色：外层冷色→内层暖色，进度影响整体偏移
      const hue = (this.baseHue + i * this.hueShift + hueProgOffset) % 360
      const saturation = 50 + (1 - t) * 30
      const lightness = dark
        ? 50 + t * 30 + innerFade * 10 // 暗色：50~80，确保可见
        : 45 + t * 25 + innerFade * 10 // 亮色：45~70

      // 内层小形状用发光
      if (i <= 2 && progress > 0.5 && radius < 10) {
        const glowIntensity = mapRange(progress, 0.5, 1, 0, 0.8)
        ctx.beginPath()
        ctx.arc(cx, cy, radius * 3, 0, Math.PI * 2)
        ctx.fillStyle = hslToString(hue, 80, 70, glowIntensity * 0.3)
        ctx.fill()
      }

      // 绘制形状
      const rot = rotation * (i % 2 === 0 ? 1 : -1) * (i + 1) * 0.3
      const verts = this.shapeVertices[this.shapeType](cx, cy, radius, rot)

      ctx.beginPath()
      ctx.moveTo(verts[0][0], verts[0][1])
      for (let v = 1; v < verts.length; v++) {
        ctx.lineTo(verts[v][0], verts[v][1])
      }
      ctx.closePath()

      ctx.strokeStyle = hslToString(hue, saturation, lightness, alpha)
      ctx.lineWidth = Math.max(1.5 * t, 0.5)
      ctx.stroke()

      // 填充（内层填充更实）
      if (i > depth - 3) {
        ctx.fillStyle = hslToString(hue, saturation, lightness, 0.05)
        ctx.fill()
      }
    }

    // 中心光晕（进度高时更亮）
    if (progress > 0.3) {
      const glowAlpha = mapRange(progress, 0.3, 1, 0.1, 0.5)
      const gradient = ctx.createRadialGradient(
        cx,
        cy,
        0,
        cx,
        cy,
        maxRadius * 0.15
      )
      const centerHue =
        (this.baseHue + depth * this.hueShift + progress * 60) % 360
      gradient.addColorStop(0, hslToString(centerHue, 80, 75, glowAlpha))
      gradient.addColorStop(
        0.5,
        hslToString(centerHue, 70, 65, glowAlpha * 0.3)
      )
      gradient.addColorStop(1, hslToString(centerHue, 60, 55, 0))
      ctx.fillStyle = gradient
      ctx.fillRect(0, 0, width, height)
    }
  }

  resize(width: number, height: number): void {
    this.width = width
    this.height = height
    this.cx = width / 2
    this.cy = height / 2
  }

  destroy(): void {
    // nothing to clean up
  }
}
