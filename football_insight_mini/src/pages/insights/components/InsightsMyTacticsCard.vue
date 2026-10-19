<template>
  <view class="my-tactics">
    <view class="my-tactics__heading"><text>我的战术</text><text class="my-tactics__status">{{ authenticated ? (loading ? '同步中…' : summary.status) : '登录后同步草稿' }}</text></view>
    <view v-if="loading" class="my-tactics__skeleton fi-skeleton-block" />
    <view v-else-if="error" class="my-tactics__body">
      <text class="my-tactics__hint">{{ error }}</text><button class="my-tactics__action" @tap="emit('retry')">重试</button>
    </view>
    <view v-else class="my-tactics__body">
      <view class="mini-pitch" aria-hidden="true">
        <view class="mini-pitch__half" /><view class="mini-pitch__circle" />
        <view v-for="slot in summary.snapshot?.slots ?? []" :key="slot.slot" class="mini-pitch__player"
          :class="{ 'mini-pitch__player--empty': slot.player_id === null }" :style="miniPitchPosition(slot.x, slot.y)" />
        <text v-if="!summary.snapshot" class="mini-pitch__empty">＋</text>
      </view>
      <view class="my-tactics__copy">
        <text class="my-tactics__formation">{{ summary.snapshot?.formation ?? '排出你的首发' }}</text>
        <text class="my-tactics__hint">{{ summary.snapshot ? `${summary.filled} / 11 人 · 发布后参与球队排名` : '选择球员，自由调整站位' }}</text>
      </view>
      <button class="my-tactics__action" hover-class="my-tactics__pressed" @tap="emit('edit')">{{ summary.action }}</button>
    </view>
  </view>
</template>
<script setup lang="ts">
import { computed } from 'vue'
import type { SavedLineup, SavedLineupDraft } from '../../../types/lineup'
import { miniPitchPosition, myTacticsSummary } from '../tacticsPresentation'
const props = defineProps<{ authenticated: boolean; loading: boolean; error: string; draft: SavedLineupDraft | null; published: SavedLineup | null }>()
const emit = defineEmits<{ (e: 'edit'): void; (e: 'retry'): void }>()
const summary = computed(() => myTacticsSummary(props.draft, props.published))
</script>
<style scoped>
@import '../../../styles/fi-skeleton.css';
.my-tactics { padding:var(--fi-space-20); border-radius:var(--fi-radius-md); background:var(--fi-color-page-soft); }
.my-tactics__heading { display:flex; justify-content:space-between; align-items:center; gap:var(--fi-space-12); font-size:var(--fi-font-28); font-weight:var(--fi-weight-extrabold); color:var(--fi-color-text-primary); }
.my-tactics__status { font-size:var(--fi-font-22); color:var(--fi-color-text-caption); font-weight:normal; }
.my-tactics__body { margin-top:var(--fi-space-20); display:flex; align-items:center; gap:var(--fi-space-16); }
.my-tactics__copy { display:flex; flex-direction:column; gap:var(--fi-space-8); flex:1; min-width:0; }
.my-tactics__formation { font-size:var(--fi-font-28); color:var(--fi-color-text-primary); font-weight:var(--fi-weight-bold); }
.my-tactics__hint { font-size:var(--fi-font-22); color:var(--fi-color-text-caption); line-height:var(--fi-leading-normal); }
.my-tactics__action { flex-shrink:0; margin:0; padding:var(--fi-space-16); background:var(--fi-color-primary); color:var(--fi-color-surface); font-size:var(--fi-font-24); line-height:var(--fi-leading-snug); border-radius:var(--fi-radius-round); font-weight:var(--fi-weight-bold); }
.my-tactics__action::after { border:0; }
.my-tactics__pressed { opacity:var(--fi-opacity-pressed); }
.my-tactics__skeleton { margin-top:var(--fi-space-20); height:var(--fi-space-64); }
.mini-pitch { position:relative; width:calc(var(--fi-space-64) * 1.5); height:calc(var(--fi-space-64) * 1.8); flex-shrink:0; overflow:hidden; border-radius:var(--fi-radius-xs); border:var(--fi-border-width) solid var(--fi-color-lineup-markings); background:var(--fi-color-lineup-grass); }
.mini-pitch__half { position:absolute; top:50%; width:100%; border-top:var(--fi-border-width) solid var(--fi-color-lineup-markings); }
.mini-pitch__circle { position:absolute; top:50%; left:50%; width:var(--fi-space-24); height:var(--fi-space-24); transform:translate(-50%,-50%); border:var(--fi-border-width) solid var(--fi-color-lineup-markings); border-radius:50%; }
.mini-pitch__player { position:absolute; width:var(--fi-space-10); height:var(--fi-space-10); transform:translate(-50%,-50%); background:var(--fi-color-surface); border-radius:50%; }
.mini-pitch__player--empty { opacity:var(--fi-opacity-watermark); }
.mini-pitch__empty { position:absolute; top:50%; left:50%; transform:translate(-50%,-50%); color:var(--fi-color-surface); font-size:var(--fi-font-36); }
</style>
