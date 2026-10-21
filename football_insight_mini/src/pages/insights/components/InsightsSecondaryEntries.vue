<template>
  <view class="secondary-entries">
    <view class="secondary-entry secondary-entry--tactics" role="button" aria-label="排阵容" hover-class="secondary-entry--pressed" :hover-stay-time="80" @tap="emit('open', 'tactics')">
      <image :src="lineupIcon" class="secondary-entry__mark" mode="aspectFit" />
      <view class="secondary-entry__icon"><image :src="lineupIcon" class="secondary-entry__icon-image" mode="aspectFit" /></view>
      <view class="secondary-entry__copy">
        <text class="secondary-entry__title">排阵容</text>
        <text class="secondary-entry__caption">看大家怎么排</text>
      </view>
    </view>
    <view class="secondary-entry secondary-entry--transfer" role="button" aria-label="转会表态" hover-class="secondary-entry--pressed" :hover-stay-time="80" @tap="emit('open', 'transfer')">
      <image :src="transferIcon" class="secondary-entry__mark" mode="aspectFit" />
      <view class="secondary-entry__icon"><image :src="transferIcon" class="secondary-entry__icon-image" mode="aspectFit" /></view>
      <view class="secondary-entry__copy">
        <text class="secondary-entry__title">转会表态</text>
        <text class="secondary-entry__caption">送走谁、买回谁</text>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import lineupIcon from '../../../static/insights/entry-lineup.svg'
import transferIcon from '../../../static/insights/entry-transfer.svg'
const emit = defineEmits<{ (e: 'open', target: 'tactics' | 'transfer'): void }>()
</script>

<style scoped lang="css">
.secondary-entries { display: flex; flex-direction: row; gap: var(--fi-space-16); }
.secondary-entry {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: var(--fi-space-16);
  box-sizing: border-box;
  padding: var(--fi-space-24) var(--fi-space-16);
  border-radius: var(--fi-radius-lg);
  position: relative;
  overflow: hidden;
  border: var(--fi-border-card);
}
.secondary-entry--tactics { background: linear-gradient(135deg, #fff1ee 0%, #ffffff 70%); }
.secondary-entry--transfer { background: linear-gradient(135deg, #eef3ff 0%, #ffffff 70%); }
.secondary-entry--transfer .secondary-entry__icon { background: #e3ecff; }
/* 右下角放大的淡色图标作底纹，不抢文字。 */
.secondary-entry__mark { position: absolute; right: -16rpx; bottom: -24rpx; width: 128rpx; height: 128rpx; opacity: 0.1; pointer-events: none; }
.secondary-entry__icon, .secondary-entry__copy { position: relative; }
.secondary-entry__icon { flex-shrink: 0; display: flex; align-items: center; justify-content: center; width: var(--fi-space-64); height: var(--fi-space-64); border-radius: var(--fi-radius-insights-icon); background: var(--fi-color-insights-entry-icon); }
.secondary-entry__icon-image { width: var(--fi-space-36); height: var(--fi-space-36); }
.secondary-entry__copy { flex: 1; min-width: 0; }
.secondary-entry--pressed { opacity: var(--fi-opacity-insights-pressed); }
.secondary-entry__title { display: block; color: var(--fi-color-text-strong); font-size: var(--fi-font-32); line-height: var(--fi-leading-snug); font-weight: var(--fi-weight-extrabold); }
.secondary-entry__caption { display: block; margin-top: var(--fi-space-8); color: var(--fi-color-text-muted); font-size: var(--fi-font-24); line-height: var(--fi-leading-normal); }
</style>
