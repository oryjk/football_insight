import { describe, expect, test } from 'bun:test'
import { ref } from 'vue'
import type { CurrentUser } from '../../types/auth'
import { ApiRequestError } from '../../utils/apiError'
import { useTelegraphEntryPage, type EntrySessionPort } from './useTelegraphEntryPage'

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

function makeRawTopic(overrides: Record<string, unknown> = {}) {
  return {
    id: 'topic_1',
    team_id: 77680,
    title: '每日电报',
    summary: '摘要',
    paragraphs: ['正文'],
    published_at: '2026-11-17T07:00:00Z',
    heat: 10,
    comments_enabled: true,
    comments_count: 1,
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
        comments_count: 1,
        liked_by_me: false,
        disliked_by_me: false,
      },
    ],
    ...overrides,
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
        avatar_url: avatar_data_url ?? currentUser.value?.avatar_url ?? 'https://cdn.example.com/a.jpg',
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

describe('useTelegraphEntryPage', () => {
  test('loads target entry and initial X timeline comments page', async () => {
    const page = useTelegraphEntryPage({
      topicId: () => 'topic_1',
      entryId: () => 'entry_1',
      session: makeSession(null, false),
      miniReview: { shouldRestrictFeatures: ref(false) },
      api: {
        getTopicDetail: async () => makeRawTopic(),
        getComments: async () => ({
          comments_enabled: true,
          total_count: 1,
          has_more: false,
          next_cursor: null,
          items: [
            {
              id: 'c1',
              topic_id: 'topic_1',
              entry_id: 'entry_1',
              author: { display_name: '南看台', avatar_url: 'https://cdn.example/1.jpg' },
              reply_to: null,
              content: '期待官宣',
              likes_count: 3,
              liked_by_me: false,
              replies_count: 0,
              is_mine: false,
              created_at: '2026-11-17T06:58:00Z',
            },
          ],
        }),
      },
      ui: { redirectToLogin: () => {}, toast: () => {} },
    })

    await page.loadInitial()
    expect(page.errorMessage.value).toBe('')
    expect(page.entry.value?.title).toBe('斯坦丘有望加盟')
    expect(page.comments.value.length).toBe(1)
    expect(page.comments.value[0]?.likesCount).toBe(3)
  })

  test('comment like requires login and toggles single-comment heart count', async () => {
    const redirects: string[] = []
    const guestPage = useTelegraphEntryPage({
      topicId: () => 'topic_1',
      entryId: () => 'entry_1',
      session: makeSession(null, false),
      miniReview: { shouldRestrictFeatures: ref(false) },
      api: {
        getTopicDetail: async () => makeRawTopic(),
        getComments: async () => ({
          comments_enabled: true,
          total_count: 1,
          has_more: false,
          next_cursor: null,
          items: [{
            id: 'c1',
            topic_id: 'topic_1',
            entry_id: 'entry_1',
            author: { display_name: '球迷', avatar_url: null },
            reply_to: null,
            content: '好消息',
            likes_count: 0,
            liked_by_me: false,
            replies_count: 0,
            is_mine: false,
            created_at: '2026-11-17T06:58:00Z',
          }],
        }),
      },
      ui: { redirectToLogin: url => redirects.push(url), toast: () => {} },
    })

    await guestPage.loadInitial()
    await guestPage.toggleCommentLike(guestPage.comments.value[0]!)
    expect(redirects).toEqual(['/pages/telegraph-entry/index?topicId=topic_1&entryId=entry_1'])

    const authedPage = useTelegraphEntryPage({
      topicId: () => 'topic_1',
      entryId: () => 'entry_1',
      session: makeSession(makeUser()),
      miniReview: { shouldRestrictFeatures: ref(false) },
      api: {
        getTopicDetail: async () => makeRawTopic(),
        getComments: async () => ({
          comments_enabled: true,
          total_count: 1,
          has_more: false,
          next_cursor: null,
          items: [{
            id: 'c1',
            topic_id: 'topic_1',
            entry_id: 'entry_1',
            author: { display_name: '球迷', avatar_url: null },
            reply_to: null,
            content: '好消息',
            likes_count: 0,
            liked_by_me: false,
            replies_count: 0,
            is_mine: false,
            created_at: '2026-11-17T06:58:00Z',
          }],
        }),
        setCommentLike: async () => ({ comment_id: 'c1', likes_count: 1, liked_by_me: true }),
      },
      ui: { redirectToLogin: () => {}, toast: () => {} },
    })

    await authedPage.loadInitial()
    await authedPage.toggleCommentLike(authedPage.comments.value[0]!)
    expect(authedPage.comments.value[0]?.likesCount).toBe(1)
    expect(authedPage.comments.value[0]?.likedByMe).toBe(true)
  })

  test('filters out nested replies when backend is legacy and navigates to comment detail on openComment', async () => {
    const openedUrls: string[] = []
    let capturedRootOnly: boolean | undefined
    const page = useTelegraphEntryPage({
      topicId: () => 'topic_1',
      entryId: () => 'entry_1',
      session: makeSession(makeUser({ display_name: '13800138000', profile_completed: false })),
      miniReview: { shouldRestrictFeatures: ref(false) },
      api: {
        getTopicDetail: async () => makeRawTopic(),
        getComments: async (_t, _tid, _eid, _cursor, _limit, opts) => {
          capturedRootOnly = opts?.rootOnly
          return {
            comments_enabled: true,
            total_count: 2,
            has_more: false,
            next_cursor: null,
            items: [
              {
                id: 'r1',
                topic_id: 'topic_1',
                entry_id: 'entry_1',
                author: { display_name: '楼中回复者', avatar_url: 'https://cdn.example/b.jpg' },
                reply_to: { comment_id: 'c1', author_display_name: '球迷甲' },
                content: '子回复不应混在一级列表',
                likes_count: 0,
                liked_by_me: false,
                replies_count: 0,
                is_mine: false,
                created_at: '2026-11-17T06:59:00Z',
              },
              {
                id: 'c1',
                topic_id: 'topic_1',
                entry_id: 'entry_1',
                author: { display_name: '球迷甲', avatar_url: 'https://cdn.example/a.jpg' },
                reply_to: null,
                content: '第一条评论',
                likes_count: 0,
                liked_by_me: false,
                replies_count: 1,
                is_mine: false,
                created_at: '2026-11-17T06:58:00Z',
              },
            ],
          }
        },
        createComment: async (_teamId, _topicId, _entryId, payload) => ({
          comments_count: 3,
          comment: {
            id: 'c2',
            topic_id: 'topic_1',
            entry_id: 'entry_1',
            author: { display_name: '川足铁杆', avatar_url: 'data:image/png;base64,AAAA' },
            reply_to: null,
            content: payload.content,
            likes_count: 0,
            liked_by_me: false,
            replies_count: 0,
            is_mine: true,
            created_at: '2026-11-17T07:01:00Z',
          },
        }),
        deleteComment: async () => ({ comments_count: 2 }),
      },
      ui: {
        redirectToLogin: () => {},
        toast: () => {},
        openComment: url => openedUrls.push(url),
      },
    })

    await page.loadInitial()
    expect(capturedRootOnly).toBe(true)
    expect(page.comments.value.map(c => c.id)).toEqual(['c1'])

    page.draftContent.value = '直接评论电报'
    await page.submitComment()
    expect(page.profileSheetVisible.value).toBe(true)
    expect(page.draftContent.value).toBe('直接评论电报')

    await page.saveProfileAndContinue({
      display_name: '川足铁杆',
      avatar_data_url: 'data:image/png;base64,AAAA',
    })
    expect(page.profileSheetVisible.value).toBe(false)
    expect(page.comments.value.map(c => c.id)).toEqual(['c2', 'c1'])

    page.openComment(page.comments.value[1]!, false)
    await page.startReply(page.comments.value[1]!)
    expect(openedUrls).toEqual([
      '/pages/telegraph-entry/comment?topicId=topic_1&entryId=entry_1&commentId=c1',
      '/pages/telegraph-entry/comment?topicId=topic_1&entryId=entry_1&commentId=c1&focus=1',
    ])

    await page.deleteOwnComment(page.comments.value[0]!)
    expect(page.comments.value.map(c => c.id)).toEqual(['c1'])
  })

  test('server 403 PROFILE_INCOMPLETE re-opens profile setup sheet and keeps draft', async () => {
    const page = useTelegraphEntryPage({
      topicId: () => 'topic_1',
      entryId: () => 'entry_1',
      session: makeSession(makeUser({ display_name: '老球迷', avatar_url: 'https://cdn.example/a.jpg', profile_completed: true })),
      miniReview: { shouldRestrictFeatures: ref(false) },
      api: {
        getTopicDetail: async () => makeRawTopic(),
        getComments: async () => ({
          comments_enabled: true,
          total_count: 0,
          has_more: false,
          next_cursor: null,
          items: [],
        }),
        createComment: async () => {
          throw new ApiRequestError('请先完善头像和昵称', 403, false, { code: 'PROFILE_INCOMPLETE' })
        },
      },
      ui: { redirectToLogin: () => {}, toast: () => {} },
    })

    await page.loadInitial()
    page.draftContent.value = '保留草稿'
    await page.submitComment()
    expect(page.profileSheetVisible.value).toBe(true)
    expect(page.draftContent.value).toBe('保留草稿')
  })
})
