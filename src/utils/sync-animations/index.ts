/**
 * 同步动画引擎
 *
 * 生成艺术驱动的随机等待动画系统。
 * 每次同步从 4 种模式中随机选择一种，并用随机参数初始化，
 * 实现「每次等待都不同」的体验。
 *
 * 使用方式:
 *   const anim = createAnimation(Date.now())
 *   anim.init(ctx, 180, 180, seed)
 *   // 帧循环:
 *   anim.update(progress, isDark)
 *   anim.draw(ctx)
 */

import type { AnimationMode, ModeConstructor } from './types'
import { ParticleFlowField } from './modes/particle-flow'
import { WaveInterference } from './modes/wave-interference'
import { GravityParticles } from './modes/gravity-particles'
import { FractalRotation } from './modes/fractal-rotation'
import { OrbitRings } from './modes/orbit-rings'
import { MorphingShapes } from './modes/morphing-shapes'
import { BubbleSwirl } from './modes/bubble-swirl'

export type { AnimationMode, AnimationConfig } from './types'

export {
  ParticleFlowField,
  WaveInterference,
  GravityParticles,
  FractalRotation,
  OrbitRings,
  MorphingShapes,
  BubbleSwirl
}

/** 所有已注册的模式 */
const MODES: ModeConstructor[] = [
  ParticleFlowField,
  WaveInterference,
  GravityParticles,
  FractalRotation,
  OrbitRings,
  MorphingShapes,
  BubbleSwirl
]

/** 模式中文名称映射 */
export const MODE_NAMES: Record<string, string> = {
  'particle-flow': '粒子流场',
  'wave-interference': '波形干扰',
  'gravity-particles': '引力粒子',
  'fractal-rotation': '几何分形',
  'orbit-rings': '轨道环',
  'morphing-shapes': '形态变形',
  'bubble-swirl': '气泡漩涡'
}

/**
 * 根据种子创建随机动画模式实例。
 * 种子决定：选择哪个模式 + 该模式的所有参数。
 * 相同种子 → 相同动画（可用于重放）
 */
export function createAnimation(seed: number): AnimationMode {
  const modeIndex = Math.floor(seed % MODES.length)
  const Constructor = MODES[modeIndex]
  return new Constructor()
}

/**
 * 获取所有可用模式列表
 */
export function getAvailableModes(): { name: string; label: string }[] {
  return MODES.map((Ctor) => {
    const instance = new Ctor()
    return {
      name: instance.name,
      label: MODE_NAMES[instance.name] || instance.name
    }
  })
}
