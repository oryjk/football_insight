<template>
  <view class="topic-timeline">
    <text class="topic-timeline__heading">话题追踪</text>
    <view class="topic-timeline__tags"><FiTopicTeamTags :tags="tags" :selected-team-id="selectedTeamId" @select="$emit('select', $event)" /></view>
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
</template>
<script setup lang="ts">
import type { DailyTopicItem, DailyTopicTag } from '../../../types/dailyTopic'
import FiTopicTeamTags from '../../../components/topics/FiTopicTeamTags.vue'
defineProps<{ anchor: DailyTopicItem; tags: DailyTopicTag[]; selectedTeamId: number | null; items: DailyTopicItem[]; loading: boolean; errorMessage: string; hasMore: boolean }>()
defineEmits<{ (event: 'select', teamId: number): void; (event: 'read', id: string): void; (event: 'more'): void; (event: 'retry'): void }>()
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
