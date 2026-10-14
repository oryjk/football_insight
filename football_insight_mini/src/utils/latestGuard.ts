/**
 * 「只认最新一次」的异步防护：每次 begin 捕获自身代次与上下文快照（如 session revision、matchId），
 * 被更新的 begin / invalidate 替代、或上下文变化后，迟到的成功/错误一律丢弃。
 *
 * - isCurrent()：结果是否仍可发布（代次最新且上下文未变）；
 * - owns()：是否仍由本次操作负责收尾（只看代次）——上下文变了但没有新操作接手时，
 *   仍要由它清 loading，避免按钮永久卡在提交中。
 */
export interface LatestToken {
  isCurrent(): boolean
  owns(): boolean
}

export interface LatestGuard {
  begin(): LatestToken
  invalidate(): void
}

export function createLatestGuard(snapshot: () => readonly unknown[] = () => []): LatestGuard {
  let generation = 0

  return {
    begin() {
      const owner = ++generation
      const captured = snapshot()
      const owns = () => generation === owner
      return {
        owns,
        isCurrent: () => owns() && snapshot().every((value, index) => Object.is(value, captured[index])),
      }
    },
    invalidate() {
      generation += 1
    },
  }
}
