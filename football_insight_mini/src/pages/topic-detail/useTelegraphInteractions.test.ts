import { describe, expect, test } from 'bun:test'
import { ref } from 'vue'
import type { CurrentUser } from '../../types/auth'
import type { DailyTopicEntry, DailyTopicItem } from '../../types/dailyTopic'
import { ApiRequestError } from '../../utils/apiError'
import { useTelegraphInteractions, type TelegraphSessionPort } from './useTelegraphInteractions'

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

function makeEntry(overrides: Partial<DailyTopicEntry> = {}): DailyTopicEntry {
  return {
    id: 'entry-1',
    category: 'domestic',
    title: '中超快讯',
    text: '蓉城客场取胜',
    publishedAt: '2026-11-17T10:00:00Z',
    timeLabel: '18:00',
    sources: [],
    likesCount: 2,
    dislikesCount: 1,
    commentsCount: 1,
    likedByMe: false,
    dislikedByMe: false,
    ...overrides,
  }
}

function makeTopic(entryOverrides: Partial<DailyTopicEntry> = {}, itemOverrides: Partial<DailyTopicItem> = {}): DailyTopicItem {
  return {
    id: 'topic_2026-11-17',
    teamId: 77680,
    title: '每日电报',
    summary: '今日足坛快讯',
    paragraphs: [],
    publishedAt: '2026-11-17T10:00:00Z',
    publishedLabel: '11-17 18:00',
    sources: [],
    sourceLabel: '汇总',
    heat: 100,
    commentsEnabled: true,
    commentsCount: 1,
    readMinutes: 2,
    audio: { status: 'ready', url: null, durationSeconds: null },
    entries: [makeEntry(entryOverrides)],
    categoryAudio: {},
    ...itemOverrides,
  }
}

