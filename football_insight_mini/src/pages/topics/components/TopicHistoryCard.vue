<template>
  <view class="topic-row" :class="`topic-row--${presentation.tone}`" role="button" :aria-label="`阅读${item.title}`" hover-class="topic-row--pressed" @tap="emit('read', item.id)">
    <FiTopicCrest :item="item" variant="history" />
    <view class="topic-row__top">
      <text class="topic-row__badge">{{ presentation.label }}</text>
      <text class="topic-row__date">{{ item.publishedLabel }}</text>
    </view>
    <text class="topic-row__title">{{ item.title }}</text>
    <text class="topic-row__summary">{{ item.summary }}</text>
    <view class="topic-row__footer">
      <view class="topic-row__meta"><FiTopicMeta :heat="heat" :minutes="item.readMinutes" /></view>
      <view class="topic-row__read-entry"><FiTopicReadEntry @read="emit('read', item.id)" /></view>
    </view>
  </view>
</template>
<script setup lang="ts">
import { computed } from 'vue'
import FiTopicCrest from '../../../components/topics/FiTopicCrest.vue'
import FiTopicMeta from '../../../components/topics/FiTopicMeta.vue'
import FiTopicReadEntry from '../../../components/topics/FiTopicReadEntry.vue'
import type { DailyTopicItem } from '../../../types/dailyTopic'
import { topicCardPresentation } from '../presentation'
const props = defineProps<{ item: DailyTopicItem; heat: number }>()
const emit = defineEmits<{ (e: 'read', id: string): void }>()
const presentation = computed(() => topicCardPresentation(props.item.title))
</script>
<style scoped>
.topic-row { --topic-bg: var(--fi-color-topic-history-news-bg); --topic-accent: var(--fi-color-topic-history-news-accent); position: relative; overflow: hidden; padding: var(--fi-space-28);  background: linear-gradient(120deg, var(--topic-bg), var(--fi-color-bg)); border: var(--fi-border-card); border-radius: var(--fi-radius-lg); box-shadow: var(--fi-shadow-soft); }
.topic-row--championship { --topic-bg: var(--fi-color-topic-history-championship-bg); --topic-accent: var(--fi-color-topic-history-championship-accent); }
.topic-row--preview { --topic-bg: var(--fi-color-topic-history-preview-bg); --topic-accent: var(--fi-color-topic-history-preview-accent); }
.topic-row--analysis { --topic-bg: var(--fi-color-topic-history-analysis-bg); --topic-accent: var(--fi-color-topic-history-analysis-accent); }
.topic-row--pressed { opacity: 0.9; transform: scale(0.99); }
.topic-row__top, .topic-row__title, .topic-row__summary, .topic-row__footer { position: relative; z-index: 1; }
.topic-row__top { display: flex; justify-content: space-between; align-items: center; gap: var(--fi-space-12); }
.topic-row__badge { flex-shrink: 0; padding: var(--fi-space-6) var(--fi-space-14); border: var(--fi-border-width) solid currentColor; border-radius: var(--fi-radius-round); color: var(--topic-accent); background: var(--fi-color-bg); font-size: var(--fi-font-22); font-weight: var(--fi-weight-bold); }
.topic-row__date { min-width: 0; color: var(--fi-color-text-caption); font-size: var(--fi-font-20); text-align: right; }
.topic-row__title { display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; margin-top: var(--fi-space-20); color: var(--fi-color-text-primary); font-size: var(--fi-font-32); font-weight: var(--fi-weight-bold); line-height: var(--fi-leading-normal); }
.topic-row__summary { display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; margin-top: var(--fi-space-16); color: var(--fi-color-text-secondary); font-size: var(--fi-font-26); line-height: var(--fi-leading-relaxed); }
.topic-row__footer { display: flex; align-items: center; justify-content: space-between; gap: var(--fi-space-16); margin-top: var(--fi-space-24); padding-top: var(--fi-space-18); border-top: var(--fi-border-divider); }
.topic-row__meta { flex: 1; min-width: 0; display: flex; align-items: center; overflow: hidden; }
.topic-row__read-entry { flex-shrink: 0; margin-left: auto; }
</style>
