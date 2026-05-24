<template>
  <view class="stadium-map" :class="{ 'stadium-map--review': mode === 'review' }">
    <view v-if="hint" class="stadium-map__hint">{{ hint }}</view>
    <view class="stadium-map__field">
      <view class="stadium-price-legend" aria-hidden="true">
        <view
          v-for="item in priceLegendItems"
          :key="item.grade"
          class="stadium-price-legend__ticket"
          :class="`stadium-price-legend__ticket--${item.colorGroup}`"
        >
          <text class="stadium-price-legend__grade">{{ item.grade }}</text>
          <text class="stadium-price-legend__price">{{ item.price }}</text>
        </view>
      </view>
      <button
        v-for="region in renderedRegions"
        :key="region.key"
        class="stadium-region"
        :class="region.classes"
        :style="region.style"
        @tap="handleTap(region.key, region.disabled, region.unmapped)"
      >
        <text class="stadium-region__name">{{ region.name }}</text>
      </button>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { TicketWatchRegion } from '../types/ticketWatch'
import {
  canInteractSeatSwapMap,
  shouldDimSeatSwapRegion,
} from '../pages/seat-swap/helpers'
import {
  resolveSeatSwapRegionColorGroup,
  resolveSeatSwapRegionLayout,
} from '../utils/stadiumRegions'

type Mode =
  | 'browse'
  | 'filter'
  | 'published'
  | 'select-current'
  | 'select-desired'
  | 'review'

interface Props {
  mode: Mode
  regions: TicketWatchRegion[]
  badges?: Record<string, number>
  filterKey?: string
  currentKey?: string
  desiredKeys?: string[]
  stagedCurrentKey?: string
  stagedDesiredKeys?: string[]
  hint?: string
}

const props = withDefaults(defineProps<Props>(), {
  badges: () => ({}),
  filterKey: '',
  currentKey: '',
  desiredKeys: () => [],
  stagedCurrentKey: '',
  stagedDesiredKeys: () => [],
  hint: '',
})

const emit = defineEmits<{
  (e: 'region-tap', key: string): void
}>()

const hasStagedDesiredKeys = computed(() => props.stagedDesiredKeys.length > 0)
const priceLegendItems = [
  { grade: 'VIP', price: '1288元', colorGroup: 'vip' },
  { grade: 'S类', price: '400元', colorGroup: 'red' },
  { grade: 'A类', price: '220元', colorGroup: 'yellow' },
  { grade: 'B类', price: '180元', colorGroup: 'green' },
  { grade: 'C类', price: '150元', colorGroup: 'blue' },
  { grade: 'D类', price: '120元', colorGroup: 'navy' },
  { grade: 'E类', price: '100元', colorGroup: 'purple' },
] as const

function regionKey(region: TicketWatchRegion): string {
  return region.block_key || region.block_name
}

const renderedRegions = computed(() =>
  props.regions.map((region) => {
    const key = regionKey(region)
    const name = region.block_name
    const position = resolveSeatSwapRegionLayout(name) || resolveSeatSwapRegionLayout(key)
    const colorGroup = resolveSeatSwapRegionColorGroup(name)

    const isCurrent = !!props.currentKey && props.currentKey === key
    const isDesired = props.desiredKeys.includes(key)
    const isStagedCurrent = !!props.stagedCurrentKey && props.stagedCurrentKey === key
    const isStagedDesired = props.stagedDesiredKeys.includes(key)
    const isFilterMatched = !!props.filterKey && props.filterKey === key

    const mode = props.mode
    const isReview = mode === 'review'
    const isPublished = mode === 'published'
    const isFilter = mode === 'filter'
    const isSelectCurrent = mode === 'select-current'
    const isSelectDesired = mode === 'select-desired'

    const highlightCurrent =
      (isPublished && isCurrent) ||
      (isSelectCurrent && isStagedCurrent) ||
      (isSelectDesired && isStagedCurrent)
    const highlightDesired =
      (isPublished && isDesired) || (isSelectDesired && isStagedDesired)
    const highlightFilter = isFilter && isFilterMatched
    const reviewCurrent = isReview && isCurrent
    const reviewDesired = isReview && isDesired

    const dimmed = shouldDimSeatSwapRegion({
      mode,
      hasFilterKey: Boolean(props.filterKey),
      isFilterMatched,
      hasStagedCurrentKey: Boolean(props.stagedCurrentKey),
      isStagedCurrent,
      hasStagedDesiredKeys: hasStagedDesiredKeys.value,
      isStagedDesired,
      isCurrent,
      isDesired,
    })

    const disabled = isSelectDesired && isStagedCurrent

    const unmapped = !position

    return {
      key,
      name,
      style: position
        ? `left:${position.left}%;top:${position.top}%;width:${position.width}%;height:${position.height}%;`
        : '',
      disabled,
      unmapped,
      classes: [
        !isReview && `stadium-region--${colorGroup}`,
        {
          'stadium-region--unmapped': unmapped,
          'stadium-region--current': highlightCurrent,
          'stadium-region--desired': highlightDesired,
          'stadium-region--filter': highlightFilter,
          'stadium-region--review-muted': isReview && !isCurrent && !isDesired,
          'stadium-region--review-current': reviewCurrent,
          'stadium-region--review-desired': reviewDesired,
          'stadium-region--dimmed': dimmed,
          'stadium-region--disabled': disabled,
        },
      ],
    }
  }),
)

