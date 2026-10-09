<template>
  <view
    class="panel inventory-panel"
    :class="resolvePriceToneClass(section.price)"
  >
    <view class="inventory-panel__header">
      <view>
        <text class="inventory-panel__price">¥{{ section.price }}</text>
        <text class="inventory-panel__meta"
          >{{ section.region_count }} 个区域</text
        >
      </view>
      <view class="inventory-panel__actions">
        <text class="inventory-panel__summary">
          {{ section.available_region_count }} 区有回流 · 共
          {{ section.total_occurrences }} 张
        </text>
        <button
          class="collapse-pill collapse-pill--inventory"
          @tap="emit('toggle')"
        >
          {{ collapsed ? '展开' : '收起' }}
        </button>
      </view>
    </view>

    <view v-if="!collapsed" class="inventory-grid">
      <template
        v-for="item in section.items"
        :key="resolveInventoryBlockKey(item)"
      >
        <view v-if="readonly" class="inventory-cell" :class="cellClasses(item)">
          <text class="inventory-cell__name">{{ item.block_name }}</text>
          <text class="inventory-cell__count">{{
            item.has_inventory ? `${item.occurrences}张` : '--'
          }}</text>
          <text class="inventory-cell__time">{{
            item.has_inventory ? formatLatestTime(item.latest_time) : '无回流'
          }}</text>
          <view
            v-if="item.interested_user_count > 0"
            class="inventory-cell__interest-strip"
          >
            <view class="inventory-cell__interest-meter">
              <text
                v-for="barIndex in 4"
                :key="`history-interest-bar-${resolveInventoryBlockKey(item)}-${barIndex}`"
                class="inventory-cell__interest-bar"
                :class="{
                  'inventory-cell__interest-bar--active':
                    barIndex <=
                    resolveBlockInterestHeatLevel(item.interested_user_count),
                }"
              />
            </view>
            <text class="inventory-cell__interest-label"
              >{{ item.interested_user_count }}位钓友</text
            >
          </view>
        </view>
        <button
          v-else
          class="inventory-cell"
          :disabled="isBusy(item)"
          hover-class="inventory-cell--pressed"
          :class="cellClasses(item)"
          @tap="emit('interest', item)"
        >
          <text class="inventory-cell__name">{{ item.block_name }}</text>
          <text class="inventory-cell__count">{{
            item.has_inventory ? `${item.occurrences}张` : '--'
          }}</text>
          <text class="inventory-cell__time">{{
            item.has_inventory ? formatLatestTime(item.latest_time) : '无回流'
          }}</text>
          <view
            v-if="item.interested_user_count > 0"
            class="inventory-cell__interest-strip"
          >
            <view class="inventory-cell__interest-meter">
              <text
                v-for="barIndex in 4"
                :key="`current-interest-bar-${resolveInventoryBlockKey(item)}-${barIndex}`"
                class="inventory-cell__interest-bar"
                :class="{
                  'inventory-cell__interest-bar--active':
                    barIndex <=
                    resolveBlockInterestHeatLevel(item.interested_user_count),
                }"
              />
            </view>
            <text class="inventory-cell__interest-label"
              >{{ item.interested_user_count }}位钓友</text
            >
          </view>
        </button>
      </template>
    </view>
  </view>
</template>

<script setup lang="ts">
import type {
  TicketWatchGroupedInventoryItem,
  TicketWatchGroupedInventorySection,
} from '../../../types/ticketWatch'

import {
  resolveBlockInterestHeatLevel,
  resolveInventoryHeatLevel,
} from '../helpers'
import {
  formatLatestTime,
  resolveInventoryBlockKey,
  resolvePriceToneClass,
} from '../presentation'
const props = defineProps<{
  section: TicketWatchGroupedInventorySection
  matchId: number
  collapsed: boolean
  readonly?: boolean
  busyBlocks?: Record<string, boolean>
}>()
const emit = defineEmits<{
  (e: 'toggle'): void
  (e: 'interest', item: TicketWatchGroupedInventoryItem): void
}>()
function isBusy(item: TicketWatchGroupedInventoryItem): boolean {
  return Boolean(
    props.busyBlocks?.[`${props.matchId}:${resolveInventoryBlockKey(item)}`],
  )
}
function cellClasses(item: TicketWatchGroupedInventoryItem) {
  const max = props.section.items.reduce(
    (value, entry) => Math.max(value, entry.occurrences),
    0,
  )
  const heat = resolveInventoryHeatLevel(item.occurrences, max)
  return [
    resolvePriceToneClass(props.section.price),
    {
      'inventory-cell--active': item.occurrences > 0,
      'inventory-cell--heat-1': heat === 1,
      'inventory-cell--heat-2': heat === 2,
      'inventory-cell--heat-3': heat === 3,
      'inventory-cell--heat-4': heat === 4,
      'inventory-cell--wanted': item.viewer_interested,
      'inventory-cell--busy': isBusy(item),
      'inventory-cell--readonly': props.readonly,
    },
  ]
}
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

