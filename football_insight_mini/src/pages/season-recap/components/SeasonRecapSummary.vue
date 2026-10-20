<template>
 <view class="summary">
  <view class="summary__head">
   <text class="summary__title">{{season}} 赛季里程</text>
   <text v-if="dirty" class="summary__dirty">未保存</text>
  </view>
   <view class="summary__total">
    <text :key="pulseTick" class="summary__number">{{totalKm}}</text>
    <text class="summary__unit">公里</text>
   </view>
   <text v-if="!summary.matches" class="summary__empty">还没有记录，勾选比赛后生成海报</text>
   <view v-else class="summary__stats">
    <view class="summary__stat">
     <text class="summary__stat-value">{{awayKm}} 公里</text>
     <text class="summary__stat-label">客场</text>
    </view>
    <view class="summary__stat">
     <text class="summary__stat-value">{{homeValue}}</text>
     <text class="summary__stat-label">主场</text>
    </view>
    <view class="summary__stat">
     <text class="summary__stat-value">{{matchCount}} 场 · {{cityCount}} 城</text>
     <text class="summary__stat-label">足迹</text>
    </view>
   </view>
 </view>
</template>
<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useAnimatedInteger } from '../../../composables/useAnimatedInteger'
import type { RecapSummary } from '../../../types/seasonRecap'
const props = defineProps<{summary:RecapSummary;season:number;dirty:boolean}>()
// 首次加载从 0 滚动一次；后续勾选/改距离从当前显示值继续过渡。
const totalSource = computed(() => props.summary.matches ? props.summary.total_km : 0)
const totalKm = useAnimatedInteger(totalSource, { initialFromZero: true })
const awayKm = useAnimatedInteger(() => props.summary.away_km, { initialFromZero: true })
const homeKm = useAnimatedInteger(() => props.summary.home_km ?? 0, { initialFromZero: true })
const matchCount = useAnimatedInteger(() => props.summary.matches, { initialFromZero: true })
const cityCount = useAnimatedInteger(() => props.summary.cities, { initialFromZero: true })
// 有已选主场但距离未填/非法时提示「待填写」；没有主场时主场格不适用。非数字状态不滚动，直接切换文案。
const homeValue = computed(() => props.summary.home_km !== null ? `${homeKm.value} 公里` : props.summary.home_matches > 0 ? '待填写' : '—')
// 总里程每次变化时通过 remount 重放一次轻微 scale 强调动画（小程序端无法用 reflow 重启 animation）。
const pulseTick = ref(0)
watch(totalSource, () => { pulseTick.value += 1 })
</script>
<style scoped>
.summary__head {display:flex;align-items:center;justify-content:space-between;gap:var(--fi-space-12);}
.summary__title {font-size:var(--fi-font-24);font-weight:var(--fi-weight-bold);color:var(--fi-color-text-muted);letter-spacing:var(--fi-component-kicker-tracking);}
.summary__dirty {flex-shrink:0;padding:var(--fi-space-4) var(--fi-space-12);border-radius:var(--fi-radius-round);background:var(--fi-color-primary-soft);color:var(--fi-color-primary);font-size:var(--fi-font-20);font-weight:var(--fi-weight-bold);line-height:var(--fi-leading-normal);}
.summary__total {display:flex;align-items:baseline;gap:var(--fi-space-8);margin-top:var(--fi-space-12);}
.summary__number {display:inline-block;font-size:var(--fi-font-56);font-weight:var(--fi-weight-black);line-height:var(--fi-leading-heading);color:var(--fi-color-primary);animation:summary-number-pulse var(--fi-motion-duration-slow) var(--fi-motion-ease-out);}
@keyframes summary-number-pulse {0% {transform:scale(1);} 50% {transform:scale(1.06);} 100% {transform:scale(1);}}
.summary__unit {font-size:var(--fi-font-24);color:var(--fi-color-text-secondary);}
.summary__empty {display:block;margin-top:var(--fi-space-12);font-size:var(--fi-font-24);line-height:var(--fi-leading-normal);color:var(--fi-color-text-secondary);}
.summary__stats {display:flex;gap:var(--fi-space-12);margin-top:var(--fi-space-16);}
.summary__stat {flex:1;min-width:0;display:flex;flex-direction:column;gap:var(--fi-space-4);padding:var(--fi-space-12);border-radius:var(--fi-radius-sm);background:var(--fi-color-page);}
.summary__stat-value {font-size:var(--fi-font-26);font-weight:var(--fi-weight-bold);color:var(--fi-color-text-primary);line-height:var(--fi-leading-snug);}
.summary__stat-label {font-size:var(--fi-font-20);color:var(--fi-color-text-muted);line-height:var(--fi-leading-normal);}
</style>
