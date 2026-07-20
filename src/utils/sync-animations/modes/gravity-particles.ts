/**
 * 引力粒子 (Gravity Particles)
 *
 * 生成艺术灵感: N-Body 简易引力模拟
 * 粒子之间互相吸引/排斥，形成动态的星云/星系效果。
 * 每次随机参数: 粒子数、引力常数、初始分布、颜色
 */

import type { AnimationMode } from '../types'
import { SeededRandom, dist, isDarkTheme, hslToString } from '../utils'

interface GParticle {
  x: number
  y: number
  vx: number
  vy: number
  mass: number
  hue: number
  radius: number
  trail: { x: number; y: number }[]
  maxTrail: number
}

export class GravityParticles implements AnimationMode {
  name = 'gravity-particles'

  private width = 0
  private height = 0
  private particles: GParticle[] = []
  private time = 0
  private gravityConstant = 0.3
  private damping = 0.98
  private minDist = 5
  private softening = 20       // 软化参数，消除引力奇点
  private repulsion = 3        // 短程排斥强度
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
    const count = rng.int(15, 40)

    this.gravityConstant = rng.range(0.1, 0.6)
    this.damping = rng.range(0.99, 0.999)    // 弱阻尼，保留轨道动能
    this.minDist = rng.range(5, 15)
    this.softening = rng.range(12, 30)       // 软化半径

    const baseHue = rng.int(0, 360)
    const hueRange = rng.int(40, 120)
    const distMode = rng.int(0, 2) // 0=随机, 1=环形, 2=双团

    this.particles = []
    for (let i = 0; i < count; i++) {
      let x: number, y: number

      switch (distMode) {
        case 0: // 随机分布
          x = rng.range(width * 0.1, width * 0.9)
          y = rng.range(height * 0.1, height * 0.9)
          break
        case 1: {
          // 环形分布
          const angle = rng.range(0, Math.PI * 2)
          const radius = rng.range(width * 0.15, width * 0.4)
          x = width / 2 + Math.cos(angle) * radius
          y = height / 2 + Math.sin(angle) * radius
          break
        }
        case 2: {
          // 双团分布
          const cluster = rng.next() > 0.5 ? 0 : 1
          const cx = cluster === 0 ? width * 0.3 : width * 0.7
          const cy = height / 2
          x = cx + rng.range(-width * 0.1, width * 0.1)
          y = cy + rng.range(-height * 0.1, height * 0.1)
          break
        }
        default:
          x = rng.range(0, width)
          y = rng.range(0, height)
      }

      // 初始速度：随机方向 + 切向轨道分量（避免直接坠落）
      let vx = rng.range(-1, 1)
      let vy = rng.range(-1, 1)
      if (distMode === 1 || distMode === 2) {
        // 环/双团模式：添加切向速度，产生轨道运动
        const cx = distMode === 1 ? width / 2 : (width * 0.5)
        const cy = height / 2
        const dx = x - cx
        const dy = y - cy
        const d = Math.sqrt(dx * dx + dy * dy) || 1
        const orbitSpeed = rng.range(0.3, 1.2) * Math.sqrt(this.gravityConstant * 1.5)
        // 切向 = (-dy, dx) 归一化 × 轨道速度
        vx += (-dy / d) * orbitSpeed
        vy += (dx / d) * orbitSpeed
      }

      this.particles.push({
        x,
        y,
        vx,
        vy,
        mass: rng.range(0.5, 2),
        hue: baseHue + rng.range(-hueRange, hueRange),
        radius: rng.range(2, 5),
        trail: [],
        maxTrail: rng.int(8, 20)
      })
    }

    // 清空 canvas 残留内容
    ctx.fillStyle = this.dark ? '#212121' : '#ffffff'
    ctx.fillRect(0, 0, width, height)

