<template>
  <div class="region-chips">
    <p v-if="hint" class="region-chips__hint">{{ hint }}</p>
    <div v-for="group in renderedGroups" :key="group.key" class="region-chips__group">
      <span class="region-chips__group-label">{{ group.label }}</span>
      <div class="region-chips__row">
        <button
          v-for="item in group.items"
          :key="item.key"
          type="button"
          class="region-chip"
          :class="item.classes"
          :disabled="item.disabled"
          @click="handleTap(item.key, item.disabled)"
        >
          <span class="region-chip__name">{{ item.name }}</span>
          <span v-if="badges[item.key]" class="region-chip__badge">{{ badges[item.key] }}</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { TicketWatchRegion } from '../types/ticketWatch'
import {
  canInteractSeatSwapMap,
  groupSeatSwapRegionsForChips,
  resolveSeatSwapRegionVisualState,
  type SeatSwapRegionMode,
} from '../utils/seatSwap'
import { resolveSeatSwapRegionColorGroup } from '../utils/stadiumRegions'

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

const emit = defineEmits<{
  (e: 'region-tap', key: string): void
}>()

function regionKey(region: TicketWatchRegion): string {
  return region.block_key || region.block_name
}

const renderedGroups = computed(() =>
  groupSeatSwapRegionsForChips(props.regions).map((group) => ({
    key: group.key,
    label: group.label,
    items: group.items.map((region) => {
      const key = regionKey(region)
      const colorGroup = resolveSeatSwapRegionColorGroup(region.block_name)
      const state = resolveSeatSwapRegionVisualState({
        mode: props.mode,
        key,
        filterKey: props.filterKey,
        currentKey: props.currentKey,
        desiredKeys: props.desiredKeys,
        stagedCurrentKey: props.stagedCurrentKey,
        stagedDesiredKeys: props.stagedDesiredKeys,
      })
      const isReview = props.mode === 'review'

      return {
        key,
        name: region.block_name,
        disabled: state.disabled,
        classes: [
          !isReview && `region-chip--${colorGroup}`,
          {
            'region-chip--current': state.highlightCurrent,
            'region-chip--desired': state.highlightDesired,
            'region-chip--filter': state.highlightFilter,
            'region-chip--review-muted': state.reviewMuted,
            'region-chip--review-current': state.reviewCurrent,
            'region-chip--review-desired': state.reviewDesired,
            'region-chip--dimmed': state.dimmed,
          },
        ],
      }
    }),
  })),
)

function handleTap(key: string, disabled: boolean) {
  if (disabled) return
  if (!canInteractSeatSwapMap(props.mode)) return
  emit('region-tap', key)
}
</script>

<style scoped>
.region-chips {
  display: grid;
  gap: 10px;
}

.region-chips__hint {
  margin: 0;
  color: #988f84;
  font-size: 11px;
  text-align: center;
}

.region-chips__group {
  display: grid;
  gap: 6px;
}

.region-chips__group-label {
  color: #988f84;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.5px;
}

.region-chips__row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.region-chip {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 56px;
  height: 38px;
  padding: 0 12px;
  box-sizing: border-box;
  border: 0;
  border-radius: 8px;
  background: #e8ebf1;
  color: #17191f;
  font-size: 13px;
  font-weight: 800;
  line-height: 1;
  box-shadow: 0 2px 5px rgba(18, 25, 20, 0.12);
  transition: transform 0.16s ease, box-shadow 0.16s ease, opacity 0.16s ease;
  cursor: pointer;
}

.region-chip:active {
  transform: scale(0.96);
}

.region-chip__name {
  display: block;
  white-space: nowrap;
}

.region-chip__badge {
  position: absolute;
  top: -6px;
  right: -4px;
  min-width: 17px;
  height: 17px;
  padding: 0 4px;
  box-sizing: border-box;
  background: linear-gradient(180deg, #20242c, #191d26);
  color: #fff;
  font-size: 10px;
  font-weight: 700;
  border-radius: 999px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1.5px solid #fffaef;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.22);
  z-index: 2;
}

/* 票价分区配色，与球场示意图一致 */
.region-chip--blue { background: #336fbd; color: #fff; }
.region-chip--green { background: #46ab59; color: #fff; }
.region-chip--purple { background: #6c369b; color: #fff; }
.region-chip--yellow { background: #f4c23a; }
.region-chip--navy { background: #0f215e; color: #fff; }
.region-chip--red { background: #ec3b20; color: #fff; }
.region-chip--vip { background: #b90000; color: #fff; }
.region-chip--muted { background: #d9dee7; }

.region-chip--current,
.region-chip--desired {
  transform: scale(1.05);
  box-shadow: 0 4px 9px rgba(18, 25, 20, 0.2);
  z-index: 1;
}

.region-chip--current {
  outline: 2.5px solid rgba(216, 155, 52, 0.9);
  outline-offset: 1px;
}

.region-chip--desired {
  outline: 2.5px solid rgba(29, 138, 85, 0.9);
  outline-offset: 1px;
}

.region-chip--filter {
  transform: scale(1.05);
  outline: 2.5px solid #15161b;
  outline-offset: 1px;
  box-shadow: 0 5px 10px rgba(18, 25, 20, 0.24);
  z-index: 1;
}

.region-chip--review-muted {
  background: #eef0f4;
  color: #a4a8b2;
  box-shadow: none;
}

.region-chip--review-current {
  background: rgba(216, 155, 52, 0.16);
  color: #8a6420;
  outline: 2px solid rgba(216, 155, 52, 0.9);
  outline-offset: 1px;
}

.region-chip--review-desired {
  background: rgba(29, 138, 85, 0.14);
  color: #1d8a55;
  outline: 2px solid rgba(29, 138, 85, 0.9);
  outline-offset: 1px;
}

.region-chip--dimmed {
  opacity: 0.45;
}

.region-chip:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}
</style>
