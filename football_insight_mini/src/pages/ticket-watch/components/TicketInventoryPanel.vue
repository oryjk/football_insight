<template>
  <view
    class="panel inventory-panel"
    :class="resolvePriceToneClass(section.price)"
  >
    <view class="inventory-panel__header">
      <view class="inventory-panel__title">
        <text class="inventory-panel__price">¥{{ section.price }}</text>
        <text class="inventory-panel__meta">{{ section.region_count }} 个区域</text>
      </view>
      <view class="inventory-panel__actions">
        <view class="inventory-panel__summary">
          <text class="inventory-panel__summary-main">{{ section.total_occurrences }} 张回流</text>
          <text class="inventory-panel__meta">{{ section.available_region_count }} 区有回流</text>
        </view>
        <button
          class="collapse-pill collapse-pill--inventory"
          @tap="emit('toggle')"
        >
          {{ collapsed ? '展开' : '收起' }}
        </button>
      </view>
    </view>

    <FiCollapse :open="!collapsed">
      <view class="inventory-grid">
        <view
          v-for="item in section.items"
          :key="resolveInventoryBlockKey(item)"
          class="inventory-cell"
          :class="cellClasses(item)"
          :hover-class="readonly ? 'none' : 'inventory-cell--pressed'"
          @tap="handleCellTap(item)"
        >
          <image
            v-if="item.viewer_interested"
            class="inventory-cell__flag"
            :src="starIcon"
            mode="aspectFit"
          />
          <view class="inventory-cell__main">
            <text class="inventory-cell__name">{{ item.block_name }}</text>
            <text
              class="inventory-cell__count"
              :class="{ 'inventory-cell__count--empty': !item.has_inventory }"
              >{{ item.has_inventory ? `${item.occurrences} 张` : '—' }}</text
            >
            <text v-if="item.has_inventory" class="inventory-cell__time">{{
              formatLatestTime(item.latest_time)
            }}</text>
          </view>
          <view
            v-if="item.interested_user_count > 0"
            class="inventory-cell__interest"
            :class="{
              'inventory-cell__interest--hot':
                resolveBlockInterestHeatLevel(item.interested_user_count) >= 3,
            }"
          >
            <text>{{ item.interested_user_count }} 位钓友</text>
          </view>
        </view>
      </view>
    </FiCollapse>
  </view>
</template>

<script setup lang="ts">
import FiCollapse from '../../../components/FiCollapse.vue'
import starIcon from '../../../static/ticket-watch/lucide-star.svg'
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
function handleCellTap(item: TicketWatchGroupedInventoryItem): void {
  if (props.readonly || isBusy(item)) return
  emit('interest', item)
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
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-16);
}

.inventory-panel__title,
.inventory-panel__summary {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-6);
}

.inventory-panel__summary {
  align-items: flex-end;
}

.inventory-panel__actions {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: var(--fi-space-16);
}

.inventory-panel__price {
  display: block;
  color: var(--fi-component-ticket-tone-text);
  font-size: var(--fi-font-44);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-none);
}

.inventory-panel__meta {
  display: block;
  color: var(--fi-color-ticket-inventory-panel-meta-color);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-none);
}

.inventory-panel__summary-main {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-none);
}

.inventory-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: var(--fi-space-10);
  margin-top: var(--fi-space-16);
}

/* 固定高度、内容自上而下排：名称 / 张数 / 时间对齐，钓友胶囊固定贴底，不再随内容上下跳。 */
.inventory-cell {
  position: relative;
  box-sizing: border-box;
  overflow: hidden;
  height: calc(var(--fi-space-100) + var(--fi-space-72));
  padding: var(--fi-space-16) var(--fi-space-8) var(--fi-space-12);
  border-radius: var(--fi-radius-sm);
  background: var(--fi-color-ticket-inventory-cell-background);
  border: var(--fi-border-width) solid transparent;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: space-between;
  transition:
    transform 0.18s ease,
    box-shadow 0.18s ease,
    border-color 0.18s ease;
}

.inventory-cell--pressed {
  transform: scale(0.98);
}

.inventory-cell--busy {
  opacity: 0.72;
}

.inventory-cell--wanted {
  border-color: rgba(var(--fi-component-ticket-tone-strong-rgb), 0.72);
  box-shadow: 0 var(--fi-space-12) var(--fi-space-24)
    rgba(var(--fi-component-ticket-tone-rgb), 0.18);
}

.inventory-cell--active {
  background: rgba(var(--fi-component-ticket-tone-rgb), 0.16);
  border-color: rgba(var(--fi-component-ticket-tone-rgb), 0.28);
}

.inventory-cell--heat-1 {
  background: rgba(var(--fi-component-ticket-tone-rgb), 0.12);
  border-color: rgba(var(--fi-component-ticket-tone-rgb), 0.32);
}

.inventory-cell--heat-2 {
  background: rgba(var(--fi-component-ticket-tone-rgb), 0.22);
  border-color: rgba(var(--fi-component-ticket-tone-rgb), 0.4);
}

.inventory-cell--heat-3 {
  background: rgba(var(--fi-component-ticket-tone-rgb), 0.34);
  border-color: rgba(var(--fi-component-ticket-tone-strong-rgb), 0.48);
}

.inventory-cell--heat-4 {
  background: rgba(var(--fi-component-ticket-tone-rgb), 0.48);
  border-color: rgba(var(--fi-component-ticket-tone-strong-rgb), 0.62);
  box-shadow: var(--fi-shadow-ticket-inventory-cell-heat-4);
}

.inventory-cell--wanted.inventory-cell--heat-4,
.inventory-cell--wanted.inventory-cell--heat-3 {
  border-color: rgba(var(--fi-component-ticket-tone-strong-rgb), 0.9);
}

.inventory-cell__flag {
  position: absolute;
  top: var(--fi-space-8);
  right: var(--fi-space-8);
  width: var(--fi-icon-sm);
  height: var(--fi-icon-sm);
}

.inventory-cell__main {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--fi-space-8);
}

.inventory-cell__name {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-32);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-none);
}

.inventory-cell__count {
  color: var(--fi-component-ticket-tone-text);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

.inventory-cell__count--empty {
  color: var(--fi-color-text-muted);
  font-weight: var(--fi-weight-normal);
}

.inventory-cell__time {
  color: var(--fi-color-ticket-inventory-cell-time-color);
  font-size: var(--fi-font-20);
  line-height: var(--fi-leading-none);
}

.inventory-cell__interest {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--fi-space-4);
  max-width: 100%;
  padding: var(--fi-space-4) var(--fi-space-10);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-surface);
  color: var(--fi-color-ticket-inventory-cell-interest-bar-active-background);
  font-size: var(--fi-font-20);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
  white-space: nowrap;
}

.inventory-cell__interest--hot {
  background: var(--fi-color-ticket-inventory-cell-interest-bar-active-background);
  color: var(--fi-color-surface);
}

</style>
