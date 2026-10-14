<template>
  <view class="topic-timeline">
    <text class="topic-timeline__heading">话题追踪</text>
    <view class="topic-timeline__tags"><FiTopicTeamTags :tags="tags" :selected-team-id="selectedTeamId" @select="selectTeam" /></view>
    <view class="topic-timeline__body" :style="{ minHeight: `${minimumHeight}rpx` }">
      <view class="topic-timeline__list">
        <view class="topic-timeline__row topic-timeline__row--current">
          <text class="topic-timeline__date">{{ anchor.publishedLabel.split(' ')[0] }}</text>
          <view class="topic-timeline__content"><view class="topic-timeline__dot" /><text class="topic-timeline__current">当前话题</text><text class="topic-timeline__title">{{ anchor.title }}</text></view>
        </view>
        <view v-for="item in items" :key="item.id" class="topic-timeline__row" @tap="$emit('read', item.id)">
          <text class="topic-timeline__date">{{ item.publishedLabel.split(' ')[0] }}</text>
          <view class="topic-timeline__content"><view class="topic-timeline__dot" /><text class="topic-timeline__title">{{ item.title }}</text></view>
        </view>
        <view class="topic-timeline__state">
          <text v-if="loading">正在加载追踪话题…</text>
          <template v-else-if="errorMessage"><text>{{ errorMessage }}</text><button @tap="$emit('retry')">重新加载</button></template>
          <text v-else-if="!items.length">该球队暂无更早的话题</text>
          <button v-else-if="hasMore" @tap="$emit('more')">加载更多</button>
          <text v-else>已显示此前全部话题</text>
        </view>
      </view>
    </view>
  </view>
</template>
<script setup lang="ts">
import { getCurrentInstance, onScopeDispose, ref, watch } from 'vue'
import type { DailyTopicItem, DailyTopicTag } from '../../../types/dailyTopic'
import FiTopicTeamTags from '../../../components/topics/FiTopicTeamTags.vue'
const props = defineProps<{ anchor: DailyTopicItem; tags: DailyTopicTag[]; selectedTeamId: number | null; items: DailyTopicItem[]; loading: boolean; errorMessage: string; hasMore: boolean }>()
const emit = defineEmits<{ (event: 'select', teamId: number): void; (event: 'read', id: string): void; (event: 'more'): void; (event: 'retry'): void }>()
const instance = getCurrentInstance()
const minimumHeight = ref(0)
let selection = 0
let disposed = false

// Keep the tallest displayed list for this article. Short/empty teams must not
// shrink the page and force its bottom scroll position upwards.
function reserveHeight(): Promise<void> {
  if (!instance || disposed) return Promise.resolve()
  const anchorId = props.anchor.id
  return new Promise(resolve => {
    const query = uni.createSelectorQuery().in(instance)
    query.select('.topic-timeline__list').boundingClientRect()
    query.exec(result => {
      const rect = result?.[0] as { height?: number } | undefined
      const width = uni.getSystemInfoSync().windowWidth
      if (!disposed && props.anchor.id === anchorId && typeof rect?.height === 'number' && width > 0) {
        minimumHeight.value = Math.max(minimumHeight.value, rect.height * 750 / width)
      }
      resolve()
    })
  })
}
async function selectTeam(teamId: number) {
  const current = ++selection
  if (teamId === props.selectedTeamId) return
  const anchorId = props.anchor.id
  await reserveHeight()
  // uni-app's instance tick also waits for the native setData callback.
  await instance?.proxy?.$nextTick()
  if (!disposed && current === selection && anchorId === props.anchor.id) emit('select', teamId)
}
watch(() => props.anchor.id, () => { minimumHeight.value = 0; selection += 1 })
watch(() => [props.items, props.loading, props.errorMessage, props.hasMore], async () => {
  await instance?.proxy?.$nextTick()
  void reserveHeight()
}, { immediate: true, flush: 'post' })
onScopeDispose(() => { disposed = true; selection += 1 })
</script>
<style scoped>
.topic-timeline { border-top: var(--fi-border-divider); padding-top: var(--fi-space-24); }
.topic-timeline__heading { display: block; color: var(--fi-color-text-primary); font-size: var(--fi-font-28); font-weight: var(--fi-weight-bold); }
.topic-timeline__tags { margin: var(--fi-space-20) 0 var(--fi-space-28); }
.topic-timeline__row { display: flex; align-items: stretch; }
.topic-timeline__date { flex-shrink: 0; width: calc(var(--fi-space-64) + var(--fi-space-64) + var(--fi-space-32)); color: var(--fi-color-text-caption); font-size: var(--fi-font-22); line-height: var(--fi-leading-normal); padding-top: var(--fi-space-4); }
.topic-timeline__content { position: relative; flex: 1; min-width: 0; padding: 0 0 var(--fi-space-28) var(--fi-space-24); border-left: var(--fi-border-divider); }
.topic-timeline__dot { position: absolute; top: var(--fi-space-8); left: 0; transform: translateX(-50%); width: var(--fi-space-12); height: var(--fi-space-12); border-radius: var(--fi-radius-round); background: var(--fi-color-primary); }
.topic-timeline__title { display: block; color: var(--fi-color-text-primary); font-size: var(--fi-font-26); line-height: var(--fi-leading-normal); }
.topic-timeline__current { display: block; margin-bottom: var(--fi-space-8); color: var(--fi-color-primary); font-size: var(--fi-font-20); }
.topic-timeline__state { color: var(--fi-color-text-caption); font-size: var(--fi-font-22); line-height: var(--fi-leading-normal); padding-top: var(--fi-space-12); }
.topic-timeline__state button { margin: var(--fi-space-16) 0 0; padding: var(--fi-space-12) var(--fi-space-20); background: var(--fi-color-bg); color: var(--fi-color-primary); font-size: var(--fi-font-24); line-height: var(--fi-leading-normal); border-radius: var(--fi-radius-lg); }
.topic-timeline__state button::after { border: 0; }
</style>
