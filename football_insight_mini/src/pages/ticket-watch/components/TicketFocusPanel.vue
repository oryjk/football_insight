<template>
  <view class="panel focus-panel">
    <view class="section-heading section-heading--compact">
      <view class="focus-panel__heading">
        <view class="focus-panel__headline">
          <image
            class="focus-panel__headline-icon"
            :src="fireIcon"
            mode="aspectFit"
          />
          <text class="focus-panel__headline-title">热区速览</text>
          <text class="focus-panel__headline-subtitle">先看双榜热区</text>
        </view>
      </view>
      <view class="section-heading__actions">
        <text class="meta-pill">
          <image
            class="inline-image-icon inline-image-icon--small"
            :src="swapArrowsIcon"
            mode="aspectFit"
          />
          <text>{{ switching ? '切换中' : '回流 + 钓友' }}</text>
        </text>
        <button class="collapse-pill" @tap="emit('toggle')">
          {{ collapsed ? '展开' : '收起' }}
        </button>
      </view>
    </view>

    <template v-if="!collapsed">
      <view v-if="focusBlocks.length" class="focus-section">
        <view class="focus-section__head">
          <view class="focus-section__title-row">
            <image
              class="focus-section__title-icon-image"
              :src="refluxBarsIcon"
              mode="aspectFit"
            />
            <text class="focus-section__title">回流热区</text>
          </view>
          <text class="focus-section__meta">{{
            history ? '复盘先看这 3 个区' : '先刷这 3 个区'
          }}</text>
        </view>
        <view class="focus-grid focus-grid--triple">
          <view
            v-for="block in focusBlocks"
            :key="`focus-${block.block_key}`"
            class="focus-chip focus-chip--compact"
            :class="resolvePriceToneClass(block.price)"
          >
            <view class="focus-chip__name-row">
              <text class="focus-chip__name">{{ block.block_name }}</text>
              <image
                class="focus-chip__name-icon-image"
                :src="fireIcon"
                mode="aspectFit"
              />
            </view>
            <text class="focus-chip__meta">¥{{ block.price }}</text>
            <text class="focus-chip__count">{{ block.occurrences }} 张</text>
          </view>
        </view>
      </view>

      <view v-if="interestBlocks.length" class="focus-section">
        <view class="focus-section__head">
          <view class="focus-section__title-row">
            <image
              class="focus-section__title-icon-image"
              :src="friendUserIcon"
              mode="aspectFit"
            />
            <text class="focus-section__title">钓友钓区</text>
          </view>
          <text class="focus-section__meta">{{
            history ? '这场大家更想蹲的 3 个区' : '大家更想蹲这 3 个区'
          }}</text>
        </view>
        <view class="focus-grid focus-grid--triple">
          <view
            v-for="block in interestBlocks"
            :key="`interest-focus-${block.block_key}`"
            class="focus-chip focus-chip--compact focus-chip--interest"
            :class="resolvePriceToneClass(block.price)"
          >
            <view class="focus-chip__name-row">
              <text class="focus-chip__name">{{ block.block_name }}</text>
              <image
                class="focus-chip__name-icon-image"
                :src="fireIcon"
                mode="aspectFit"
              />
            </view>
            <text class="focus-chip__meta">¥{{ block.price }}</text>
            <text class="focus-chip__count focus-chip__count--interest"
              >{{ block.interested_user_count }} 位钓友</text
            >
          </view>
        </view>
      </view>

      <text class="focus-copy"
        >先扫回流前三，再看钓友都蹲哪几个区，{{
          history ? '下面再看完整价位分布。' : '下面再看完整分区看板。'
        }}</text
      >
    </template>
  </view>
</template>

