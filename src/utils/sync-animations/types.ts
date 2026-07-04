/**
 * 同步动画模式接口
 *
 * 每个模式都是一个独立的状态机，由外部驱动帧循环。
 * 设计原则：简单规则 → 复杂涌现
 */
export interface AnimationMode {
  /** 模式唯一标识 */
  name: string

  /** 初始化（接收画布、尺寸、种子）
   *  seed 保证一次同步内确定性随机，每次同步不同 */
  init(
    ctx: CanvasRenderingContext2D,
    width: number,
    height: number,
    seed: number
  ): void

  /** 每帧更新（progress: 0~1） */
  update(progress: number, isDarkTheme: boolean): void

  /** 每帧绘制 */
  draw(ctx: CanvasRenderingContext2D): void

  /** 窗口尺寸变化 */
  resize?(width: number, height: number): void

  /** 清理资源 */
  destroy(): void
}

/** 模式构造器 */
export type ModeConstructor = new () => AnimationMode

/** 动画配置，由外部传入 */
export interface AnimationConfig {
  /** 画布宽度 */
  width: number
  /** 画布高度 */
  height: number
  /** 随机种子（0~1e6） */
  seed: number
  /** 当前进度（0~1） */
  progress: number
  /** 是否暗色主题 */
  isDarkTheme: boolean
}
