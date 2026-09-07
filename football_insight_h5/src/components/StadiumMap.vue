<template>
  <div class="stadium-map">
    <p v-if="hint" class="stadium-map__hint">{{ hint }}</p>
    <div class="stadium-map__field">
      <div class="stadium-price-legend" aria-hidden="true">
        <div
          v-for="item in priceLegendItems"
          :key="item.grade"
          class="stadium-price-legend__ticket"
          :class="`stadium-price-legend__ticket--${item.colorGroup}`"
        >
          <span class="stadium-price-legend__grade">{{ item.grade }}</span>
          <span class="stadium-price-legend__price">{{ item.price }}</span>
        </div>
      </div>
      <div
        v-for="region in renderedRegions"
        :key="region.key"
        class="stadium-region"
        :class="region.classes"
        :style="region.style"
      >
        <span class="stadium-region__name">{{ region.name }}</span>
        <span v-if="badges[region.key]" class="stadium-region__badge">{{ badges[region.key] }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { TicketWatchRegion } from '../types/ticketWatch'
import {
  resolveSeatSwapRegionVisualState,
  type SeatSwapRegionMode,
} from '../utils/seatSwap'
import {
  resolveSeatSwapRegionColorGroup,
  resolveSeatSwapRegionLayout,
} from '../utils/stadiumRegions'

interface Props {
  mode: SeatSwapRegionMode
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
    const state = resolveSeatSwapRegionVisualState({
      mode: props.mode,
      key,
      filterKey: props.filterKey,
      currentKey: props.currentKey,
      desiredKeys: props.desiredKeys,
      stagedCurrentKey: props.stagedCurrentKey,
      stagedDesiredKeys: props.stagedDesiredKeys,
    })

    const unmapped = !position
    const isReview = props.mode === 'review'

    return {
      key,
      name,
      style: position
        ? `left:${position.left}%;top:${position.top}%;width:${position.width}%;height:${position.height}%;`
        : '',
      unmapped,
      classes: [
        !isReview && `stadium-region--${colorGroup}`,
        {
          'stadium-region--unmapped': unmapped,
          'stadium-region--current': state.highlightCurrent,
          'stadium-region--desired': state.highlightDesired,
          'stadium-region--filter': state.highlightFilter,
          'stadium-region--review-muted': state.reviewMuted,
          'stadium-region--review-current': state.reviewCurrent,
          'stadium-region--review-desired': state.reviewDesired,
          'stadium-region--dimmed': state.dimmed,
        },
      ],
    }
  }),
)
</script>

<style scoped>
.stadium-map__hint {
  display: block;
  margin: 0 0 5px;
  text-align: center;
  color: #8f9198;
  font-size: 12px;
}

.stadium-map__field {
  position: relative;
  width: 100%;
  height: 340px;
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
  min-height: 22px;
  border: 0;
  border-radius: 6px;
  box-sizing: border-box;
  margin: 0;
  padding: 1px 2px;
  background-image: none;
  color: #17191f;
  font: inherit;
  line-height: 1.1;
  box-shadow: 0 4px 9px rgba(18, 25, 20, 0.18);
  transition: transform 0.16s ease, box-shadow 0.16s ease, opacity 0.16s ease, filter 0.16s ease;
}

.stadium-region__name {
  position: relative;
  z-index: 1;
  display: block;
  max-width: 100%;
  overflow: hidden;
  font-size: 10px;
  font-weight: 900;
  text-align: center;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.stadium-region__badge {
  position: absolute;
  top: -5px;
  right: -5px;
  min-width: 15px;
  height: 15px;
  padding: 0 3px;
  box-sizing: border-box;
  background: linear-gradient(180deg, #20242c, #191d26);
  color: #fff;
  font-size: 8px;
  font-weight: 700;
  border-radius: 999px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1.5px solid #fffaef;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.22);
  z-index: 2;
}

.stadium-region--unmapped {
  display: none;
}

/* decorative: 票价分区配色 */
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
  gap: 4px;
  pointer-events: none;
}

.stadium-price-legend__ticket {
  position: relative;
  display: flex;
  align-items: center;
  height: 17px;
  min-width: 53px;
  overflow: hidden;
  border-radius: 4px;
  color: #fff;
  box-shadow: 0 3px 6px rgba(18, 25, 20, 0.12);
}

.stadium-price-legend__ticket::before,
.stadium-price-legend__ticket::after {
  position: absolute;
  left: 36%;
  width: 5px;
  height: 5px;
  border-radius: 999px;
  background: #f3f2ef;
  content: '';
  transform: translateX(-50%);
}

.stadium-price-legend__ticket::before { top: -3px; }
.stadium-price-legend__ticket::after { bottom: -3px; }

.stadium-price-legend__grade {
  display: flex;
  width: 21px;
  height: 100%;
  align-items: center;
  justify-content: center;
  border-right: 1px solid rgba(255, 255, 255, 0.38);
  font-size: 9px;
  font-weight: 900;
}

.stadium-price-legend__price {
  flex: 1;
  padding: 0 6px;
  font-size: 9px;
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

.stadium-region--current,
.stadium-region--desired {
  transform: scale(1.08);
  box-shadow: 0 6px 13px rgba(18, 25, 20, 0.24);
  z-index: 3;
}

.stadium-region--current {
  outline: 2.5px solid rgba(216, 155, 52, 0.9);
  outline-offset: 1px;
}

.stadium-region--desired {
  outline: 2.5px solid rgba(29, 138, 85, 0.9);
  outline-offset: 1px;
}

.stadium-region--filter {
  transform: scale(1.14);
  outline: 2.5px solid #15161b;
  outline-offset: 1px;
  box-shadow: 0 7px 14px rgba(18, 25, 20, 0.32);
  z-index: 4;
}

.stadium-region--dimmed {
  opacity: 0.82;
}
</style>
