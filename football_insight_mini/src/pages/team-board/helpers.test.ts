import { describe, expect, test } from 'bun:test'
import { ApiRequestError } from '../../utils/apiError'
import type { TeamBoardComment, TeamBoardPost, TeamBoardViewResponse } from '../../types/teamBoard'
import { appendComment, applyLikeSummary, prependPost, resolveTeamBoardLoadFailure } from './helpers'

const FALLBACK = '战术板加载失败，请稍后重试。'

describe('resolveTeamBoardLoadFailure', () => {
  test('401 无论文案如何都走登录引导', () => {
    expect(resolveTeamBoardLoadFailure(new ApiRequestError('登录已过期', 401), FALLBACK)).toEqual({ kind: 'unauthorized' })
  })

  test('403 保留后端文案继续展示', () => {
    expect(resolveTeamBoardLoadFailure(new ApiRequestError('无权限查看该球队', 403), FALLBACK)).toEqual({
      kind: 'error',
      message: '无权限查看该球队',
    })
  })

  test('500 空响应体时使用兜底文案', () => {
    expect(resolveTeamBoardLoadFailure(new ApiRequestError('', 500), FALLBACK)).toEqual({
      kind: 'error',
      message: FALLBACK,
    })
  })

  test('非 401 且文案包含“未登录”不得当作登录失效（旧实现的误报场景）', () => {
    expect(resolveTeamBoardLoadFailure(new ApiRequestError('服务器开小差（未登录字样误报）', 500), FALLBACK)).toEqual({
      kind: 'error',
      message: '服务器开小差（未登录字样误报）',
    })
  })

  test('普通 Error 不按认证失效处理', () => {
    expect(resolveTeamBoardLoadFailure(new Error('未登录'), FALLBACK)).toEqual({
      kind: 'error',
      message: '未登录',
    })
  })
})

function makePost(postId: string): TeamBoardPost {
  return {
    post_id: postId,
    team_id: 77680,
    insight_kind: 'goals_for',
    insight_label: '进球贡献',
    title: '头部火力',
    commentary: '围绕洞察卡的判断',
    author: { user_id: 'user-1', display_name: '球迷', avatar_url: null },
    snapshot: {
      current_season: 2026,
      round_number: 24,
      team_id: 77680,
      team_name: '成都蓉城',
      rank_no: 2,
      avatar_storage_url: null,
      insight_kind: 'goals_for',
      insight_label: '进球贡献',
      summary_label: '球队进球',
      summary_value: 40,
      sections: [],
    },
    like_count: 2,
    // The total may exceed the number of loaded comments.
    comment_count: 7,
    liked_by_viewer: false,
    created_at: '2026-10-15T08:00:00+08:00',
    comments: [{
      comment_id: 'comment-old',
      post_id: postId,
      author: { user_id: 'user-2', display_name: '另一位球迷', avatar_url: null },
      content: '已有评论',
      created_at: '2026-10-15T08:01:00+08:00',
    }],
  }
}

function makeBoard(): TeamBoardViewResponse {
  return {
    current_season: 2026,
    round_number: 24,
    team: { team_id: 77680, team_name: '成都蓉城', rank_no: 2, avatar_storage_url: null },
    composer_presets: [],
    posts: [makePost('post-1'), makePost('post-2')],
  }
}

function makeComment(postId: string): TeamBoardComment {
  return {
    comment_id: 'comment-new',
    post_id: postId,
    author: { user_id: 'user-3', display_name: '新球迷', avatar_url: null },
    content: '新评论',
    created_at: '2026-10-15T08:02:00+08:00',
  }
}

describe('prependPost', () => {
  test('新帖放到最前，保留旧帖顺序和其他看板数据且不修改输入', () => {
    const board = makeBoard()
    const before = structuredClone(board)
    const created = makePost('post-new')
    const updated = prependPost(board, created)

    expect(updated.posts).toEqual([created, ...before.posts])
    expect(updated.posts[0]).toBe(created)
    expect(updated.posts[1]).toBe(board.posts[0])
    expect(updated.posts[2]).toBe(board.posts[1])
    expect(updated.team).toBe(board.team)
    expect(updated.composer_presets).toBe(board.composer_presets)
    expect(updated === board).toBe(false)
    expect(updated.posts === board.posts).toBe(false)
    expect(board).toEqual(before)
  })

  test('空看板也可插入第一篇观点', () => {
    const board = { ...makeBoard(), posts: [] }
    const created = makePost('post-first')
    expect(prependPost(board, created).posts).toEqual([created])
    expect(board.posts).toEqual([])
  })
})

