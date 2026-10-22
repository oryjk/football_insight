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
          <text v-if="switching" class="focus-panel__headline-subtitle">切换中…</text>
        </view>
      </view>
      <view class="section-heading__actions">
        <button class="collapse-pill" @tap="emit('toggle')">
          {{ collapsed ? '展开' : '收起' }}
        </button>
      </view>
    </view>

    <FiCollapse :open="!collapsed">
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
            <text class="focus-chip__name">{{ block.block_name }}</text>
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
            <text class="focus-chip__name">{{ block.block_name }}</text>
            <text class="focus-chip__meta">¥{{ block.price }}</text>
            <text class="focus-chip__count focus-chip__count--interest"
              >{{ block.interested_user_count }} 位钓友</text
            >
          </view>
        </view>
      </view>

    </FiCollapse>
  </view>
</template>

<script setup lang="ts">
import FiCollapse from '../../../components/FiCollapse.vue'
import type {
  TicketWatchBoardTopBlock,
  TicketWatchBoardTopInterestBlock,
} from '../helpers'
import fireIcon from '../../../static/ticket-watch/lucide-flame.svg'
import friendUserIcon from '../../../static/ticket-watch/lucide-users.svg'
import refluxBarsIcon from '../../../static/ticket-watch/lucide-chart-column.svg'
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
  background: var(--fi-color-ticket-focus-panel-background);
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
  width: var(--fi-icon-sm);
  height: var(--fi-icon-sm);
  flex-shrink: 0;
}

.focus-panel__headline-title {
  color: var(--fi-color-ticket-focus-panel-headline-title-color);
  font-size: var(--fi-font-34);
  font-weight: var(--fi-weight-extrabold);
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
  width: var(--fi-icon-sm);
  height: var(--fi-icon-sm);
  flex-shrink: 0;
}

.focus-section__title {
  color: var(--fi-color-ticket-focus-panel-headline-title-color);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
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
  background: var(--fi-color-surface);
  border: var(--fi-border-card);
  box-shadow: var(--fi-shadow-ticket-focus-chip);
}

/* 价位只用左侧色带区分；底色与文字统一，避免多色底上文字看不清。 */
.focus-chip::before {
  content: '';
  position: absolute;
  left: 0;
  top: var(--fi-space-14);
  bottom: var(--fi-space-14);
  width: var(--fi-space-6);
  border-radius: 0 var(--fi-radius-round) var(--fi-radius-round) 0;
  background: rgba(var(--fi-component-ticket-tone-strong-rgb), 0.85);
}

.focus-chip--compact {
  min-height: calc(var(--fi-space-100) + var(--fi-space-18));
  padding: var(--fi-space-16) var(--fi-space-14) var(--fi-space-16) var(--fi-space-20);
  border-radius: var(--fi-radius-sm);
}


.focus-chip__name {
  display: block;
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-30);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-none);
}


.focus-chip--compact .focus-chip__name {
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-extrabold);
}

.focus-chip__meta {
  display: block;
  margin-top: var(--fi-space-8);
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-18);
  font-weight: var(--fi-weight-bold);
}

.focus-chip--compact .focus-chip__meta {
  font-size: var(--fi-font-22);
}

.focus-chip__count {
  display: block;
  margin-top: var(--fi-space-8);
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-extrabold);
}

.focus-chip--compact .focus-chip__count {
  font-size: var(--fi-font-28);
}


</style>
