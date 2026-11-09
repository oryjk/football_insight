<template>
  <!-- 每日电报：顶部「全部 / 国内 / 国际 / 留洋」切换。全部按时间混排并标分类；切到某类只看这一类，播报也只播这一类。 -->
  <view class="roundup">
    <view class="roundup__day">
      <view class="roundup__day-copy">
        <text class="roundup__date">{{ dayLabel }}</text>
        <text class="roundup__total">共 {{ item.entries.length }} 条</text>
      </view>
      <FiTopicAudioEntry :ready="audioReady" :playing="audioPlaying" :loading="audioLoading" :label="`播报${activeLabel}`" @play="emit('play', active)" />
    </view>
    <view class="roundup__tabs" role="tablist" aria-label="电报分类">
      <view v-for="tab in tabs" :key="tab.key" class="roundup__tab" :class="{ 'roundup__tab--active': tab.key === activeKey }"
        role="tab" :aria-selected="tab.key === activeKey" hover-class="roundup__pressed" @tap="select(tab.value)">
        <text>{{ tab.label }}</text>
        <text class="roundup__tab-count">{{ tab.count }}</text>
      </view>
    </view>
    <view v-if="!visible.length" class="roundup__empty">今天还没有{{ activeLabel }}消息，每小时更新一次</view>
    <view v-for="entry in visible" :key="entry.id" class="roundup__entry">
      <text class="roundup__line" :style="{ fontSize: `${fontRpx}rpx` }">
        <text class="roundup__time">{{ entry.timeLabel }}</text>
        <text class="roundup__title">【{{ entry.title }}】</text>
        <text>{{ entry.text }}</text>
      </text>
      <view class="roundup__meta">
        <text v-if="active === null" class="roundup__tag" :class="`roundup__tag--${entry.category}`">{{ telegraphCategoryLabel(entry.category) }}</text>
        <!-- 来源默认折叠，只露出媒体名；点开后逐条列出，可复制原文链接。 -->
        <view v-if="entry.sources.length" class="roundup__source-toggle" role="button" :aria-expanded="expanded.has(entry.id)" hover-class="roundup__pressed" @tap="toggle(entry.id)">
          <text>来源：{{ publishers(entry) }}</text>
          <text class="roundup__chevron" :class="{ 'roundup__chevron--open': expanded.has(entry.id) }">›</text>
        </view>
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
import type { DailyTopicEntry, DailyTopicItem, TelegraphCategory } from '../../../types/dailyTopic'
import { roundupDayLabel, TELEGRAPH_CATEGORIES, telegraphCategoryCounts, telegraphCategoryLabel } from '../../../utils/dailyTopics'

const props = defineProps<{ item: DailyTopicItem; fontRpx: number; active: TelegraphCategory | null; audioReady: boolean; audioPlaying: boolean; audioLoading: boolean }>()
const emit = defineEmits<{ (e: 'play', category: TelegraphCategory | null): void; (e: 'select', category: TelegraphCategory | null): void; (e: 'copy', url: string): void }>()
const dayLabel = computed(() => roundupDayLabel(props.item.publishedAt))
const counts = computed(() => telegraphCategoryCounts(props.item))
const tabs = computed(() => [
  { key: 'all', value: null, label: '全部', count: props.item.entries.length },
  ...TELEGRAPH_CATEGORIES.map(({ value, label }) => ({ key: value, value, label, count: counts.value[value] })),
])
const activeKey = computed(() => props.active ?? 'all')
const activeLabel = computed(() => props.active ? telegraphCategoryLabel(props.active) : '全部')
const visible = computed(() => props.active ? props.item.entries.filter(entry => entry.category === props.active) : props.item.entries)
function select(category: TelegraphCategory | null) { if (category !== props.active) emit('select', category) }
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
.roundup__day { display: flex; align-items: center; justify-content: space-between; gap: var(--fi-space-16); }
.roundup__day-copy { display: flex; align-items: baseline; gap: var(--fi-space-16); min-width: 0; }
.roundup__date { color: var(--fi-color-text-primary); font-size: var(--fi-font-30); font-weight: var(--fi-weight-extrabold); }
.roundup__total { color: var(--fi-color-text-secondary); font-size: var(--fi-font-24); }
.roundup__tabs { display: flex; margin-top: var(--fi-space-20); border-bottom: var(--fi-border-divider); }
.roundup__tab { flex: 1; display: flex; align-items: center; justify-content: center; gap: var(--fi-space-6); min-height: 88rpx; border-bottom: 6rpx solid transparent; color: var(--fi-color-text-secondary); font-size: var(--fi-font-30); }
.roundup__tab--active { border-bottom-color: var(--fi-color-primary); color: var(--fi-color-text-primary); font-weight: var(--fi-weight-extrabold); }
.roundup__tab-count { font-size: var(--fi-font-22); font-weight: var(--fi-weight-bold); color: var(--fi-color-text-caption); }
.roundup__tab--active .roundup__tab-count { color: var(--fi-color-primary); }
.roundup__empty { padding: var(--fi-space-40) 0; text-align: center; color: var(--fi-color-text-secondary); font-size: var(--fi-font-26); }
.roundup__entry { padding: var(--fi-space-28) 0 var(--fi-space-20); border-bottom: var(--fi-border-divider); }
.roundup__line { display: block; color: var(--fi-color-text-primary); line-height: var(--fi-leading-relaxed); }
.roundup__time { color: var(--fi-color-primary); font-weight: var(--fi-weight-bold); margin-right: var(--fi-space-8); }
.roundup__title { font-weight: var(--fi-weight-bold); }
.roundup__meta { display: flex; align-items: center; gap: var(--fi-space-16); margin-top: var(--fi-space-12); }
/* 三类各一个色：国内红、国际蓝、留洋橙；浅底深字保证对比度。 */
.roundup__tag { flex-shrink: 0; padding: var(--fi-space-2) var(--fi-space-14); border-radius: var(--fi-radius-xs); font-size: var(--fi-font-22); font-weight: var(--fi-weight-bold); }
.roundup__tag--domestic { background: #fdecec; color: #b3282a; }
.roundup__tag--world { background: #e8f0fd; color: #1f56b8; }
.roundup__tag--abroad { background: #fff0dc; color: #8f4700; }
.roundup__source-toggle { display: inline-flex; align-items: center; gap: var(--fi-space-8); padding: var(--fi-space-8) 0; color: var(--fi-color-text-caption); font-size: var(--fi-font-22); line-height: var(--fi-leading-normal); }
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