describe('appendComment', () => {
  test('追加到目标帖末尾并让总数加一，保持其他帖子与原看板不变', () => {
    const board = makeBoard()
    const before = structuredClone(board)
    const created = makeComment('post-2')
    const updated = appendComment(board, 'post-2', created)

    expect(updated.posts[1]).toEqual({
      ...before.posts[1],
      comment_count: 8,
      comments: [...before.posts[1].comments, created],
    })
    expect(updated.posts[1].comments[1]).toBe(created)
    expect(updated.posts[0]).toBe(board.posts[0])
    expect(updated.team).toBe(board.team)
    expect(updated.composer_presets).toBe(board.composer_presets)
    expect(updated === board).toBe(false)
    expect(updated.posts[1] === board.posts[1]).toBe(false)
    expect(updated.posts[1].comments === board.posts[1].comments).toBe(false)
    expect(board).toEqual(before)
  })

  test('没有评论时可追加第一条，计数仍从返回总数递增', () => {
    const board = makeBoard()
    board.posts[0] = { ...board.posts[0], comments: [], comment_count: 0 }
    const created = makeComment('post-1')
    const updated = appendComment(board, 'post-1', created)
    expect(updated.posts[0].comments).toEqual([created])
    expect(updated.posts[0].comment_count).toBe(1)
    expect(board.posts[0].comments).toEqual([])
    expect(board.posts[0].comment_count).toBe(0)
  })

  test('找不到目标帖时不增加其他帖评论数', () => {
    const board = makeBoard()
    const updated = appendComment(board, 'missing', makeComment('missing'))
    expect(updated).toEqual(board)
    expect(updated.posts[0]).toBe(board.posts[0])
    expect(updated.posts[1]).toBe(board.posts[1])
  })
})

describe('applyLikeSummary', () => {
  test('仅更新目标帖的支持标记和数量，保留评论及原看板', () => {
    const board = makeBoard()
    const before = structuredClone(board)
    const updated = applyLikeSummary(board, 'post-2', {
      post_id: 'post-2', liked_by_viewer: true, like_count: 9,
    })

    expect(updated.posts[1]).toEqual({ ...before.posts[1], liked_by_viewer: true, like_count: 9 })
    expect(updated.posts[1].comments).toBe(board.posts[1].comments)
    expect(updated.posts[0]).toBe(board.posts[0])
    expect(updated.team).toBe(board.team)
    expect(updated.composer_presets).toBe(board.composer_presets)
    expect(updated === board).toBe(false)
    expect(updated.posts[1] === board.posts[1]).toBe(false)
    expect(board).toEqual(before)
  })

  test('取消支持时完整采用 false 与零值，以提交的帖子 id 为准', () => {
    const board = makeBoard()
    board.posts[0] = { ...board.posts[0], liked_by_viewer: true }
    const updated = applyLikeSummary(board, 'post-1', {
      post_id: 'different-response-id', liked_by_viewer: false, like_count: 0,
    })
    expect(updated.posts[0]).toEqual({ ...board.posts[0], liked_by_viewer: false, like_count: 0 })
    expect(updated.posts[1]).toBe(board.posts[1])
    expect(board.posts[0].liked_by_viewer).toBe(true)
    expect(board.posts[0].like_count).toBe(2)
  })

  test('找不到目标帖时保留全部支持状态', () => {
    const board = makeBoard()
    const updated = applyLikeSummary(board, 'missing', {
      post_id: 'missing', liked_by_viewer: true, like_count: 1,
    })
    expect(updated).toEqual(board)
    expect(updated.posts[0]).toBe(board.posts[0])
    expect(updated.posts[1]).toBe(board.posts[1])
  })
})