    this.draw(ctx)
  }

  update(_progress: number, dark: boolean): void {
    this.dark = dark
    this.time += 0.016

    // 引力常量恒定 — 不受 progress 影响（避免轨迹突变）
    const currentG = this.gravityConstant

    const dt = 0.5

    // 计算引力 + 短程排斥（软化力，防止奇点 + 死区冻结）
    for (let i = 0; i < this.particles.length; i++) {
      let fx = 0
      let fy = 0
      const pi = this.particles[i]

      for (let j = 0; j < this.particles.length; j++) {
        if (i === j) continue
        const pj = this.particles[j]
        const d = dist(pi.x, pi.y, pj.x, pj.y)
        if (d < 0.01) continue // 极近距离保护，几乎不会触发

        const dx = pj.x - pi.x
        const dy = pj.y - pi.y

        // — 软化引力：F = G·m₁·m₂ / (d² + ε²)，消除奇点 —
        const gravityForce = (currentG * pi.mass * pj.mass) / (d * d + this.softening * this.softening)
        fx += (gravityForce * dx) / d
        fy += (gravityForce * dy) / d

        // — 短程排斥：防止粒子重叠堆积（类 Pauli 排斥） —
        if (d < this.minDist * this.repulsion) {
          const repelStrength = currentG * pi.mass * pj.mass * 3
          const repForce = repelStrength / (d * d + this.softening * 0.3)
          fx -= (repForce * dx) / d
          fy -= (repForce * dy) / d
        }
      }

      pi.vx += fx * dt
      pi.vy += fy * dt

      // 弱阻尼 — 保留轨道动能
      pi.vx *= this.damping
      pi.vy *= this.damping

      pi.x += pi.vx
      pi.y += pi.vy

      // 动能过低时注能 — 防止系统"热寂"
      const kineticEnergy = pi.vx * pi.vx + pi.vy * pi.vy
      if (kineticEnergy < 0.005 && this.particles.length > 1) {
        pi.vx += (Math.random() - 0.5) * 0.15
        pi.vy += (Math.random() - 0.5) * 0.15
      }

      // 边界反弹（带能量损失）
      if (pi.x < 2 || pi.x > this.width - 2) {
        pi.vx *= -0.7
        pi.x = Math.max(2, Math.min(this.width - 2, pi.x))
      }
      if (pi.y < 2 || pi.y > this.height - 2) {
        pi.vy *= -0.7
        pi.y = Math.max(2, Math.min(this.height - 2, pi.y))
      }

      // 轨迹
      pi.trail.push({ x: pi.x, y: pi.y })
      if (pi.trail.length > pi.maxTrail) {
        pi.trail.shift()
      }
    }
  }

  draw(ctx: CanvasRenderingContext2D): void {
    // 半透明背景拖尾
    ctx.fillStyle = this.dark
      ? 'rgba(33, 33, 33, 0.12)'
      : 'rgba(255, 255, 255, 0.1)'
    ctx.fillRect(0, 0, this.width, this.height)

    for (const p of this.particles) {
      // 轨迹
      if (p.trail.length > 1) {
        for (let i = 1; i < p.trail.length; i++) {
          const t = i / p.trail.length
          const alpha = t * 0.4

          ctx.beginPath()
          ctx.moveTo(p.trail[i - 1].x, p.trail[i - 1].y)
          ctx.lineTo(p.trail[i].x, p.trail[i].y)
          ctx.strokeStyle = hslToString(p.hue % 360, 60, 55, alpha)
          ctx.lineWidth = Math.max(p.radius * t * 0.6, 0.3)
          ctx.stroke()
        }
      }

      // 粒子本体（发光）
      const gradient = ctx.createRadialGradient(
        p.x,
        p.y,
        0,
        p.x,
        p.y,
        p.radius * 3
      )
      gradient.addColorStop(0, hslToString(p.hue % 360, 80, 75, 0.9))
      gradient.addColorStop(0.3, hslToString(p.hue % 360, 70, 60, 0.4))
      gradient.addColorStop(1, hslToString(p.hue % 360, 70, 60, 0))

      ctx.beginPath()
      ctx.arc(p.x, p.y, p.radius * 3, 0, Math.PI * 2)
      ctx.fillStyle = gradient
      ctx.fill()

      // 粒子核心
      ctx.beginPath()
      ctx.arc(p.x, p.y, p.radius, 0, Math.PI * 2)
      ctx.fillStyle = hslToString(p.hue % 360, 80, 80, 0.95)
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