<script setup lang="ts">
import type {
  TicketWatchBoardTopBlock,
  TicketWatchBoardTopInterestBlock,
} from '../helpers'
import fireIcon from '../../../static/ticket-watch/fire.svg'
import friendUserIcon from '../../../static/ticket-watch/friend-user.svg'
import swapArrowsIcon from '../../../static/ticket-watch/swap-arrows.svg'
import refluxBarsIcon from '../../../static/ticket-watch/reflux-bars.svg'
import { resolvePriceToneClass } from '../presentation'
defineProps<{
  focusBlocks: TicketWatchBoardTopBlock[]
  interestBlocks: TicketWatchBoardTopInterestBlock[]
  collapsed: boolean
  history?: boolean
  switching?: boolean
}>()
const emit = defineEmits<{ (e: 'toggle'): void }>()
</script>

<style scoped lang="css">
@import '../ticket-watch-shared.css';

.price-tone--neutral {
  --fi-component-ticket-tone-rgb: var(
    --fi-color-ticket-price-tone-neutral-tone-rgb
  );
  --fi-component-ticket-tone-strong-rgb: var(
    --fi-color-ticket-price-tone-neutral-tone-strong-rgb
  );
  --fi-component-ticket-tone-text: var(
    --fi-color-ticket-price-tone-neutral-tone-text
  );
}

.price-tone--vip {
  --fi-component-ticket-tone-rgb: var(
    --fi-color-ticket-price-tone-vip-tone-rgb
  );
  --fi-component-ticket-tone-strong-rgb: var(
    --fi-color-ticket-price-tone-vip-tone-strong-rgb
  );
  --fi-component-ticket-tone-text: var(
    --fi-color-ticket-price-tone-vip-tone-text
  );
}

.price-tone--s {
  --fi-component-ticket-tone-rgb: var(--fi-color-ticket-price-tone-s-tone-rgb);
  --fi-component-ticket-tone-strong-rgb: var(
    --fi-color-ticket-price-tone-s-tone-strong-rgb
  );
  --fi-component-ticket-tone-text: var(
    --fi-color-ticket-price-tone-s-tone-text
  );
}

.price-tone--a {
  --fi-component-ticket-tone-rgb: var(--fi-color-ticket-price-tone-a-tone-rgb);
  --fi-component-ticket-tone-strong-rgb: var(
    --fi-color-ticket-price-tone-a-tone-strong-rgb
  );
  --fi-component-ticket-tone-text: var(
    --fi-color-ticket-price-tone-a-tone-text
  );
}

.price-tone--b {
  --fi-component-ticket-tone-rgb: var(--fi-color-ticket-price-tone-b-tone-rgb);
  --fi-component-ticket-tone-strong-rgb: var(
    --fi-color-ticket-price-tone-b-tone-strong-rgb
  );
  --fi-component-ticket-tone-text: var(
    --fi-color-ticket-price-tone-b-tone-text
  );
}

.price-tone--c {
  --fi-component-ticket-tone-rgb: var(--fi-color-ticket-price-tone-c-tone-rgb);
  --fi-component-ticket-tone-strong-rgb: var(
    --fi-color-ticket-price-tone-c-tone-strong-rgb
  );
  --fi-component-ticket-tone-text: var(
    --fi-color-ticket-price-tone-c-tone-text
  );
}

.price-tone--d {
  --fi-component-ticket-tone-rgb: var(--fi-color-ticket-price-tone-d-tone-rgb);
  --fi-component-ticket-tone-strong-rgb: var(
    --fi-color-ticket-price-tone-d-tone-strong-rgb
  );
  --fi-component-ticket-tone-text: var(
    --fi-color-ticket-price-tone-d-tone-text
  );
}

.price-tone--e {
  --fi-component-ticket-tone-rgb: var(--fi-color-ticket-price-tone-e-tone-rgb);
  --fi-component-ticket-tone-strong-rgb: var(
    --fi-color-ticket-price-tone-e-tone-strong-rgb
  );
  --fi-component-ticket-tone-text: var(
    --fi-color-ticket-price-tone-e-tone-text
  );
}

.focus-panel {
  background: linear-gradient(
    180deg,
    var(--fi-color-ticket-focus-panel-background),
    var(--fi-color-ticket-history-trend-summary-item-background-2)
  );
}