.inventory-panel__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--fi-space-16);
}

.inventory-panel__actions {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: var(--fi-space-12);
}

.inventory-panel__price {
  display: block;
  color: var(--fi-component-ticket-tone-text);
  font-size: var(--fi-font-44);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-none);
}

.inventory-panel__meta,
.inventory-panel__summary {
  display: block;
  margin-top: var(--fi-space-6);
  color: var(--fi-color-ticket-inventory-panel-meta-color);
  font-size: var(--fi-font-24);
}

.inventory-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: var(--fi-space-10);
  margin-top: var(--fi-space-14);
}

.inventory-cell {
  position: relative;
  overflow: hidden;
  border-radius: var(--fi-radius-sm);
  background: var(--fi-color-ticket-inventory-cell-background);
  padding: var(--fi-space-18) var(--fi-space-10) var(--fi-space-14);
  min-height: 138rpx;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  gap: var(--fi-space-6);
  transition:
    transform 0.18s ease,
    box-shadow 0.18s ease,
    border-color 0.18s ease;
}

.inventory-cell::after {
  border: none;
}

.inventory-cell--pressed {
  transform: scale(0.98);
}

.inventory-cell--wanted {
  --fi-component-ticket-wanted-shadow: 0 var(--fi-space-16) var(--fi-space-30)
    rgba(var(--fi-component-ticket-tone-rgb), 0.16);
  box-shadow: var(--fi-component-ticket-wanted-shadow);
}

.inventory-cell--busy {
  opacity: 0.72;
}

.inventory-cell--readonly {
  cursor: default;
}

.inventory-cell--active {
  background: linear-gradient(
    180deg,
    rgba(var(--fi-component-ticket-tone-rgb), 0.16),
    rgba(var(--fi-component-ticket-tone-rgb), 0.07)
  );
  border: var(--fi-border-width) solid
    rgba(var(--fi-component-ticket-tone-rgb), 0.28);
}

.inventory-cell--heat-1 {
  background: linear-gradient(
    180deg,
    rgba(var(--fi-component-ticket-tone-rgb), 0.12),
    var(--fi-color-ticket-history-trend-summary-item-background-2)
  );
  border: var(--fi-border-width) solid
    rgba(var(--fi-component-ticket-tone-rgb), 0.32);
}

.inventory-cell--heat-2 {
  background: linear-gradient(
    180deg,
    rgba(var(--fi-component-ticket-tone-rgb), 0.22),
    var(--fi-color-ticket-history-trend-summary-item-background-2)
  );
  border: var(--fi-border-width) solid
    rgba(var(--fi-component-ticket-tone-rgb), 0.4);
}

.inventory-cell--heat-3 {
  background: linear-gradient(
    180deg,
    rgba(var(--fi-component-ticket-tone-rgb), 0.34),
    var(--fi-color-ticket-history-trend-summary-item-background-2)
  );
  border: var(--fi-border-width) solid
    rgba(var(--fi-component-ticket-tone-strong-rgb), 0.48);
}

.inventory-cell--heat-4 {
  background: linear-gradient(
    180deg,
    rgba(var(--fi-component-ticket-tone-rgb), 0.48),
    var(--fi-color-ticket-history-trend-summary-item-background-2)
  );
  border: var(--fi-border-width) solid
    rgba(var(--fi-component-ticket-tone-strong-rgb), 0.62);
  box-shadow: var(--fi-shadow-ticket-inventory-cell-heat-4);
}

.inventory-cell__name {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-34);
  font-weight: var(--fi-weight-normal);
  line-height: var(--fi-leading-none);
  position: relative;
  z-index: 1;
}

.inventory-cell__count {
  color: var(--fi-component-ticket-tone-text);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
  position: relative;
  z-index: 1;
}

.inventory-cell__time {
  color: var(--fi-color-ticket-inventory-cell-time-color);
  font-size: var(--fi-font-20);
  position: relative;
  z-index: 1;
}

.inventory-cell__interest-strip {
  margin-top: auto;
  width: 100%;
  min-height: 34rpx;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: flex-end;
  gap: var(--fi-space-6);
  padding: 0 var(--fi-space-6);
  position: relative;
  z-index: 1;
}

.inventory-cell__interest-meter {
  display: flex;
  align-items: center;
  gap: var(--fi-space-4);
}

.inventory-cell__interest-bar {
  width: 12rpx;
  height: 5rpx;
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-ticket-inventory-cell-interest-bar-background);
}

.inventory-cell__interest-bar--active {
  background: var(
    --fi-color-ticket-inventory-cell-interest-bar-active-background
  );
  box-shadow: var(--fi-shadow-ticket-inventory-cell-interest-bar-active);
}

.inventory-cell__interest-label {
  color: var(--fi-color-ticket-inventory-cell-interest-bar-active-background);
  font-size: var(--fi-font-16);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}
</style>
