import { describe, expect, test } from 'bun:test'
import { ref } from 'vue'
import type { CurrentUser } from '../../types/auth'
import { ApiRequestError } from '../../utils/apiError'
import { useTelegraphCommentPage } from './useTelegraphCommentPage'
import type { EntrySessionPort } from './useTelegraphEntryPage'

function makeUser(overrides: Partial<CurrentUser> = {}): CurrentUser {
  return {
    id: 'user-1',
    account_identifier: '13800138000',
    display_name: '13800138000',
    avatar_url: null,
    profile_completed: false,
    membership_tier: 'free',
    membership_expires_at: null,
    has_wechat_binding: false,
    official_wechat_bound: false,
    official_wechat_subscribed: false,
    invite_code: null,
    created_at: '2026-11-17T00:00:00Z',
    ...overrides,
  }
}

function makeRawTopic() {
  return {
    id: 'topic_1',
    team_id: 77680,
    title: '每日电报',
    summary: '摘要',
    paragraphs: ['正文'],
    published_at: '2026-11-17T07:00:00Z',
    heat: 10,
    comments_enabled: true,
    comments_count: 3,
    sources: [{ title: '原文', publisher: '直播吧', url: 'https://news.example/1', published_at: '2026-11-17T06:50:00Z' }],
    audio: { status: 'pending' },
    entries: [
      {
        id: 'entry_1',
        category: 'domestic',
        title: '斯坦丘有望加盟',
        text: '大连英博正在推进签约。',
        published_at: '2026-11-17T06:57:00Z',
        sources: [{ title: '原文', publisher: '直播吧', url: 'https://news.example/1', published_at: '2026-11-17T06:57:00Z' }],
        likes_count: 1,
        dislikes_count: 0,
        comments_count: 3,
        liked_by_me: false,
        disliked_by_me: false,
      },
    ],
  }
}

function makeSession(initialUser: CurrentUser | null, loggedIn = Boolean(initialUser)): EntrySessionPort {
  const currentUser = ref<CurrentUser | null>(initialUser)
  const hasAccessToken = ref(loggedIn)
  const revision = ref(1)
  return {
    currentUser,
    hasAccessToken,
    revision,
    ensureSessionReady: async () => currentUser.value,
    updateProfile: async ({ display_name, avatar_data_url }) => {
      const next = makeUser({
        ...(currentUser.value ?? {}),
        display_name,
        avatar_url: avatar_data_url ?? currentUser.value?.avatar_url ?? 'https://cdn.example/a.jpg',
        profile_completed: true,
      })
      currentUser.value = next
      return next
    },
    invalidateSession: () => {
      hasAccessToken.value = false
      currentUser.value = null
      revision.value += 1
    },
  }
}

function rawComment(id: string, content: string, replyTo: { comment_id: string; author_display_name: string } | null = null, isMine = false) {
  return {
    id,
    topic_id: 'topic_1',
    entry_id: 'entry_1',
    author: { display_name: '川足球迷', avatar_url: 'https://cdn.example/a.jpg' },
    reply_to: replyTo,
    content,
    likes_count: 2,
    liked_by_me: false,
    replies_count: 0,
    is_mine: isMine,
    created_at: '2026-11-17T07:00:00Z',
  }
}