function handleTap(key: string, disabled: boolean, unmapped: boolean) {
  if (disabled || unmapped) return
  if (!canInteractSeatSwapMap(props.mode)) return
  emit('region-tap', key)
}
</script>

<style scoped>
.stadium-map {
  position: relative;
}

.stadium-map__hint {
  display: block;
  margin-bottom: 10rpx;
  text-align: center;
  color: #8f9198;
  font-size: 22rpx;
  font-weight: 400;
}

.stadium-map__field {
  position: relative;
  width: 100%;
  height: 520rpx;
  background: transparent;
  overflow: visible;
}

.stadium-region {
  position: absolute;
  z-index: 2;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 36rpx;
  border-radius: 12rpx;
  box-sizing: border-box;
  margin: 0;
  padding: 2rpx 3rpx;
  background-image: none;
  color: #17191f;
  font: inherit;
  line-height: 1.1;
  box-shadow: 0 8rpx 18rpx rgba(18, 25, 20, 0.18);
  transition: transform 0.16s ease, box-shadow 0.16s ease, opacity 0.16s ease, filter 0.16s ease;
}

.stadium-region::after {
  border: 0;
  display: none;
  content: none;
}

.stadium-region__name {
  position: relative;
  z-index: 1;
  display: block;
  max-width: 100%;
  overflow: hidden;
  font-size: 17rpx;
  font-weight: 900;
  text-align: center;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.stadium-region--unmapped {
  position: relative;
  left: auto !important;
  top: auto !important;
  display: none;
}

.stadium-region--blue { background: #336fbd; color: #fff; }
.stadium-region--green { background: #46ab59; color: #fff; }
.stadium-region--purple { background: #6c369b; color: #fff; }
.stadium-region--yellow { background: #f4c23a; }
.stadium-region--navy { background: #0f215e; color: #fff; }
.stadium-region--red { background: #ec3b20; color: #fff; }
.stadium-region--vip { background: #b90000; color: #fff; }
.stadium-region--muted { background: #d9dee7; }

.stadium-price-legend {
  position: absolute;
  left: 24%;
  right: 24%;
  top: 32%;
  z-index: 1;
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 7rpx 8rpx;
  pointer-events: none;
}

.stadium-price-legend__ticket {
  position: relative;
  display: flex;
  align-items: center;
  height: 34rpx;
  min-width: 106rpx;
  overflow: hidden;
  border-radius: 7rpx;
  color: #fff;
  box-shadow: 0 6rpx 12rpx rgba(18, 25, 20, 0.12);
}

.stadium-price-legend__ticket::before,
.stadium-price-legend__ticket::after {
  position: absolute;
  left: 36%;
  width: 9rpx;
  height: 9rpx;
  border-radius: 999rpx;
  background: #f3f2ef;
  content: '';
  transform: translateX(-50%);
}

.stadium-price-legend__ticket::before {
  top: -5rpx;
}

.stadium-price-legend__ticket::after {
  bottom: -5rpx;
}

.stadium-price-legend__grade {
  display: flex;
  width: 42rpx;
  height: 100%;
  align-items: center;
  justify-content: center;
  border-right: 1rpx solid rgba(255, 255, 255, 0.38);
  font-size: 18rpx;
  font-weight: 900;
}

.stadium-price-legend__price {
  flex: 1;
  padding: 0 11rpx;
  font-size: 18rpx;
  font-weight: 800;
  text-align: center;
  white-space: nowrap;
}

.stadium-price-legend__ticket--blue { background: #336fbd; }
.stadium-price-legend__ticket--green { background: #46ab59; }
.stadium-price-legend__ticket--purple { background: #6c369b; }
.stadium-price-legend__ticket--yellow { background: #f4c23a; color: #fff; }
.stadium-price-legend__ticket--navy { background: #0f215e; }
.stadium-price-legend__ticket--red { background: #ec3b20; }
.stadium-price-legend__ticket--vip { background: #b90000; }

.stadium-region--current {
  transform: scale(1.06);
  box-shadow: 0 12rpx 26rpx rgba(18, 25, 20, 0.24);
  z-index: 3;
}

.stadium-region--desired {
  transform: scale(1.06);
  box-shadow: 0 12rpx 26rpx rgba(18, 25, 20, 0.24);
  z-index: 3;
}

.stadium-region--filter {
  transform: scale(1.12);
  box-shadow: 0 14rpx 28rpx rgba(18, 25, 20, 0.32);
  z-index: 4;
}

.stadium-region--dimmed {
  opacity: 0.82;
}

.stadium-region--disabled {
  z-index: 3;
}

.stadium-map--review .stadium-region {
  transform: none;
  border-color: transparent;
  background: #d9dee7;
  box-shadow: 0 6rpx 14rpx rgba(98, 107, 122, 0.12);
  color: #6f7683;
}

.stadium-map--review .stadium-region--review-muted {
  opacity: 1;
}

.stadium-map--review .stadium-region--review-current,
.stadium-map--review .stadium-region--review-desired {
  transform: scale(1.08);
  border-color: rgba(23, 25, 31, 0.34);
  color: #17191f;
  opacity: 1;
  box-shadow: 0 14rpx 28rpx rgba(18, 25, 20, 0.28);
}

.stadium-map--review .stadium-region--review-current {
  background: #fff1f0;
  outline: 6rpx solid rgba(226, 59, 46, 0.34);
}

.stadium-map--review .stadium-region--review-desired {
  background: #eef8f2;
  outline: 6rpx solid rgba(29, 138, 85, 0.45);
}
</style>
