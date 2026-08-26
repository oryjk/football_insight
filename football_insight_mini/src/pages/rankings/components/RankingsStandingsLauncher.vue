<template>
  <view class="panel standings-launcher">
    <image class="standings-launcher__dots" :src="memberCardDotsImage" mode="aspectFill" />
    <view class="standings-launcher__grid">
      <view
        v-for="table in tables"
        :key="table.slug"
        class="standings-launcher-card"
        @click="emit('open', table.slug)"
      >
        <view class="standings-launcher-card__header">
          <view>
            <text class="section-kicker">{{ table.label }}</text>
            <text class="standings-launcher-card__title">查看完整图片</text>
          </view>
          <text class="meta-pill">查看</text>
        </view>

        <text class="standings-launcher-card__summary">{{ summary(table) }}</text>

        <view class="standings-launcher-card__footer">
          <text>{{ table.entries.length }} 支球队</text>
          <text class="standings-launcher-card__action">打开图片</text>
        </view>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import memberCardDotsImage from '../../../static/user/member-card-dots.png'
import type { StandingsTable } from '../../../types/insight'
import { buildStandingsPreviewSummary } from '../helpers'

defineProps<{
  tables: StandingsTable[]
}>()

const emit = defineEmits<{
  (e: 'open', slug: string): void
}>()

function summary(table: StandingsTable): string {
  return buildStandingsPreviewSummary(table)
}
</script>

<style scoped lang="css">
.panel {
  background: rgba(255, 255, 255, 0.94);
  border-radius: var(--fi-radius-xl);
  padding: var(--fi-space-20);
  border: var(--fi-border-card);
  box-shadow: var(--fi-shadow-card);
}

.section-kicker {
  margin: 0;
  color: var(--fi-component-kicker-text);
  font-size: var(--fi-component-kicker-size);
  font-weight: var(--fi-weight-bold);
  letter-spacing: 3rpx;
}

.meta-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  white-space: nowrap;
  line-height: var(--fi-leading-none);
  box-sizing: border-box;
  padding: var(--fi-space-14) var(--fi-space-24);
  border-radius: var(--fi-radius-round);
  border: var(--fi-primitive-border-width) solid var(--fi-color-border-chip);
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
}

.standings-launcher {
  position: relative;
  overflow: hidden;
  display: grid;
  gap: 0;
  padding-top: 4rpx;
  padding-bottom: var(--fi-space-8);
}

.standings-launcher__dots {
  position: absolute;
  z-index: 0;
  top: -54rpx;
  right: -54rpx;
  width: 292rpx;
  height: 180rpx;
  opacity: 0.34;
  pointer-events: none;
}

.standings-launcher__grid {
  position: relative;
  z-index: 1;
  display: grid;
  gap: 0;
  border-top: var(--fi-border-divider);
}

.standings-launcher-card {
  position: relative;
  display: grid;
  gap: var(--fi-space-8);
  padding: var(--fi-space-22) 0;
  border-bottom: var(--fi-border-divider);
  background: transparent;
}

.standings-launcher-card__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-12);
}

.standings-launcher-card__title {
  display: block;
  margin-top: var(--fi-space-8);
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-32);
  line-height: 1.18;
  font-weight: var(--fi-weight-extrabold);
}

.standings-launcher-card__summary {
  display: block;
  margin-top: 0;
  color: #747986;
  font-size: var(--fi-font-24);
  line-height: 1.5;
}

.standings-launcher-card__footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 4rpx;
  color: #9a9ea8;
  font-size: var(--fi-font-22);
}

.standings-launcher-card__action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-none);
  white-space: nowrap;
}
</style>
