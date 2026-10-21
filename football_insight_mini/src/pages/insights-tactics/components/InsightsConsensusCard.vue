<template>
  <!-- 大家心中的最佳 11 人：未解锁时是样本进度，并明确说明不影响查看别人的阵容；
       解锁后在小球场展示共识，我也选了的球员红色描边、没选的变淡，点球员看被选比例。 -->
  <view class="consensus">
    <view class="consensus__head">
      <text class="consensus__title">{{ consensus.unlocked ? '大家心中的最佳 11 人' : '最佳 11 人 · 样本积累中' }}</text>
      <text v-if="consensus.unlocked" class="consensus__meta">基于 {{ consensus.total }} 套阵容 · {{ consensus.formation }}（{{ consensus.formation_share }}% 的人用）</text>
    </view>
    <view v-if="!consensus.unlocked" class="consensus__locked">
      <text class="consensus__progress-text">{{ consensusProgressText(consensus.total, consensus.threshold) }}</text>
      <text class="consensus__progress-text">不影响查看：别人排的阵容在下方可以直接看，不用先排。</text>
      <view class="consensus__bar"><view class="consensus__bar-fill" :style="{ width: `${progress}%` }" /></view>
      <button class="consensus__action" hover-class="consensus__action--pressed" @tap="emit('edit')">我也排一套</button>
    </view>
    <view v-else class="consensus__pitch">
      <view class="consensus__half" />
      <view class="consensus__circle" />
      <view
        v-for="entry in placed"
        :key="entry.slot"
        class="consensus__player"
        :class="{ 'consensus__player--mine': mine.has(entry.player.player_id), 'consensus__player--other': hasMine && !mine.has(entry.player.player_id) }"
        :style="{ left: `${entry.x * 100}%`, top: `${entry.y * 100}%` }"
        hover-class="consensus__player--pressed"
        @tap="explain(entry.player)"
      >
        <view class="consensus__avatar"><FiAvatar :src="entry.player.avatar_url" :size="96" :name="entry.player.name" /></view>
        <text class="consensus__name">{{ entry.player.name }}</text>
        <text class="consensus__rate">{{ entry.player.pick_rate }}%</text>
      </view>
    </view>
    <text v-if="consensus.unlocked && hasMine" class="consensus__legend">红框是你也选了的球员</text>
  </view>
</template>
<script setup lang="ts">
import { computed } from 'vue'
import FiAvatar from '../../../components/FiAvatar.vue'
import type { LineupConsensus, LineupConsensusPlayer } from '../../../types/lineup'
import { consensusPlayerHint, consensusProgressText, placeConsensus } from '../../../utils/lineupConsensus'
const props = defineProps<{ consensus: LineupConsensus; myPlayerIds: number[] }>()
const emit = defineEmits<{ (e: 'edit'): void; (e: 'explain', message: string): void }>()
const placed = computed(() => placeConsensus(props.consensus))
const mine = computed(() => new Set(props.myPlayerIds))
const hasMine = computed(() => props.myPlayerIds.length > 0)
const progress = computed(() => props.consensus.threshold > 0 ? Math.min(100, Math.round(props.consensus.total * 100 / props.consensus.threshold)) : 0)
// 组件只发事件，由页面决定怎么提示（toast），保持组件不直接调用 uni。
function explain(player: LineupConsensusPlayer): void { emit('explain', consensusPlayerHint(player, mine.value.has(player.player_id))) }
</script>
<style scoped>
.consensus { display: flex; flex-direction: column; gap: var(--fi-space-12); padding: var(--fi-space-20); border-radius: var(--fi-radius-md); background: var(--fi-color-page-soft); }
.consensus__head { display: flex; flex-direction: column; gap: var(--fi-space-4); }
.consensus__title { color: var(--fi-color-text-strong); font-size: var(--fi-font-28); font-weight: var(--fi-weight-extrabold); }
.consensus__meta, .consensus__progress-text, .consensus__legend { color: var(--fi-color-text-secondary); font-size: var(--fi-font-22); line-height: var(--fi-leading-normal); }
.consensus__locked { display: flex; flex-direction: column; gap: var(--fi-space-12); }
.consensus__bar { height: var(--fi-space-10); border-radius: var(--fi-radius-round); background: var(--fi-color-surface); overflow: hidden; }
.consensus__bar-fill { height: 100%; border-radius: var(--fi-radius-round); background: var(--fi-color-primary); transition: width var(--fi-motion-duration-base, 240ms) var(--fi-motion-ease-out); }
.consensus__action { align-self: flex-start; margin: 0; padding: var(--fi-space-14) var(--fi-space-32); border-radius: var(--fi-radius-round); background: var(--fi-color-primary); color: var(--fi-color-surface); font-size: var(--fi-font-24); font-weight: var(--fi-weight-bold); line-height: var(--fi-leading-normal); }
.consensus__action::after { border: 0; }
.consensus__action--pressed { opacity: var(--fi-opacity-pressed); }
.consensus__pitch { position: relative; width: 100%; height: 0; padding-top: var(--fi-component-consensus-pitch-ratio); border-radius: var(--fi-radius-md); background: var(--fi-color-lineup-grass); overflow: hidden; }
.consensus__half { position: absolute; top: 50%; left: 3%; right: 3%; border-top: var(--fi-border-width) solid var(--fi-color-lineup-markings); }
.consensus__circle { position: absolute; top: 50%; left: 50%; width: 22%; height: 16%; border: var(--fi-border-width) solid var(--fi-color-lineup-markings); border-radius: var(--fi-radius-round); transform: translate(-50%, -50%); }
.consensus__player { position: absolute; display: flex; flex-direction: column; align-items: center; gap: var(--fi-space-2); width: var(--fi-space-96); transform: translate(-50%, -50%); transition: opacity var(--fi-motion-duration-fast) var(--fi-motion-ease-out); }
.consensus__player--other { opacity: var(--fi-opacity-pressed); }
.consensus__player--pressed { transform: translate(-50%, -50%) scale(0.94); }
.consensus__avatar { overflow: hidden; width: var(--fi-space-48); height: var(--fi-space-48); border: var(--fi-border-width) solid var(--fi-color-surface); border-radius: var(--fi-radius-round); background: var(--fi-color-surface); font-size: var(--fi-font-22); }
.consensus__player--mine .consensus__avatar { border-color: var(--fi-color-primary); box-shadow: 0 0 0 var(--fi-border-width) var(--fi-color-primary); }
.consensus__name { max-width: 100%; overflow: hidden; color: var(--fi-color-surface); font-size: var(--fi-font-20); font-weight: var(--fi-weight-bold); text-overflow: ellipsis; white-space: nowrap; }
.consensus__rate { color: var(--fi-color-surface); font-size: var(--fi-font-18); }
@media (prefers-reduced-motion: reduce) {
  .consensus__bar-fill, .consensus__player { transition: none; }
}
</style>
