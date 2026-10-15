<template>
  <view class="hero-card team-board-hero">
    <view class="team-board-hero__back-row">
      <button class="team-board-hero__back" @click="emit('back')">返回洞察</button>
    </view>

    <view class="hero-card__top">
      <view>
        <text class="eyebrow">Team Board</text>
        <text class="hero-card__title">{{ team.team_name }}战术板</text>
      </view>
      <text class="hero-card__badge">第 {{ team.rank_no }} 名</text>
    </view>

    <text class="hero-card__summary">
      不发普通灌水帖，只围绕当前球队洞察卡发布观点。让讨论先站在数据上，再延伸到判断。
    </text>
  </view>
</template>

<script setup lang="ts">
import type { TeamBoardTeam } from '../../../types/teamBoard'

defineProps<{ team: TeamBoardTeam }>()
const emit = defineEmits<{ (e: 'back'): void }>()
</script>

<style scoped lang="css">
@import '../team-board-shared.css';

.team-board-hero__back-row { margin-bottom: var(--fi-space-16); }
.team-board-hero__back {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  white-space: nowrap;
  line-height: 1;
  padding: var(--fi-space-10) var(--fi-space-18);
  border-radius: 999rpx;
  background: #f6f7fb;
  color: #6d7280;
  font-size: 24rpx;
}
</style>
