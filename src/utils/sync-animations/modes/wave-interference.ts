/**
 * 波形干扰 (Wave Interference)
 *
 * 生成艺术灵感: 叠加正弦波 ／ 油彩表面
 * 多层正弦波叠加形成复杂的干涉图案，颜色随波形值映射。
 * 每次随机参数: 波数、频率、振幅、颜色映射
 */

import type { AnimationMode } from '../types'
import { SeededRandom, mapRange, isDarkTheme, hslToString } from '../utils'

interface Wave {
  frequencyX: number
  frequencyY: number
  speed: number
  amplitude: number
  phaseX: number
  phaseY: number
  weight: number
}

export class WaveInterference implements AnimationMode {
  name = 'wave-interference'

  private width = 0
  private height = 0
  private waves: Wave[] = []
  private time = 0
  private palette: number[] = [] // HSL hues
  private dark = false
  private progress = 0
  private imageData: ImageData | null = null
  private usePixelMode = false

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

    const rng = new SeededRandom(seed)
    const waveCount = rng.int(3, 6)

    this.waves = []
    for (let i = 0; i < waveCount; i++) {
      this.waves.push({
        frequencyX: rng.range(0.01, 0.04),
        frequencyY: rng.range(0.01, 0.04),
        speed: rng.range(0.5, 2.0),
        amplitude: rng.range(0.5, 1.5),
        phaseX: rng.range(0, Math.PI * 2),
        phaseY: rng.range(0, Math.PI * 2),
        weight: rng.range(0.5, 1.5)
      })
    }

    // 颜色调色板
    const paletteSize = rng.int(3, 5)
    this.palette = []
    const baseHue = rng.int(0, 360)
    for (let i = 0; i < paletteSize; i++) {
      this.palette.push(
        (baseHue + i * (360 / paletteSize) + rng.range(-20, 20)) % 360
      )
    }

    this.usePixelMode = width * height < 400 * 400 // 小画布用像素模式更快

    // 预分配 ImageData
    if (this.usePixelMode) {
      this.imageData = ctx.createImageData(width, height)
    }

    this.draw(ctx)
  }

  update(progress: number, dark: boolean): void {
    this.time += 0.02
    this.dark = dark
    this.progress = progress
  }

  draw(ctx: CanvasRenderingContext2D): void {
    if (this.usePixelMode && this.imageData) {
      // 逐像素模式 — 高质量，适合小画布
      this.drawPixelMode(ctx)
    } else {
      // Canvas 2D 绘制模式 — 性能优先，适合大画布
      this.drawCanvasMode(ctx)
    }
  }

  private drawPixelMode(ctx: CanvasRenderingContext2D): void {
    const { width, height, waves, time, palette, dark, progress } = this
    const data = this.imageData!.data
    const bgLight = dark ? 25 : 250

    // 进度只影响视觉属性（对比度/饱和度），不影响波相位
    const contrastBoost = 1 + progress * 0.5

    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        let value = 0
        for (const w of waves) {
          // 相位只用纯时间累积，不受 progress 影响 — 避免跳帧
          const v =
            Math.sin(
              x * w.frequencyX + time * w.speed + w.phaseX
            ) *
            Math.cos(
              y * w.frequencyY + time * w.speed * 0.7 + w.phaseY
            )
          value += v * w.amplitude * w.weight
        }

        // 归一化到 [0, 1]
        value = (value + 3) / 6 // 假设最大振幅和 ≈ 3
        value = Math.pow(Math.max(0, Math.min(1, value)), contrastBoost)

        // 颜色映射
        const hueIndex = Math.floor(value * (palette.length - 1))
        const hueLerp = (value * (palette.length - 1)) % 1
        const h1 = palette[Math.min(hueIndex, palette.length - 1)]
        const h2 = palette[Math.min(hueIndex + 1, palette.length - 1)]
        let hue = h1 + (h2 - h1) * hueLerp
        if (hue > 360) hue -= 360
        if (hue < 0) hue += 360

        // 进度影响：颜色饱和度逐渐降低，趋向柔和
        const saturation = 55 + (1 - progress) * 25
        const lightness = dark
          ? mapRange(value, 0, 1, 15, 60)
          : mapRange(value, 0, 1, 40, 80)

        const idx = (y * width + x) * 4
        const rgb = this.hslToRgb(hue, saturation, lightness)

        // 背景混合
        const alpha = 0.85
        data[idx] = Math.round(rgb.r * alpha + bgLight * (1 - alpha))
        data[idx + 1] = Math.round(rgb.g * alpha + bgLight * (1 - alpha))
        data[idx + 2] = Math.round(rgb.b * alpha + bgLight * (1 - alpha))
        data[idx + 3] = 255
      }
    }

    ctx.putImageData(this.imageData!, 0, 0)
  }

  private drawCanvasMode(ctx: CanvasRenderingContext2D): void {
    const { width, height, waves, time, palette, dark, progress } = this

    // 清空
    ctx.fillStyle = dark ? '#212121' : '#ffffff'
    ctx.fillRect(0, 0, width, height)

    const step = 4 // 采样步长（性能优化）

    for (let y = 0; y < height; y += step) {
      for (let x = 0; x < width; x += step) {
        let value = 0
        for (const w of waves) {
          // 相位只用纯时间累积，不受 progress 影响
          const v =
            Math.sin(
              x * w.frequencyX + time * w.speed + w.phaseX
            ) *
            Math.cos(
              y * w.frequencyY + time * w.speed * 0.7 + w.phaseY
            )
          value += v * w.amplitude * w.weight
        }

        value = (value + 3) / 6
        value = Math.max(0, Math.min(1, value))

        const hueIndex = Math.floor(value * (palette.length - 1))
        const hueLerp = (value * (palette.length - 1)) % 1
        const h1 = palette[Math.min(hueIndex, palette.length - 1)]
        const h2 = palette[Math.min(hueIndex + 1, palette.length - 1)]
        let hue = h1 + (h2 - h1) * hueLerp
        if (hue > 360) hue -= 360
        if (hue < 0) hue += 360

        const saturation = 55 + (1 - progress) * 25
        const lightness = dark
          ? mapRange(value, 0, 1, 20, 55)
          : mapRange(value, 0, 1, 45, 75)

        ctx.fillStyle = hslToString(hue % 360, saturation, lightness, 0.85)
        ctx.fillRect(x, y, step, step)
      }
    }
  }

  private hslToRgb(
    h: number,
    s: number,
    l: number
  ): { r: number; g: number; b: number } {
    h /= 360
    s /= 100
    l /= 100
    let r: number, g: number, b: number

    if (s === 0) {
      r = g = b = l
    } else {
      const hue2rgb = (p: number, q: number, t: number): number => {
        if (t < 0) t += 1
        if (t > 1) t -= 1
        if (t < 1 / 6) return p + (q - p) * 6 * t
        if (t < 1 / 2) return q
        if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6
        return p
      }
      const q = l < 0.5 ? l * (1 + s) : l + s - l * s
      const p = 2 * l - q
      r = hue2rgb(p, q, h + 1 / 3)
      g = hue2rgb(p, q, h)
      b = hue2rgb(p, q, h - 1 / 3)
    }
    return {
      r: Math.round(r * 255),
      g: Math.round(g * 255),
      b: Math.round(b * 255)
    }
  }

  resize(width: number, height: number): void {
    this.width = width
    this.height = height
    this.usePixelMode = width * height < 400 * 400
    if (this.usePixelMode) {
      // ImageData 会在下次 draw 时重建
      this.imageData = null
    }
  }

  destroy(): void {
    this.imageData = null
    this.waves = []
  }
}
