/**
 * 粒子流场 (Particle Flow Field)
 *
 * 生成艺术灵感: Perlin Noise Flow Field
 * 粒子在噪声驱动流场中运动，形成有机流动线条。
 * 每次同步随机参数: 粒子数、噪声尺度、速度、颜色、轨迹长度
 */

import type { AnimationMode } from '../types'
import { SeededRandom, fbm, isDarkTheme, hslToString } from '../utils'

interface Particle {
  x: number
  y: number
  vx: number
  vy: number
  trail: { x: number; y: number }[]
  maxTrail: number
  hue: number
  alpha: number
}

export class ParticleFlowField implements AnimationMode {
  name = 'particle-flow'

  private width = 0
  private height = 0
  private particles: Particle[] = []
  private time = 0
  private noiseScale = 0.005
  private speed = 1.5
  private particleCount = 50
  private trailLength = 20
  private flowStrength = 4
  private baseHue = 210
  private hueRange = 60
  private dark = false

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

    const rng = new SeededRandom(seed)

    // 随机参数（一次同步内固定）
    this.particleCount = rng.int(30, 70)
    this.noiseScale = rng.range(0.003, 0.01)
    this.speed = rng.range(0.8, 2.0)
    this.trailLength = rng.int(10, 35)
    this.flowStrength = rng.range(2, 6)
    this.baseHue = rng.int(180, 330)
    this.hueRange = rng.int(30, 90)

    this.particles = []
    for (let i = 0; i < this.particleCount; i++) {
      this.particles.push(this.createParticle(rng))
    }

    // 清空 canvas 残留内容
    ctx.fillStyle = this.dark ? '#212121' : '#ffffff'
    ctx.fillRect(0, 0, width, height)

    // 初始绘制一帧
    this.draw(ctx)
  }

  private createParticle(rng: SeededRandom): Particle {
    return {
      x: rng.range(0, this.width),
      y: rng.range(0, this.height),
      vx: 0,
      vy: 0,
      trail: [],
      maxTrail: this.trailLength,
      hue: this.baseHue + rng.range(-this.hueRange, this.hueRange),
      alpha: rng.range(0.3, 0.9)
    }
  }

  update(progress: number, dark: boolean): void {
    this.time += 0.008
    this.dark = dark

    // 进度影响：越接近完成，速度越慢，粒子越聚集
    const progressFactor = 1 - progress * 0.5
    const currentSpeed = this.speed * progressFactor

    // 进度影响：颜色向主题色收敛
    const convergeProgress = Math.min(progress * 2, 1)

    for (const p of this.particles) {
      // 通过 Perlin Noise 计算流向
      const noiseVal = fbm(
        p.x * this.noiseScale + this.time * 0.3,
        p.y * this.noiseScale + this.time * 0.3,
        2
      )
      const angle = noiseVal * Math.PI * 4

      p.vx = Math.cos(angle) * currentSpeed * this.flowStrength
      p.vy = Math.sin(angle) * currentSpeed * this.flowStrength

      p.x += p.vx
      p.y += p.vy

      // 环绕边界
      if (p.x < -10) p.x = this.width + 10
      if (p.x > this.width + 10) p.x = -10
      if (p.y < -10) p.y = this.height + 10
      if (p.y > this.height + 10) p.y = -10

      // 记录轨迹
      p.trail.push({ x: p.x, y: p.y })
      if (p.trail.length > p.maxTrail) {
        p.trail.shift()
      }

      // 颜色向主题色收敛
      if (convergeProgress > 0) {
        p.hue = p.hue + (this.baseHue - p.hue) * convergeProgress * 0.02
      }
    }
  }

  draw(ctx: CanvasRenderingContext2D): void {
    // 半透明背景实现拖尾效果
    ctx.fillStyle = this.dark
      ? 'rgba(33, 33, 33, 0.15)'
      : 'rgba(255, 255, 255, 0.12)'
    ctx.fillRect(0, 0, this.width, this.height)

    for (const p of this.particles) {
      if (p.trail.length < 2) continue

      // 轨迹发光效果
      for (let i = 1; i < p.trail.length; i++) {
        const t = i / p.trail.length
        const alpha = t * p.alpha * 0.6
        const width = t * 2.5 + 0.5

        ctx.beginPath()
        ctx.moveTo(p.trail[i - 1].x, p.trail[i - 1].y)
        ctx.lineTo(p.trail[i].x, p.trail[i].y)
        ctx.strokeStyle = hslToString(p.hue, 70, 55 + t * 20, alpha)
        ctx.lineWidth = width
        ctx.lineCap = 'round'
        ctx.stroke()
      }

      // 粒子头部光晕
      ctx.beginPath()
      ctx.arc(p.x, p.y, 2.5, 0, Math.PI * 2)
      ctx.fillStyle = hslToString(p.hue, 80, 70, 0.8)
      ctx.fill()

      // 发光外圈
      ctx.beginPath()
      ctx.arc(p.x, p.y, 5, 0, Math.PI * 2)
      ctx.fillStyle = hslToString(p.hue, 80, 70, 0.15)
      ctx.fill()
    }
  }

  resize(width: number, height: number): void {
    const scaleX = width / this.width
    const scaleY = height / this.height
    this.width = width
    this.height = height
    for (const p of this.particles) {
      p.x *= scaleX
      p.y *= scaleY
    }
  }

  destroy(): void {
    this.particles = []
  }
}
