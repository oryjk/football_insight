<template>
 <view class="match-list">
  <button v-for="match in matches" :key="match.match_id" class="match" :class="{'match--selected':selected.includes(match.match_id)}" :disabled="disabled" :aria-label="`${selected.includes(match.match_id)?'取消':'选择'} ${match.match_date} ${match.opponent}`" hover-class="match--pressed" @tap="emit('toggle',match.match_id)">
   <text class="match__check">{{selected.includes(match.match_id)?'✓':'＋'}}</text>
   <view class="match__copy"><text class="match__title">{{match.is_home?'主场':'客场'}} · {{match.opponent}}</text><text class="match__date">{{match.match_date}}</text></view>
  </button>
  <text v-if="!matches.length" class="match-list__empty">本赛季暂时没有已结束的蓉城比赛</text>
 </view>
</template>
<script setup lang="ts">
import type { RecapMatch } from '../../../types/seasonRecap'
defineProps<{matches:RecapMatch[];selected:number[];disabled:boolean}>()
const emit=defineEmits<{(event:'toggle',id:number):void}>()
</script>
<style scoped>
.match-list { display:flex; flex-direction:column; gap:var(--fi-space-12); }
.match { display:flex; align-items:center; gap:var(--fi-space-16); width:100%; box-sizing:border-box; text-align:left; margin:0; padding:var(--fi-space-20); background:var(--fi-color-page); border:var(--fi-border-width) solid var(--fi-color-divider); border-radius:var(--fi-radius-sm); line-height:var(--fi-leading-normal); }
.match::after {border:0;}
.match--selected {border-color:var(--fi-color-primary);background:var(--fi-color-surface);}
.match--pressed {opacity:var(--fi-opacity-pressed);}
.match__check {font-size:var(--fi-font-32);font-weight:var(--fi-weight-bold);color:var(--fi-color-primary);}
.match__copy {display:flex;flex-direction:column;gap:var(--fi-space-4);}
.match__title {font-size:var(--fi-font-28);font-weight:var(--fi-weight-bold);color:var(--fi-color-text-primary);}
.match__date,.match-list__empty {font-size:var(--fi-font-24);color:var(--fi-color-text-muted);}
</style>
