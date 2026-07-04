/**
 * 气泡漩涡 (Bubble Swirl)
 *
 * 生成艺术灵感: 气泡在漩涡中上升
 * 大小不一的半透明气泡粒子沿螺旋路径运动，
 * 带有呼吸缩放和颜色渐变。
 * 每次随机参数: 气泡数、漩涡密度、大小范围、颜色
 */

import type { AnimationMode } from '../types'
import { SeededRandom, hslToString } from '../utils'

interface Bubble {
  x: number
  y: number
  radius: number
  targetRadius: number
  speed: number
  angle: number
  angleSpeed: number
  spiralRadius: number
  hue: number
  alpha: number
  phase: number // 呼吸相位
}

export class BubbleSwirl implements AnimationMode {
  name = 'bubble-swirl'

  private width = 0
  private height = 0
  private cx = 0
  private cy = 0
  private bubbles: Bubble[] = []
  private time = 0
  private dark = false
  private bubbleCount = 30
  private maxSpiralRadius = 0

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
    this.bubbleCount = rng.int(20, 50)
    this.maxSpiralRadius = Math.min(width, height) * 0.4
    const baseHue = rng.range(0, 360)
    const hueRange = rng.range(30, 90)
    this.bubbles = []

    for (let i = 0; i < this.bubbleCount; i++) {
      const t = i / this.bubbleCount
      const spiralR = rng.range(5, this.maxSpiralRadius)
      const angle = rng.range(0, Math.PI * 2)

      this.bubbles.push({
        x: this.cx + Math.cos(angle) * spiralR,
        y: this.cy + Math.sin(angle) * spiralR,
        radius: rng.range(2, 6),
        targetRadius: rng.range(2, 8),
        speed: rng.range(0.3, 1.2),
        angle: angle,
        angleSpeed: rng.range(0.003, 0.015) * (rng.next() > 0.5 ? 1 : -1),
        spiralRadius: spiralR,
        hue: (baseHue + t * hueRange + rng.range(-10, 10)) % 360,
        alpha: rng.range(0.3, 0.7),
        phase: rng.range(0, Math.PI * 2)
      })
    }
  }

  update(progress: number, isDarkTheme: boolean): void {
    this.time++
    this.dark = isDarkTheme

    // 随进度增加速度
    const speedMultiplier = 1 + progress * 0.5

    for (let i = 0; i < this.bubbles.length; i++) {
      const b = this.bubbles[i]

      // 漩涡旋转
      b.angle += b.angleSpeed * speedMultiplier

      // 呼吸效果
      const breathe = 0.7 + 0.3 * Math.sin(this.time * 0.04 + b.phase)
      const currentRadius = b.targetRadius * breathe

      // 速度平滑过渡
      const targetR = b.spiralRadius + progress * this.maxSpiralRadius * 0.15
      b.spiralRadius += (targetR - b.spiralRadius) * 0.02

      // 位置
      b.x = this.cx + Math.cos(b.angle) * b.spiralRadius
      b.y = this.cy + Math.sin(b.angle) * b.spiralRadius

      // 更新显示的半径
      b.radius += (currentRadius - b.radius) * 0.1
    }
  }

  draw(ctx: CanvasRenderingContext2D): void {
    // 清空背景 + 轻微拖尾
    if (this.time <= 1) {
      ctx.fillStyle = this.dark ? '#212121' : '#ffffff'
      ctx.fillRect(0, 0, this.width, this.height)
    } else {
      ctx.fillStyle = this.dark
        ? 'rgba(33,33,33,0.05)'
        : 'rgba(255,255,255,0.05)'
      ctx.fillRect(0, 0, this.width, this.height)
    }

    const l = this.dark ? 75 : 45
    const s = this.dark ? 35 : 50

    // 绘制所有气泡
    for (let i = 0; i < this.bubbles.length; i++) {
      const b = this.bubbles[i]
      const alpha =
        b.alpha * (0.5 + 0.5 * Math.sin(this.time * 0.03 + b.phase * 0.5))

      // 外发光
      const glowRadius = b.radius * 2.5
      const gradient = ctx.createRadialGradient(
        b.x,
        b.y,
        0,
        b.x,
        b.y,
        glowRadius
      )
      gradient.addColorStop(0, hslToString(b.hue, s + 20, l + 10, alpha * 0.3))
      gradient.addColorStop(1, hslToString(b.hue, s, l, 0))
      ctx.fillStyle = gradient
      ctx.beginPath()
      ctx.arc(b.x, b.y, glowRadius, 0, Math.PI * 2)
      ctx.fill()

      // 气泡主体
      ctx.beginPath()
      ctx.arc(b.x, b.y, b.radius, 0, Math.PI * 2)
      ctx.fillStyle = hslToString(b.hue, s, l, alpha * 0.4)
      ctx.fill()

      // 气泡边缘
      ctx.strokeStyle = hslToString(b.hue + 10, s + 10, l + 10, alpha * 0.6)
      ctx.lineWidth = 0.5 + b.radius * 0.15
      ctx.stroke()

      // 高光（左上角小光点）
      const highlightX = b.x - b.radius * 0.3
      const highlightY = b.y - b.radius * 0.3
      const highlightR = b.radius * 0.25
      ctx.beginPath()
      ctx.arc(highlightX, highlightY, highlightR, 0, Math.PI * 2)
      ctx.fillStyle = hslToString(0, 0, 100, alpha * 0.5)
      ctx.fill()
    }

    // 中心微弱漩涡指示
    ctx.beginPath()
    ctx.arc(this.cx, this.cy, 2, 0, Math.PI * 2)
    ctx.fillStyle = this.dark ? 'rgba(255,255,255,0.15)' : 'rgba(0,0,0,0.1)'
    ctx.fill()
  }

  destroy(): void {
    this.bubbles = []
  }
}
