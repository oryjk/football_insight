<template>
  <!-- 话题的热度与阅读时长：火焰 + 热度 N + 约 N 分钟读完，一行排开。
       整行都在组件内部：若只把热度包成组件、阅读时长留在外面，小程序的组件宿主节点会按文字基线排，两边对不齐。 -->
  <view class="topic-meta" :class="`topic-meta--${tone}`">
    <image class="topic-meta__icon" :src="fireIcon" mode="aspectFit" aria-hidden="true" />
    <text class="topic-meta__heat">热度 {{ formatTopicHeat(heat) }}</text>
    <text class="topic-meta__minutes">约 {{ minutes }} 分钟读完</text>
  </view>
</template>
<script setup lang="ts">
import fireIcon from '../../static/home/topic-heat-flame.svg'
import { formatTopicHeat } from '../../utils/dailyTopics'
withDefaults(defineProps<{ heat: number; minutes: number; tone?: 'light' | 'dark' }>(), { tone: 'light' })
</script>
<style scoped>
/* 图标、热度、阅读时长同字号同行高，在一行 flex 里居中对齐；放不下时只截断阅读时长。 */
.topic-meta { display: flex; align-items: center; min-width: 0; font-size: var(--fi-font-22); line-height: var(--fi-leading-normal); white-space: nowrap; overflow: hidden; }
.topic-meta--light { color: var(--fi-color-text-caption); }
.topic-meta--dark { color: var(--fi-color-topic-hero-muted); }
.topic-meta__icon { flex-shrink: 0; width: var(--fi-space-24); height: var(--fi-space-24); margin-right: var(--fi-space-6); }
.topic-meta__heat { flex-shrink: 0; }
.topic-meta__minutes { min-width: 0; margin-left: var(--fi-space-16); overflow: hidden; text-overflow: ellipsis; }
</style>
