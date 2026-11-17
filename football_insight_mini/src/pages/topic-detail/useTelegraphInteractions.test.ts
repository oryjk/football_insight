import { describe, expect, test } from 'bun:test'
import { ref } from 'vue'
import type { DailyTopicEntry, DailyTopicItem } from '../../types/dailyTopic'
import { useTelegraphInteractions, type TelegraphSessionPort } from './useTelegraphInteractions'

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

function makeSession(loggedIn: boolean): TelegraphSessionPort {
  const hasAccessToken = ref(loggedIn)
  const revision = ref(1)
  return {
    hasAccessToken,
    revision,
    invalidateSession: () => {
      hasAccessToken.value = false
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
      session: makeSession(false),
      miniReview: { shouldRestrictFeatures: ref(false) },
      api: {
        setLike: async () => {
          likeCalls += 1
          return {}
        },
      },
      ui: {
        redirectToLogin: url => redirects.push(url),
        navigateToEntry: () => {},
        toast: () => {},
      },
    })

    await interactions.likeEntry(item.value!.entries[0]!)

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
      session: makeSession(true),
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
      ui: { redirectToLogin: () => {}, navigateToEntry: () => {}, toast: () => {} },
    })

    const firstTap = interactions.likeEntry(item.value!.entries[0]!)
    const secondTap = interactions.likeEntry(item.value!.entries[0]!)
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
    expect(item.value!.entries[0]!.likesCount).toBe(3)
    expect(item.value!.entries[0]!.likedByMe).toBe(true)

    await interactions.dislikeEntry(item.value!.entries[0]!)
    expect(item.value!.entries[0]!.likesCount).toBe(2)
    expect(item.value!.entries[0]!.dislikesCount).toBe(2)
    expect(item.value!.entries[0]!.likedByMe).toBe(false)
    expect(item.value!.entries[0]!.dislikedByMe).toBe(true)
  })

  test('openEntry builds entry detail route and strips focus flag when comments are hidden', () => {
    const item = ref<DailyTopicItem | null>(makeTopic({}, { commentsEnabled: true }))
    const restrict = ref(true)
    const navigated: string[] = []
    const interactions = useTelegraphInteractions({
      topicId: () => 'topic_2026-11-17',
      item,
      session: makeSession(false),
      miniReview: { shouldRestrictFeatures: restrict },
      ui: {
        redirectToLogin: () => {},
        navigateToEntry: url => navigated.push(url),
        toast: () => {},
      },
    })

    expect(interactions.commentsVisible.value).toBe(false)
    interactions.openEntry(item.value!.entries[0]!, true)
    restrict.value = false
    expect(interactions.commentsVisible.value).toBe(true)
    interactions.openEntry(item.value!.entries[0]!, false)
    interactions.openEntry(item.value!.entries[0]!, true)

    expect(navigated).toEqual([
      '/pages/telegraph-entry/index?topicId=topic_2026-11-17&entryId=entry-1',
      '/pages/telegraph-entry/index?topicId=topic_2026-11-17&entryId=entry-1',
      '/pages/telegraph-entry/index?topicId=topic_2026-11-17&entryId=entry-1&focus=1',
    ])
  })
})
