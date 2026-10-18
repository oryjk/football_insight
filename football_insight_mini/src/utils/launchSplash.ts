/**
 * 品牌启动幕只在「首屏数据迟迟未到」时出现，不为展示而额外等待：
 * - 数据在宽限时间内到达：完全不显示；
 * - 一旦显示：至少停留 MIN_VISIBLE 让动画能读完，数据到达后立即淡出；
 * - 兜底：无论数据状态，超过 MAX 一律收起，启动幕永远不会卡住页面。
 */
export const SPLASH_SHOW_AFTER_MS = 250
export const SPLASH_MIN_VISIBLE_MS = 900
export const SPLASH_MAX_MS = 3500
export const SPLASH_FADE_MS = 350

export interface SplashInput {
  /** 距冷启动首页创建的毫秒数 */
  elapsedMs: number
  /** 首屏数据是否已落定（成功或失败均算） */
  settled: boolean
  /** 启动幕已显示的毫秒数；尚未显示为 null */
  visibleForMs: number | null
}

export type SplashDecision = 'wait' | 'show' | 'hide' | 'skip'

export function decideSplash(input: SplashInput): SplashDecision {
  if (input.visibleForMs === null) {
    if (input.settled || input.elapsedMs >= SPLASH_MAX_MS) return 'skip'
    return input.elapsedMs >= SPLASH_SHOW_AFTER_MS ? 'show' : 'wait'
  }
  if (input.elapsedMs >= SPLASH_MAX_MS) return 'hide'
  if (input.settled && input.visibleForMs >= SPLASH_MIN_VISIBLE_MS) return 'hide'
  return 'wait'
}