describe('useTelegraphCommentPage', () => {
  test('loads ancestors, focal comment and direct child replies cleanly', async () => {
    const openedUrls: string[] = []
    const page = useTelegraphCommentPage({
      topicId: () => 'topic_1',
      entryId: () => 'entry_1',
      commentId: () => 'r1',
      session: makeSession(null, false),
      miniReview: { shouldRestrictFeatures: ref(false) },
      api: {
        getTopicDetail: async () => makeRawTopic(),
        getCommentThread: async () => ({
          comments_enabled: true,
          entry_comments_count: 3,
          ancestors: [rawComment('c1', '根评论')],
          comment: { ...rawComment('r1', '直接回复', { comment_id: 'c1', author_display_name: '川足球迷' }), replies_count: 1 },
          replies_total_count: 1,
          items: [rawComment('r2', '次级回复', { comment_id: 'r1', author_display_name: '川足球迷' })],
          next_cursor: null,
          has_more: false,
        }),
      },
      ui: {
        redirectToLogin: () => {},
        toast: () => {},
        openComment: url => openedUrls.push(url),
        openEntry: url => openedUrls.push(url),
      },
    })

    await page.loadInitial()
    expect(page.errorMessage.value).toBe('')
    expect(page.ancestors.value.map(c => c.id)).toEqual(['c1'])
    expect(page.focalComment.value?.id).toBe('r1')
    expect(page.replies.value.map(c => c.id)).toEqual(['r2'])
    expect(page.repliesTotalCount.value).toBe(1)

    page.openEntry()
    page.openComment(page.ancestors.value[0]!, false)
    page.openComment(page.replies.value[0]!, true)
    expect(openedUrls).toEqual([
      '/pages/telegraph-entry/index?topicId=topic_1&entryId=entry_1',
      '/pages/telegraph-entry/comment?topicId=topic_1&entryId=entry_1&commentId=c1',
      '/pages/telegraph-entry/comment?topicId=topic_1&entryId=entry_1&commentId=r2&focus=1',
    ])
  })

  test('falls back to assembling thread via legacy comments endpoint when /thread returns route 404', async () => {
    const page = useTelegraphCommentPage({
      topicId: () => 'topic_1',
      entryId: () => 'entry_1',
      commentId: () => 'r1',
      session: makeSession(null, false),
      miniReview: { shouldRestrictFeatures: ref(false) },
      api: {
        getTopicDetail: async () => makeRawTopic(),
        getCommentThread: async () => {
          throw new ApiRequestError('Not Found', 404, false, '')
        },
        getComments: async () => ({
          comments_enabled: true,
          total_count: 4,
          has_more: false,
          next_cursor: null,
          items: [
            rawComment('other_reply', '别人的二级回复', { comment_id: 'other_root', author_display_name: '别人' }),
            rawComment('r2', '目标评论的直接子回复', { comment_id: 'r1', author_display_name: '川足球迷' }),
            rawComment('r1', '目标评论', { comment_id: 'c1', author_display_name: '根作者' }),
            rawComment('c1', '根评论'),
          ],
        }),
      },
      ui: { redirectToLogin: () => {}, toast: () => {} },
    })

    await page.loadInitial()
    expect(page.errorMessage.value).toBe('')
    expect(page.ancestors.value.map(c => c.id)).toEqual(['c1'])
    expect(page.focalComment.value?.id).toBe('r1')
    expect(page.replies.value.map(c => c.id)).toEqual(['r2'])
  })

  test('submitReply replies to focalComment, deleteOwnComment removes reply or navigates back for focal', async () => {
    const backedTo: string[] = []
    let createdPayload: unknown = null
    const page = useTelegraphCommentPage({
      topicId: () => 'topic_1',
      entryId: () => 'entry_1',
      commentId: () => 'c1',
      session: makeSession(makeUser({ display_name: '川足铁杆', avatar_url: 'https://cdn.example/a.jpg', profile_completed: true })),
      miniReview: { shouldRestrictFeatures: ref(false) },
      api: {
        getTopicDetail: async () => makeRawTopic(),
        getCommentThread: async () => ({
          comments_enabled: true,
          entry_comments_count: 1,
          ancestors: [],
          comment: rawComment('c1', '我的根评论', null, true),
          replies_total_count: 0,
          items: [],
          next_cursor: null,
          has_more: false,
        }),
        createComment: async (_teamId, _topicId, _entryId, payload) => {
          createdPayload = payload
          return {
            comments_count: 2,
            comment: rawComment('r1', payload.content, { comment_id: 'c1', author_display_name: '川足铁杆' }, true),
          }
        },
        deleteComment: async (_t, _tid, _eid, cid) => ({ comments_count: cid === 'r1' ? 1 : 0 }),
        setCommentLike: async () => ({ comment_id: 'c1', likes_count: 3, liked_by_me: true }),
      },
      ui: {
        redirectToLogin: () => {},
        toast: () => {},
        navigateBack: fallback => backedTo.push(fallback),
      },
    })

    await page.loadInitial()
    await page.toggleCommentLike(page.focalComment.value!)
    expect(page.focalComment.value?.likesCount).toBe(3)
    expect(page.focalComment.value?.likedByMe).toBe(true)

    page.draftContent.value = '继续讨论这条评论'
    await page.submitReply()
    expect(createdPayload).toEqual({ content: '继续讨论这条评论', reply_to_comment_id: 'c1' })
    expect(page.replies.value.map(r => r.id)).toEqual(['r1'])
    expect(page.focalComment.value?.repliesCount).toBe(1)
    expect(page.repliesTotalCount.value).toBe(1)

    await page.deleteOwnComment(page.replies.value[0]!)
    expect(page.replies.value.length).toBe(0)
    expect(page.focalComment.value?.repliesCount).toBe(0)
    expect(backedTo.length).toBe(0)

    await page.deleteOwnComment(page.focalComment.value!)
    expect(backedTo).toEqual(['/pages/telegraph-entry/index?topicId=topic_1&entryId=entry_1'])
  })

  test('waits for the review result before opening a comment thread', async () => {
    const restricted = ref(true)
    let finishReview!: () => void
    let threadRequests = 0
    const page = useTelegraphCommentPage({
      topicId: () => 'topic_1',
      entryId: () => 'entry_1',
      commentId: () => 'c1',
      session: makeSession(null, false),
      miniReview: {
        shouldRestrictFeatures: restricted,
        ensureMiniReviewReady: () => new Promise(resolve => {
          finishReview = () => {
            restricted.value = false
            resolve(null)
          }
        }),
      },
      api: {
        getTopicDetail: async () => makeRawTopic(),
        getCommentThread: async () => {
          threadRequests += 1
          return {
            comments_enabled: true,
            entry_comments_count: 1,
            ancestors: [],
            comment: rawComment('c1', '根评论'),
            replies_total_count: 0,
            items: [],
            next_cursor: null,
            has_more: false,
          }
        },
      },
      ui: { redirectToLogin: () => {}, toast: () => {} },
    })

    const loading = page.loadInitial()
    await Promise.resolve()
    expect(threadRequests).toBe(0)
    finishReview()
    await loading
    expect(threadRequests).toBe(1)
    expect(page.focalComment.value?.id).toBe('c1')
    expect(page.commentsVisible.value).toBe(true)
  })

  test('closes a directly opened comment thread while reviewing', async () => {
    let threadRequests = 0
    const page = useTelegraphCommentPage({
      topicId: () => 'topic_1',
      entryId: () => 'entry_1',
      commentId: () => 'c1',
      session: makeSession(null, false),
      miniReview: { shouldRestrictFeatures: ref(true), ensureMiniReviewReady: async () => null },
      api: {
        getTopicDetail: async () => makeRawTopic(),
        getCommentThread: async () => {
          threadRequests += 1
          throw new Error('should not request comments')
        },
      },
      ui: { redirectToLogin: () => {}, toast: () => {} },
    })

    await page.loadInitial()
    expect(threadRequests).toBe(0)
    expect(page.focalComment.value).toBeNull()
    expect(page.commentsVisible.value).toBe(false)
    expect(page.errorMessage.value).toBe('评论功能已关闭')
  })
})
