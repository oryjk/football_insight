<template>
  <!-- 每日话题：电报式快讯流。整期一个播报，每条只有时间、标题、正文和来源，不单独统计或播报。 -->
  <view class="roundup">
    <view class="roundup__day">
      <text class="roundup__date">{{ dayLabel }}</text>
      <FiTopicAudioEntry :ready="audioReady" :playing="audioPlaying" :loading="audioLoading" label="播报全部" @play="emit('play')" />
    </view>
    <view v-for="entry in item.entries" :key="entry.id" class="roundup__entry">
      <text class="roundup__line" :style="{ fontSize: `${fontRpx}rpx` }">
        <text class="roundup__time">{{ entry.timeLabel }}</text>
        <text class="roundup__title">【{{ entry.title }}】</text>
        <text>{{ entry.text }}</text>
      </text>
      <!-- 来源默认折叠，只露出媒体名；点开后逐条列出，可复制原文链接。 -->
      <view v-if="entry.sources.length" class="roundup__source-toggle" role="button" :aria-expanded="expanded.has(entry.id)" hover-class="roundup__pressed" @tap="toggle(entry.id)">
        <text>来源：{{ publishers(entry) }}</text>
        <text class="roundup__chevron" :class="{ 'roundup__chevron--open': expanded.has(entry.id) }">›</text>
      </view>
      <view v-if="expanded.has(entry.id)" class="roundup__sources">
        <view v-for="source in entry.sources" :key="source.url" class="roundup__source">
          <text class="roundup__source-title">{{ source.title }}</text>
          <view class="roundup__source-meta">
            <text>{{ source.publisher }} · {{ source.dateLabel }}</text>
            <text class="roundup__copy" role="button" hover-class="roundup__pressed" @tap="emit('copy', source.url)">复制原文链接</text>
          </view>
        </view>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed, reactive } from 'vue'
import FiTopicAudioEntry from '../../../components/topics/FiTopicAudioEntry.vue'
import type { DailyTopicEntry, DailyTopicItem } from '../../../types/dailyTopic'
import { roundupDayLabel } from '../../../utils/dailyTopics'

const props = defineProps<{ item: DailyTopicItem; fontRpx: number; audioReady: boolean; audioPlaying: boolean; audioLoading: boolean }>()
const emit = defineEmits<{ (e: 'play'): void; (e: 'copy', url: string): void }>()
const dayLabel = computed(() => roundupDayLabel(props.item.publishedAt))
const expanded = reactive(new Set<string>())
function toggle(entryId: string) {
  if (expanded.has(entryId)) expanded.delete(entryId)
  else expanded.add(entryId)
}
function publishers(entry: DailyTopicEntry): string {
  return Array.from(new Set(entry.sources.map(source => source.publisher))).join(' · ')
}
</script>

<style scoped>
.roundup__day { display: flex; align-items: center; justify-content: space-between; gap: var(--fi-space-16); padding-bottom: var(--fi-space-20); border-bottom: var(--fi-border-divider); }
.roundup__date { color: var(--fi-color-text-secondary); font-size: var(--fi-font-26); font-weight: var(--fi-weight-bold); }
.roundup__entry { padding: var(--fi-space-28) 0 var(--fi-space-20); border-bottom: var(--fi-border-divider); }
.roundup__line { display: block; color: var(--fi-color-text-primary); line-height: var(--fi-leading-relaxed); }
.roundup__time { color: var(--fi-color-primary); font-weight: var(--fi-weight-bold); margin-right: var(--fi-space-8); }
.roundup__title { font-weight: var(--fi-weight-bold); }
.roundup__source-toggle { display: inline-flex; align-items: center; gap: var(--fi-space-8); margin-top: var(--fi-space-12); padding: var(--fi-space-8) 0; color: var(--fi-color-text-caption); font-size: var(--fi-font-22); line-height: var(--fi-leading-normal); }
.roundup__chevron { display: inline-block; transform: rotate(90deg); transition: transform 0.2s; }
.roundup__chevron--open { transform: rotate(-90deg); }
.roundup__pressed { opacity: 0.6; }
.roundup__sources { margin-top: var(--fi-space-8); padding: var(--fi-space-4) var(--fi-space-20); border-radius: var(--fi-radius-md); background: var(--fi-color-bg); }
.roundup__source { padding: var(--fi-space-16) 0; border-bottom: var(--fi-border-divider); }
.roundup__source:last-child { border-bottom: 0; }
.roundup__source-title { display: block; color: var(--fi-color-text-secondary); font-size: var(--fi-font-24); line-height: var(--fi-leading-normal); }
.roundup__source-meta { display: flex; align-items: center; justify-content: space-between; gap: var(--fi-space-16); margin-top: var(--fi-space-8); color: var(--fi-color-text-caption); font-size: var(--fi-font-22); }
.roundup__copy { flex-shrink: 0; color: var(--fi-color-primary); }
</style>
