<template>
  <view class="panel">
    <view class="section-heading">
      <view>
        <text class="section-kicker">{{ kicker }}</text>
        <text class="section-title">{{ title }}</text>
      </view>
    </view>

    <view class="contribution-list">
      <view v-for="row in visibleRows" :key="row.instanceKey" class="contribution-row">
        <view class="contribution-row__header">
          <view class="contribution-row__identity">
            <image
              :src="row.avatar || ''"
              :mode="row.avatarMode === 'fill' ? 'aspectFill' : 'aspectFit'"
              class="contribution-row__avatar"
              :class="{ 'contribution-row__avatar--player': row.avatarMode === 'fill' }"
            />
            <view>
              <text class="contribution-row__name">{{ row.name }}</text>
              <text class="contribution-row__note">{{ row.note }}</text>
            </view>
          </view>
          <text class="contribution-row__share">{{ formatShare(row.share) }}</text>
        </view>
        <view class="contribution-row__bar" :class="{ 'contribution-row__bar--danger': row.variant === 'danger' }">
          <view
            class="contribution-row__fill"
            :class="`contribution-row__fill--${row.variant}`"
            :style="{ width: barWidth(row.share) }"
          />
        </view>
      </view>
    </view>

    <button v-if="rows.length > 3" class="contribution-toggle" @click="emit('toggle')">
      {{ expanded ? '收起' : '更多' }}
    </button>
  </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import {
  formatInsightShare,
  getVisibleInsightContributions,
  insightBarWidth,
  type InsightContributionRow,
} from '../helpers'

const props = defineProps<{
  kicker: string
  title: string
  rows: InsightContributionRow[]
  expanded: boolean
}>()

const emit = defineEmits<{
  (e: 'toggle'): void
}>()

const visibleRows = computed(() => getVisibleInsightContributions(props.rows, props.expanded))

function formatShare(value: number): string {
  return formatInsightShare(value)
}

function barWidth(value: number): string {
  return insightBarWidth(value)
}
</script>

<style scoped lang="css">
.panel {
  position: relative;
  background: rgba(255, 255, 255, 0.94);
  border-radius: var(--fi-radius-xl);
  border: var(--fi-border-card);
  box-shadow: var(--fi-shadow-card);
  padding: var(--fi-space-20);
}

.section-heading {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--fi-space-12);
}

.section-kicker {
  margin: 0;
  color: var(--fi-component-kicker-text);
  font-size: var(--fi-component-kicker-size);
  font-weight: var(--fi-weight-bold);
  letter-spacing: 3rpx;
}

.section-title {
  display: block;
  margin-top: var(--fi-space-10);
  color: var(--fi-color-text-strong);
  font-size: var(--fi-component-title-size);
  line-height: 1.08;
  font-weight: var(--fi-weight-extrabold);
}

.contribution-list {
  margin-top: var(--fi-space-14);
  display: grid;
  gap: var(--fi-space-12);
}

.contribution-row {
  display: grid;
  gap: var(--fi-space-10);
  animation: fi-rise-in 220ms ease both;
}

.contribution-row__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.contribution-row__identity {
  display: flex;
  align-items: center;
  gap: var(--fi-space-12);
}

.contribution-row__avatar {
  width: 48rpx;
  height: 48rpx;
}

.contribution-row__avatar--player {
  border-radius: var(--fi-radius-round);
}

.contribution-row__name {
  display: block;
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
}

.contribution-row__note,
.contribution-row__share {
  color: #767a84;
  font-size: var(--fi-font-22);
}

.contribution-row__bar {
  height: 14rpx;
  border-radius: var(--fi-radius-round);
  background: #edf0f5;
  overflow: hidden;
}

.contribution-row__fill {
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(90deg, var(--fi-primitive-ink), #515563);
  transition: width 320ms cubic-bezier(0.22, 1, 0.36, 1);
}

.contribution-row__fill--red {
  background: linear-gradient(90deg, var(--fi-color-primary-deep), #ef4444);
}

.contribution-row__fill--green {
  background: linear-gradient(90deg, #0dbd73, #59df9c);
}

.contribution-row__fill--danger {
  background: linear-gradient(90deg, #ef4444, #fb7185);
}

.contribution-toggle {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin-top: var(--fi-space-18);
  align-self: flex-start;
  padding: var(--fi-space-12) var(--fi-space-22);
  border-radius: var(--fi-radius-round);
  background: rgba(var(--fi-primitive-red-rgb), 0.12);
  color: var(--fi-color-primary);
  font-size: var(--fi-font-24);
  white-space: nowrap;
  line-height: var(--fi-leading-none);
}
</style>
