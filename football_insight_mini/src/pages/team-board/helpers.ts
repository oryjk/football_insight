import { extractApiErrorMessage, isUnauthorizedError } from '../../utils/apiError'

/** loadBoard 的失败分流：401 走登录引导，其余转成页面展示文案。 */
export type TeamBoardLoadFailure =
  | { kind: 'unauthorized' }
  | { kind: 'error'; message: string }

export function resolveTeamBoardLoadFailure(error: unknown, fallbackMessage: string): TeamBoardLoadFailure {
  if (isUnauthorizedError(error)) {
    return { kind: 'unauthorized' }
  }

  return { kind: 'error', message: extractApiErrorMessage(error, fallbackMessage) }
}
