import { afterEach, beforeEach, expect, test } from 'bun:test'
import { effectScope, nextTick, ref, type EffectScope } from 'vue'
import { mapTopicDetail } from '../../utils/dailyTopics'
import type { DailyTopicItem } from '../../types/dailyTopic'
import { useTopicTimeline } from './useTopicTimeline'

interface PendingRequest {
  url: string
  success: (response: { data: unknown; statusCode: number }) => void
}
const teams = [{ team_id: 77680, name: '成都蓉城' }, { team_id: 148, name: '天津津门虎' }]
const article = (id: string, teamId: number, date = '2026-10-13T08:00:00+08:00') => ({
  id, team_id: 77680, title: id, summary: '摘要', paragraphs: ['正文'], published_at: date,
  team_tags: teams.filter(team => team.team_id === teamId),
  sources: [{ title: '原文', publisher: '新闻', url: 'https://news.example.com/article', published_at: date }],
  audio: { status: 'pending' },
})
const anchorItem = (id = 'current') => mapTopicDetail({
  ...article(id, 77680, '2026-10-14T08:00:00+08:00'), team_tags: teams,
})
let requests: PendingRequest[]
let scopes: EffectScope[]
let uniDescriptor: PropertyDescriptor | undefined
beforeEach(() => {
  requests = []; scopes = []
  uniDescriptor = Object.getOwnPropertyDescriptor(globalThis, 'uni')
  Object.defineProperty(globalThis, 'uni', { configurable: true, writable: true,
    value: { request: (options: PendingRequest) => { requests.push(options) } } })
})
afterEach(() => {
  scopes.forEach(scope => scope.stop())
  if (uniDescriptor) Object.defineProperty(globalThis, 'uni', uniDescriptor)
  else Reflect.deleteProperty(globalThis, 'uni')
})
async function setup() {
  const scope = effectScope(); scopes.push(scope)
  const anchor = ref<DailyTopicItem | null>(null)
  const timeline = scope.run(() => useTopicTimeline(anchor))!
  anchor.value = anchorItem()
  await nextTick()
  return { anchor, timeline, scope }
}
async function respond(index: number, ids: string[], next: string | null = null) {
  const request = requests[index]
  const url = new URL(request.url)
  const teamId = Number(url.searchParams.get('tag_team_id'))
  const anchorId = decodeURIComponent(url.pathname.split('/').at(-2)!)
  request.success({ statusCode: 200, data: {
    team_id: 77680, anchor_id: anchorId, tag: teams.find(team => team.team_id === teamId),
    items: ids.map(id => article(id, teamId)), next_cursor: next, has_more: next !== null,
  } })
  await nextTick(); await nextTick()
}

test('切回已加载球队立即恢复列表，不重复请求或显示加载状态', async () => {
  const { timeline } = await setup()
  await respond(0, ['chengdu-a', 'chengdu-b'])
  timeline.selectTeam(148)
  expect(timeline.items.value).toEqual([])
  await respond(1, ['tianjin-a'])
  timeline.selectTeam(77680)
  expect(timeline.items.value.map(item => item.id)).toEqual(['chengdu-a', 'chengdu-b'])
  expect(timeline.loading.value).toBe(false)
  expect(requests.length).toBe(2)
})

test('空列表也是成功缓存，切回不再次请求', async () => {
  const { timeline } = await setup()
  await respond(0, [])
  timeline.selectTeam(148); await respond(1, ['tianjin'])
  timeline.selectTeam(77680)
  expect(timeline.items.value).toEqual([])
  expect(timeline.loading.value).toBe(false)
  expect(requests.length).toBe(2)
})

test('快速往返合并各球队在途请求，未选中球队的结果仍缓存且不串线', async () => {
  const { timeline } = await setup()
  timeline.selectTeam(148); timeline.selectTeam(77680)
  expect(requests.length).toBe(2)
  await respond(1, ['tianjin'])
  expect(timeline.selectedTeamId.value).toBe(77680)
  expect(timeline.items.value).toEqual([])
  expect(timeline.loading.value).toBe(true)
  await respond(0, ['chengdu'])
  expect(timeline.items.value.map(item => item.id)).toEqual(['chengdu'])
  timeline.selectTeam(148)
  expect(timeline.items.value.map(item => item.id)).toEqual(['tianjin'])
  expect(timeline.loading.value).toBe(false)
  expect(requests.length).toBe(2)
})

