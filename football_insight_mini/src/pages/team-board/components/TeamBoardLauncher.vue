<template>
  <view class="panel team-board-launcher">
    <view class="section-heading section-heading--compact">
      <view>
        <text class="section-kicker">发布观点</text>
        <text class="section-title">先选一张洞察卡</text>
      </view>
      <text class="meta-pill">{{ postCount }} 篇观点</text>
    </view>

    <view class="team-board-launcher__grid">
      <button
        v-for="preset in presets"
        :key="preset.insight_kind"
        class="team-board-launcher__card"
        @click="emit('open-composer', preset.insight_kind)"
      >
        <view class="team-board-launcher__meta">
          <text>{{ preset.label }}</text>
          <text class="team-board-launcher__value">{{ preset.snapshot.summary_value }}</text>
        </view>
        <text class="team-board-launcher__summary">{{ preset.snapshot.summary_label }}</text>
        <text class="team-board-launcher__note">基于当前洞察卡发布观点</text>
      </button>
    </view>
  </view>
</template>

<script setup lang="ts">
import type { TeamBoardComposerPreset, TeamBoardInsightKind } from '../../../types/teamBoard'

defineProps<{ presets: TeamBoardComposerPreset[]; postCount: number }>()
const emit = defineEmits<{ (e: 'open-composer', kind: TeamBoardInsightKind): void }>()
</script>

<style scoped lang="css">
@import '../team-board-shared.css';

.team-board-launcher__grid {
  margin-top: var(--fi-space-18);
  border-top: 2rpx solid rgba(235, 236, 241, 0.86);
}
.team-board-launcher__card {
  display: grid;
  gap: var(--fi-space-10);
  padding: var(--fi-space-22) 0;
  border-radius: 0;
  border: 0;
  border-bottom: 2rpx solid rgba(235, 236, 241, 0.86);
  background: transparent;
  text-align: left;
}
.team-board-launcher__meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-12);
}
.team-board-launcher__meta text { font-size: 24rpx; color: #121212; font-weight: 700; }
.team-board-launcher__value { font-size: 34rpx !important; }
.team-board-launcher__note { color: #8f9198; font-size: 22rpx; }
</style>
