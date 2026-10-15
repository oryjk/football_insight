import type {
  TeamBoardComment,
  TeamBoardLikeSummary,
  TeamBoardPost,
  TeamBoardViewResponse,
} from '../../types/teamBoard'
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

export function formatShare(share: number): string {
  return `${(share * 100).toFixed(1)}%`
}

export function metricSuffix(metricLabel: string, value: number): string {
  return `${value} ${metricLabel}`
}

export function formatPublishedAt(value: string): string {
  const date = new Date(value)
  return `${date.getMonth() + 1}/${date.getDate()} ${String(date.getHours()).padStart(2, '0')}:${String(date.getMinutes()).padStart(2, '0')}`
}

export function prependPost(board: TeamBoardViewResponse, post: TeamBoardPost): TeamBoardViewResponse {
  return {
    ...board,
    posts: [post, ...board.posts],
  }
}

export function appendComment(
  board: TeamBoardViewResponse,
  postId: string,
  comment: TeamBoardComment,
): TeamBoardViewResponse {
  return {
    ...board,
    posts: board.posts.map((post) => {
      if (post.post_id !== postId) {
        return post
      }

      return {
        ...post,
        comment_count: post.comment_count + 1,
        comments: [...post.comments, comment],
      }
    }),
  }
}

export function applyLikeSummary(
  board: TeamBoardViewResponse,
  postId: string,
  summary: TeamBoardLikeSummary,
): TeamBoardViewResponse {
  return {
    ...board,
    posts: board.posts.map((post) =>
      post.post_id === postId
        ? {
            ...post,
            liked_by_viewer: summary.liked_by_viewer,
            like_count: summary.like_count,
          }
        : post,
    ),
  }
}