function makeSession(initialUser: CurrentUser | null, loggedIn = Boolean(initialUser)): TelegraphSessionPort {
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

describe('useTelegraphInteractions', () => {
  test('guest tapping like redirects to user tab with return url without calling API', async () => {
    const item = ref<DailyTopicItem | null>(makeTopic())
    const redirects: string[] = []
    let likeCalls = 0
    const interactions = useTelegraphInteractions({
      topicId: () => 'topic_2026-11-17',
      item,
      session: makeSession(null, false),
      miniReview: { shouldRestrictFeatures: ref(false) },
      api: {
        setLike: async () => {
          likeCalls += 1
          return {}
        },
      },
      ui: {
        redirectToLogin: url => redirects.push(url),
        toast: () => {},
      },
    })

    await interactions.likeEntry(item.value!.entries[0])

    expect(likeCalls).toBe(0)
    expect(redirects).toEqual(['/pages/topic-detail/index?id=topic_2026-11-17'])
  })

  test('logged-in user toggles like and mutual-exclusion dislike, guarding duplicate pending taps', async () => {
    const item = ref<DailyTopicItem | null>(makeTopic())
    let resolveLike!: (value: unknown) => void
    let likeCalls = 0
    const interactions = useTelegraphInteractions({
      topicId: () => 'topic_2026-11-17',
      item,
      session: makeSession(makeUser()),
      miniReview: { shouldRestrictFeatures: ref(false) },
      api: {
        setLike: () => {
          likeCalls += 1
          return new Promise((resolve) => {
            resolveLike = resolve
          })
        },
        setDislike: async () => ({
          entry_id: 'entry-1',
          likes_count: 2,
          dislikes_count: 2,
          liked_by_me: false,
          disliked_by_me: true,
        }),
      },
      ui: { redirectToLogin: () => {}, toast: () => {} },
    })

    const firstTap = interactions.likeEntry(item.value!.entries[0])
    const secondTap = interactions.likeEntry(item.value!.entries[0])
    expect(interactions.pendingReactionIds.value).toEqual(['entry-1'])

    resolveLike({
      entry_id: 'entry-1',
      likes_count: 3,
      dislikes_count: 1,
      liked_by_me: true,
      disliked_by_me: false,
    })
    await Promise.all([firstTap, secondTap])

    expect(likeCalls).toBe(1)
    expect(interactions.pendingReactionIds.value).toEqual([])
    expect(item.value!.entries[0].likesCount).toBe(3)
    expect(item.value!.entries[0].likedByMe).toBe(true)

    await interactions.dislikeEntry(item.value!.entries[0])
    expect(item.value!.entries[0].likesCount).toBe(2)
    expect(item.value!.entries[0].dislikesCount).toBe(2)
    expect(item.value!.entries[0].likedByMe).toBe(false)
    expect(item.value!.entries[0].dislikedByMe).toBe(true)
  })

  test('commentsVisible hides comment UI during review restriction or kill switch', () => {
    const item = ref<DailyTopicItem | null>(makeTopic({}, { commentsEnabled: true }))
    const restrict = ref(true)
    const interactions = useTelegraphInteractions({
      topicId: () => 'topic_2026-11-17',
      item,
      session: makeSession(null, false),
      miniReview: { shouldRestrictFeatures: restrict },
      ui: { redirectToLogin: () => {}, toast: () => {} },
    })

    expect(interactions.commentsVisible.value).toBe(false)
    restrict.value = false
    expect(interactions.commentsVisible.value).toBe(true)
    item.value = makeTopic({}, { commentsEnabled: false })
    expect(interactions.commentsVisible.value).toBe(false)
  })

  test('incomplete profile opens setup sheet, keeps draft intact, and auto-submits comment after saving', async () => {
    const item = ref<DailyTopicItem | null>(makeTopic())
    const submittedPayloads: Array<{ content: string; reply_to_comment_id?: string | null }> = []
    const interactions = useTelegraphInteractions({
      topicId: () => 'topic_2026-11-17',
      item,
      session: makeSession(makeUser({ display_name: '13800138000', avatar_url: null, profile_completed: false })),
      miniReview: { shouldRestrictFeatures: ref(false) },
      api: {
        getComments: async () => ({
          comments_enabled: true,
          total_count: 1,
          items: [],
          next_cursor: null,
          has_more: false,
        }),
        createComment: async (_teamId, _topicId, _entryId, payload) => {
          submittedPayloads.push(payload)
          return {
            comments_count: 2,
            comment: {
              id: 'comment-2',
              topic_id: 'topic_2026-11-17',
              entry_id: 'entry-1',
              author: { display_name: '川足老炮', avatar_url: 'https://cdn.example.com/avatar.jpg' },
              reply_to: null,
              content: payload.content,
              is_mine: true,
              created_at: '2026-11-17T10:05:00Z',
            },
          }
        },
      },
      ui: { redirectToLogin: () => {}, toast: () => {} },
    })

    interactions.openComments(item.value!.entries[0])
    await new Promise(resolve => setTimeout(resolve, 0))

    interactions.draftContent.value = '这场反击真犀利！'
    await interactions.submitComment()

    expect(submittedPayloads).toEqual([])
    expect(interactions.profileSheetVisible.value).toBe(true)
    expect(interactions.draftContent.value).toBe('这场反击真犀利！')

    await interactions.saveProfileAndContinue({
      display_name: '川足老炮',
      avatar_data_url: 'data:image/png;base64,AAAA',
    })

    expect(interactions.profileSheetVisible.value).toBe(false)
    expect(submittedPayloads).toEqual([{ content: '这场反击真犀利！', reply_to_comment_id: null }])
    expect(interactions.draftContent.value).toBe('')
    expect(interactions.comments.value.map(c => c.id)).toEqual(['comment-2'])
    expect(item.value!.entries[0].commentsCount).toBe(2)
    expect(item.value!.commentsCount).toBe(2)
  })

  test('server 403 PROFILE_INCOMPLETE re-opens profile setup sheet and keeps draft', async () => {
    const item = ref<DailyTopicItem | null>(makeTopic())
    const session = makeSession(makeUser({
      display_name: '球迷甲',
      avatar_url: 'https://cdn.example.com/a.jpg',
      profile_completed: true,
    }))
    const interactions = useTelegraphInteractions({
      topicId: () => 'topic_2026-11-17',
      item,
      session,
      miniReview: { shouldRestrictFeatures: ref(false) },
      api: {
        getComments: async () => ({
          comments_enabled: true,
          total_count: 1,
          items: [],
          next_cursor: null,
          has_more: false,
        }),
        createComment: async () => {
          throw new ApiRequestError('请先完善头像和昵称', 403, false, { code: 'PROFILE_INCOMPLETE' })
        },
      },
      ui: { redirectToLogin: () => {}, toast: () => {} },
    })

    interactions.openComments(item.value!.entries[0])
    await new Promise(resolve => setTimeout(resolve, 0))
    interactions.draftContent.value = '保留这条草稿'
    await interactions.submitComment()

    expect(interactions.profileSheetVisible.value).toBe(true)
    expect(interactions.draftContent.value).toBe('保留这条草稿')
  })

  test('deleting own comment removes it from list and decrements entry & topic counts', async () => {
    const item = ref<DailyTopicItem | null>(makeTopic({ commentsCount: 2 }, { commentsCount: 5 }))
    const interactions = useTelegraphInteractions({
      topicId: () => 'topic_2026-11-17',
      item,
      session: makeSession(makeUser({
        display_name: '球迷甲',
        avatar_url: 'https://cdn.example.com/a.jpg',
        profile_completed: true,
      })),
      miniReview: { shouldRestrictFeatures: ref(false) },
      api: {
        getComments: async () => ({
          comments_enabled: true,
          total_count: 2,
          items: [
            {
              id: 'comment-mine',
              topic_id: 'topic_2026-11-17',
              entry_id: 'entry-1',
              author: { display_name: '球迷甲', avatar_url: 'https://cdn.example.com/a.jpg' },
              reply_to: null,
              content: '我的评论',
              is_mine: true,
              created_at: '2026-11-17T10:02:00Z',
            },
          ],
          next_cursor: null,
          has_more: false,
        }),
        deleteComment: async () => ({ comments_count: 1 }),
      },
      ui: { redirectToLogin: () => {}, toast: () => {} },
    })

    interactions.openComments(item.value!.entries[0])
    await new Promise(resolve => setTimeout(resolve, 0))
    expect(interactions.comments.value.length).toBe(1)

    await interactions.deleteOwnComment(interactions.comments.value[0])

    expect(interactions.comments.value).toEqual([])
    expect(item.value!.entries[0].commentsCount).toBe(1)
    expect(item.value!.commentsCount).toBe(4)
  })
})