test('各球队独立保留已加载分页和游标，切回继续加载正确下一页', async () => {
  const { timeline } = await setup()
  await respond(0, ['chengdu-a'], 'chengdu-page-2')
  const more = timeline.loadMore()
  expect(new URL(requests[1].url).searchParams.get('cursor')).toBe('chengdu-page-2')
  await respond(1, ['chengdu-b'], 'chengdu-page-3'); await more
  timeline.selectTeam(148); await respond(2, ['tianjin'], 'tianjin-page-2')
  timeline.selectTeam(77680)
  expect(timeline.items.value.map(item => item.id)).toEqual(['chengdu-a', 'chengdu-b'])
  expect(timeline.nextCursor.value).toBe('chengdu-page-3')
  const last = timeline.loadMore()
  expect(new URL(requests[3].url).searchParams.get('cursor')).toBe('chengdu-page-3')
  await respond(3, ['chengdu-c']); await last
  expect(timeline.items.value.map(item => item.id)).toEqual(['chengdu-a', 'chengdu-b', 'chengdu-c'])
  timeline.selectTeam(148)
  expect(timeline.nextCursor.value).toBe('tianjin-page-2')
})

test('加载更多期间切换球队，后台分页成功写回对应缓存', async () => {
  const { timeline } = await setup()
  await respond(0, ['chengdu-a'], 'page-2')
  const more = timeline.loadMore()
  timeline.selectTeam(148); await respond(2, ['tianjin'])
  await respond(1, ['chengdu-b']); await more
  expect(timeline.items.value.map(item => item.id)).toEqual(['tianjin'])
  timeline.selectTeam(77680)
  expect(timeline.items.value.map(item => item.id)).toEqual(['chengdu-a', 'chengdu-b'])
  expect(timeline.nextCursor.value).toBeNull()
})

test('分页失败保留已有列表，重试仍使用原游标并不影响其他球队', async () => {
  const { timeline } = await setup()
  await respond(0, ['chengdu-a'], 'page-2')
  const more = timeline.loadMore()
  requests[1].success({ statusCode: 500, data: { message: '暂时不可用' } }); await more
  expect(timeline.items.value.map(item => item.id)).toEqual(['chengdu-a'])
  expect(timeline.errorMessage.value).toBe('暂时不可用')
  timeline.selectTeam(148); await respond(2, [])
  timeline.selectTeam(77680)
  expect(timeline.errorMessage.value).toBe('暂时不可用')
  const retry = timeline.retry()
  expect(new URL(requests[3].url).searchParams.get('cursor')).toBe('page-2')
  await respond(3, ['chengdu-b']); await retry
  expect(timeline.errorMessage.value).toBe('')
  expect(timeline.items.value.map(item => item.id)).toEqual(['chengdu-a', 'chengdu-b'])
})

test('首屏错误不当成成功缓存，错误响应可重试', async () => {
  const { timeline } = await setup()
  requests[0].success({ statusCode: 200, data: {} }); await nextTick(); await nextTick()
  expect(Boolean(timeline.errorMessage.value)).toBe(true)
  const retry = timeline.retry()
  await respond(1, ['chengdu']); await retry
  timeline.selectTeam(148); await respond(2, [])
  timeline.selectTeam(77680)
  expect(timeline.items.value.map(item => item.id)).toEqual(['chengdu'])
  expect(requests.length).toBe(3)
})

test('换话题清除球队缓存，旧话题的在途结果不能写入新时间线', async () => {
  const { anchor, timeline } = await setup()
  anchor.value = anchorItem('another'); await nextTick()
  await respond(0, ['stale'])
  expect(timeline.items.value).toEqual([])
  expect(timeline.loading.value).toBe(true)
  await respond(1, ['fresh'])
  expect(timeline.items.value.map(item => item.id)).toEqual(['fresh'])
})

test('页面销毁后忽略在途响应', async () => {
  const { timeline, scope } = await setup()
  scope.stop(); await respond(0, ['stale'])
  expect(timeline.items.value).toEqual([])
})
