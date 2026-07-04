/**
 * 轨道环 (Orbit Rings)
 *
 * 生成艺术灵感: 同心轨道系统
 * 多个旋转圆环，粒子沿轨道运行，随进度从内向外扩散。
 * 每次随机参数: 环数、粒子密度、旋转方向、颜色
 */

import type { AnimationMode } from '../types'
import { SeededRandom, hslToString } from '../utils'

interface RingParticle {
  angle: number
  speed: number
  radius: number // 轨道半径（相对圆心）
  size: number
  hue: number
  alpha: number
}

interface Ring {
  particles: RingParticle[]
  orbitRadius: number
  orbitSpeed: number
  hueBase: number
  hueRange: number
  clockwise: boolean
  strokeAlpha: number
}

export class OrbitRings implements AnimationMode {
  name = 'orbit-rings'

  private width = 0
  private height = 0
  private rings: Ring[] = []
  private time = 0
  private cx = 0
  private cy = 0
  private dark = false
  private progress = 0
  private ringCount = 4
  private particlesPerRing = 24
  private expansionFactor = 0 // 0~1，随进度变化

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
    this.progress = 0
    this.expansionFactor = 0

    const rng = new SeededRandom(seed)
    this.ringCount = rng.int(3, 6)
    this.particlesPerRing = rng.int(16, 36)
    this.rings = []

    const baseHue = rng.range(0, 360)

    for (let r = 0; r < this.ringCount; r++) {
      const t = r / Math.max(this.ringCount - 1, 1)
      const orbitRadius = 15 + t * (Math.min(width, height) * 0.35)
      const hue = (baseHue + r * 40 + rng.range(-10, 10)) % 360
      const particles: RingParticle[] = []

      for (let i = 0; i < this.particlesPerRing; i++) {
        particles.push({
          angle: rng.range(0, Math.PI * 2),
          speed: 0.005 + rng.range(-0.002, 0.005) * (r + 1) * 0.3,
          radius: orbitRadius,
          size: rng.range(1.5, 4),
          hue: hue + rng.range(-15, 15),
          alpha: rng.range(0.5, 1)
        })
      }

      this.rings.push({
        particles,
        orbitRadius,
        orbitSpeed: 0.008 + rng.range(0.003, 0.015) * (1 - t * 0.5),
        hueBase: hue,
        hueRange: 20,
        clockwise: rng.next() > 0.5,
        strokeAlpha: 0.15 + t * 0.25
      })
    }
  }

  update(progress: number, isDarkTheme: boolean): void {
    this.time++
    this.progress = progress
    this.dark = isDarkTheme

    // 随进度从内向外扩散 + 加速
    this.expansionFactor = progress * 0.3

    for (let r = 0; r < this.rings.length; r++) {
      const ring = this.rings[r]
      const dir = ring.clockwise ? 1 : -1

      for (let p = 0; p < ring.particles.length; p++) {
        const particle = ring.particles[p]
        particle.angle += particle.speed * dir * (1 + this.expansionFactor)
      }
    }
  }

  draw(ctx: CanvasRenderingContext2D): void {
    const bg = this.dark ? 'rgba(33,33,33,0.08)' : 'rgba(255,255,255,0.08)'
    ctx.fillStyle = bg
    ctx.fillRect(0, 0, this.width, this.height)

    const l = this.dark ? 70 : 45
    const s = this.dark ? 40 : 55

    // 从外到内绘制
    for (let r = this.rings.length - 1; r >= 0; r--) {
      const ring = this.rings[r]

      // 绘制轨道圆环（半透明）
      ctx.beginPath()
      ctx.arc(this.cx, this.cy, ring.orbitRadius, 0, Math.PI * 2)
      ctx.strokeStyle = hslToString(
        ring.hueBase + r * 10,
        s,
        l + 10,
        ring.strokeAlpha
      )
      ctx.lineWidth = 0.5 + this.progress * 0.5
      ctx.stroke()

      // 绘制粒子
      for (let p = 0; p < ring.particles.length; p++) {
        const particle = ring.particles[p]
        const x = this.cx + Math.cos(particle.angle) * particle.radius
        const y = this.cy + Math.sin(particle.angle) * particle.radius
        const size =
          particle.size * (0.8 + 0.4 * Math.sin(this.time * 0.03 + p))

        ctx.beginPath()
        ctx.arc(x, y, size, 0, Math.PI * 2)
        ctx.fillStyle = hslToString(
          particle.hue + this.time * 0.2,
          s + 10,
          l + 5,
          particle.alpha * (0.6 + 0.4 * Math.sin(this.time * 0.02 + p * 0.5))
        )
        ctx.fill()
      }
    }

    // 中心光晕
    const glowRadius = 3 + this.expansionFactor * 8
    const gradient = ctx.createRadialGradient(
      this.cx,
      this.cy,
      0,
      this.cx,
      this.cy,
      glowRadius
    )
    const hue = this.rings.length > 0 ? this.rings[0].hueBase : 200
    gradient.addColorStop(0, hslToString(hue, 80, 60, 0.6))
    gradient.addColorStop(1, hslToString(hue, 80, 60, 0))
    ctx.fillStyle = gradient
    ctx.beginPath()
    ctx.arc(this.cx, this.cy, glowRadius, 0, Math.PI * 2)
    ctx.fill()
  }

  destroy(): void {
    this.rings = []
  }
}
