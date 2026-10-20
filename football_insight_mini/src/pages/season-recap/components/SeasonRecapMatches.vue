<template>
 <view class="matches">
  <view class="matches__head">
   <text class="matches__title">我去过的比赛</text>
   <text class="matches__count">已选 {{stats.total}} / {{matches.length}}</text>
  </view>
  <view v-if="matches.length" class="matches__toolbar">
   <view class="matches__segmented">
    <FiSegmented compact :items="segmentItems" :model-value="filter" aria-label="按主客场筛选比赛" @update:model-value="changeFilter" />
   </view>
   <button v-if="filtered.length" class="matches__bulk" :class="{'matches__bulk--clear':fullySelected,'matches__bulk--disabled':disabled}" :disabled="disabled" hover-class="matches__bulk--pressed" @tap="bulkToggle">{{bulkLabel}}</button>
  </view>
  <view v-if="filtered.length" class="matches__grid">
   <button v-for="match in filtered" :key="match.match_id" class="match-card" :class="{'match-card--selected':isSelected(match.match_id),'match-card--disabled':disabled}" :disabled="disabled" :aria-label="`${isSelected(match.match_id)?'取消':'选择'} ${match.match_date} ${match.opponent}`" hover-class="match-card--pressed" @tap="emit('toggle',match.match_id)">
    <view class="match-card__row">
     <text class="match-card__tag" :class="match.is_home?'match-card__tag--home':'match-card__tag--away'">{{match.is_home?'主':'客'}}</text>
     <text class="match-card__date">{{formatRecapDate(match.match_date)}}</text>
    </view>
    <text class="match-card__opponent">{{match.opponent}}</text>
    <text v-if="isSelected(match.match_id)" class="match-card__check">✓</text>
   </button>
  </view>
  <text v-if="!matches.length" class="matches__empty">本赛季暂时没有已结束的蓉城比赛</text>
 </view>
</template>
<script setup lang="ts">
import { computed, ref } from 'vue'
import FiSegmented from '../../../components/FiSegmented.vue'
import type { RecapMatch } from '../../../types/seasonRecap'
import { filterRecapMatches, formatRecapDate, isFilterFullySelected, recapFilterCounts, recapSelectionStats, toggleFilterSelection, type RecapFilter } from '../helpers'
const props = defineProps<{matches:RecapMatch[];selected:number[];disabled:boolean}>()
const emit = defineEmits<{(event:'toggle',id:number):void;(event:'select',ids:number[]):void}>()
const filter = ref<RecapFilter>('all')
const counts = computed(() => recapFilterCounts(props.matches))
const segmentItems = computed(() => [
 {key:'all',label:`全部 ${counts.value.all}`},
 {key:'home',label:`主场 ${counts.value.home}`},
 {key:'away',label:`客场 ${counts.value.away}`},
])
const filtered = computed(() => filterRecapMatches(props.matches, filter.value))
const stats = computed(() => recapSelectionStats(props.matches, props.selected))
const fullySelected = computed(() => isFilterFullySelected(props.matches, props.selected, filter.value))
const bulkLabel = computed(() => fullySelected.value ? '清空' : filter.value === 'home' ? '全选主场' : filter.value === 'away' ? '全选客场' : '全选')
function isSelected(id: number) { return props.selected.includes(id) }
function changeFilter(key: string) { if (!props.disabled) filter.value = key as RecapFilter }
function bulkToggle() { if (!props.disabled) emit('select', toggleFilterSelection(props.matches, props.selected, filter.value)) }
</script>
<style scoped>
.matches__head {display:flex;align-items:center;justify-content:space-between;gap:var(--fi-space-12);}
.matches__title {font-size:var(--fi-font-28);font-weight:var(--fi-weight-bold);color:var(--fi-color-text-primary);}
.matches__count {flex-shrink:0;font-size:var(--fi-font-24);color:var(--fi-color-text-muted);}
.matches__toolbar {display:flex;align-items:center;gap:var(--fi-space-12);margin-top:var(--fi-space-16);}
.matches__segmented {flex:1;min-width:0;}
.matches__bulk {flex-shrink:0;margin:0;padding:var(--fi-space-8) var(--fi-space-12);background:transparent;border:0;border-radius:var(--fi-radius-sm);font-size:var(--fi-font-26);font-weight:var(--fi-weight-bold);line-height:var(--fi-leading-normal);color:var(--fi-color-primary);}
.matches__bulk::after {border:0;}
.matches__bulk--clear {color:var(--fi-color-text-secondary);}
.matches__bulk--pressed {opacity:var(--fi-opacity-pressed);}
.matches__bulk--disabled {color:var(--fi-color-text-muted);}
.matches__grid {display:grid;grid-template-columns:repeat(3, minmax(0, 1fr));gap:var(--fi-space-12);margin-top:var(--fi-space-16);}
.match-card {position:relative;display:flex;flex-direction:column;gap:var(--fi-space-8);box-sizing:border-box;width:100%;margin:0;padding:var(--fi-space-12);text-align:left;background:var(--fi-color-page);border:var(--fi-border-width) solid var(--fi-color-divider);border-radius:var(--fi-radius-sm);line-height:var(--fi-leading-normal);}
.match-card::after {border:0;}
.match-card--selected {border-color:var(--fi-color-primary);background:var(--fi-color-primary-soft);}
.match-card--pressed {opacity:var(--fi-opacity-pressed);}
.match-card--disabled {opacity:var(--fi-opacity-pressed);}
.match-card__row {display:flex;align-items:center;gap:var(--fi-space-8);}
.match-card__tag {flex-shrink:0;padding:var(--fi-space-4) var(--fi-space-8);border-radius:var(--fi-radius-xs);font-size:var(--fi-font-20);font-weight:var(--fi-weight-bold);line-height:var(--fi-leading-normal);}
.match-card__tag--home {background:var(--fi-color-primary-soft);color:var(--fi-color-primary);}
.match-card__tag--away {background:var(--fi-color-divider-soft);color:var(--fi-color-text-secondary);}
.match-card__date {font-size:var(--fi-font-22);color:var(--fi-color-text-muted);}
.match-card__opponent {font-size:var(--fi-font-26);font-weight:var(--fi-weight-bold);color:var(--fi-color-text-primary);overflow:hidden;text-overflow:ellipsis;white-space:nowrap;}
.match-card__check {position:absolute;top:var(--fi-space-8);right:var(--fi-space-8);display:flex;align-items:center;justify-content:center;width:var(--fi-space-28);height:var(--fi-space-28);border-radius:var(--fi-radius-round);background:var(--fi-color-primary);color:var(--fi-color-surface);font-size:var(--fi-font-18);line-height:var(--fi-leading-none);}
.matches__empty {display:block;margin-top:var(--fi-space-16);font-size:var(--fi-font-24);color:var(--fi-color-text-muted);text-align:center;}
</style>
