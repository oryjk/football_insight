<template>
  <view class="stadium-map" :class="{ 'stadium-map--review': mode === 'review' }">
    <view v-if="hint" class="stadium-map__hint">{{ hint }}</view>
    <view class="stadium-map__field">
      <button
        v-for="region in renderedRegions"
        :key="region.key"
        class="stadium-region"
        :class="region.classes"
        :style="region.style"
        @tap="handleTap(region.key, region.disabled, region.unmapped)"
      >
        <text class="stadium-region__name">{{ region.name }}</text>
        <text v-if="region.badge" class="stadium-region__badge">{{ region.badge }}</text>
      </button>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { TicketWatchRegion } from '../types/ticketWatch'
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

const interactiveModes: Mode[] = ['browse', 'filter', 'select-current', 'select-desired']
const hasStagedDesiredKeys = computed(() => props.stagedDesiredKeys.length > 0)

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

    const dimmed =
      (isPublished && !isCurrent && !isDesired) ||
      (isFilter && !isFilterMatched && !!props.filterKey) ||
      (isSelectCurrent && !isStagedCurrent && !!props.stagedCurrentKey) ||
      (isSelectDesired && !isStagedCurrent && !isStagedDesired && hasStagedDesiredKeys.value)

    const disabled = isSelectDesired && isStagedCurrent

    const badge = props.badges[key] || props.badges[name] || 0
    const unmapped = !position

    return {
      key,
      name,
      style: position
        ? `left:${position.left}%;top:${position.top}%;width:${position.width}%;height:${position.height}%;`
        : '',
      disabled,
      unmapped,
      badge,
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
  if (!interactiveModes.includes(props.mode)) return
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
  color: #8f7c5f;
  font-size: 22rpx;
  font-weight: 400;
}

.stadium-map__field {
  position: relative;
  width: 100%;
  height: 640rpx;
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

.stadium-region__badge {
  position: absolute;
  top: -10rpx;
  right: -10rpx;
  min-width: 28rpx;
  height: 28rpx;
  padding: 0 6rpx;
  border-radius: 999rpx;
  background: linear-gradient(180deg, #20242c, #191d26);
  color: #fff;
  font-size: 18rpx;
  font-weight: 800;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 2rpx solid rgba(255, 251, 242, 0.96);
  box-shadow: 0 4rpx 8rpx rgba(0, 0, 0, 0.18);
  z-index: 4;
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
.stadium-region--muted { background: #d9dee7; }

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
  opacity: 0.28;
  filter: grayscale(1);
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
  background: linear-gradient(180deg, #f7efe1, #f1e3ca);
  outline: 6rpx solid rgba(216, 155, 52, 0.45);
}

.stadium-map--review .stadium-region--review-desired {
  background: linear-gradient(180deg, #eaf8ef, #dff1e6);
  outline: 6rpx solid rgba(29, 138, 85, 0.45);
}
</style>