.focus-panel__heading {
  min-width: 0;
}

.focus-panel__headline {
  display: flex;
  align-items: center;
  gap: var(--fi-space-10);
  min-width: 0;
}

.focus-panel__headline-icon {
  width: 32rpx;
  height: 32rpx;
  flex-shrink: 0;
}

.focus-panel__headline-title {
  color: var(--fi-color-ticket-focus-panel-headline-title-color);
  font-size: var(--fi-font-34);
  font-weight: var(--fi-weight-normal);
  line-height: var(--fi-leading-tight);
  flex-shrink: 0;
}

.focus-panel__headline-subtitle {
  color: var(--fi-color-ticket-focus-panel-headline-subtitle-color);
  font-size: var(--fi-font-20);
  line-height: var(--fi-leading-snug);
  white-space: nowrap;
}

.focus-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--fi-space-10);
  margin-top: var(--fi-space-14);
}

.focus-grid--triple {
  grid-template-columns: repeat(3, minmax(0, 1fr));
  margin-top: 0;
}

.focus-section {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-12);
}

.focus-section + .focus-section {
  margin-top: var(--fi-space-16);
}

.focus-section__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-16);
}

.focus-section__title-row {
  display: flex;
  align-items: center;
  gap: var(--fi-space-8);
}

.focus-section__title-icon-image {
  width: 22rpx;
  height: 22rpx;
  flex-shrink: 0;
}

.focus-section__title {
  color: var(--fi-color-ticket-focus-panel-headline-title-color);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-normal);
}

.focus-section__meta {
  color: var(--fi-color-ticket-focus-section-meta-color);
  font-size: var(--fi-font-20);
}

.focus-chip {
  position: relative;
  overflow: hidden;
  border-radius: var(--fi-radius-ticket-20);
  padding: var(--fi-space-18) var(--fi-space-16);
  background: linear-gradient(
    180deg,
    rgba(var(--fi-component-ticket-tone-rgb), 0.08),
    rgba(var(--fi-component-ticket-tone-rgb), 0.02)
  );
  border: var(--fi-border-width) solid
    rgba(var(--fi-component-ticket-tone-rgb), 0.14);
  box-shadow: var(--fi-shadow-ticket-focus-chip);
}

.focus-chip--compact {
  min-height: 118rpx;
  padding: var(--fi-space-16) var(--fi-space-14);
  border-radius: var(--fi-radius-sm);
}

.focus-chip__name-row {
  display: flex;
  align-items: center;
  gap: var(--fi-space-6);
}

.focus-chip__name {
  display: block;
  color: var(--fi-component-ticket-tone-text);
  font-size: var(--fi-font-30);
  font-weight: var(--fi-weight-normal);
  line-height: var(--fi-leading-none);
}

.focus-chip__name-icon-image {
  width: 16rpx;
  height: 16rpx;
  flex-shrink: 0;
}

.focus-chip--compact .focus-chip__name {
  font-size: var(--fi-font-28);
}

.focus-chip__meta {
  display: block;
  margin-top: var(--fi-space-8);
  color: rgba(var(--fi-component-ticket-tone-strong-rgb), 0.7);
  font-size: var(--fi-font-18);
  font-weight: var(--fi-weight-bold);
}

.focus-chip--compact .focus-chip__meta {
  font-size: var(--fi-font-22);
}

.focus-chip__count {
  display: block;
  margin-top: var(--fi-space-8);
  color: var(--fi-component-ticket-tone-text);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-extrabold);
}

.focus-chip--compact .focus-chip__count {
  font-size: var(--fi-font-28);
}

.focus-chip__count--interest {
  color: var(--fi-component-ticket-tone-text);
}

.focus-copy {
  display: block;
  margin-top: var(--fi-space-14);
  color: var(--fi-color-ticket-focus-copy-color);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-normal);
}
</style>
