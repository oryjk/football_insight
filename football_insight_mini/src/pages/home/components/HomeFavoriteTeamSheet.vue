<template>
  <HomeSheetShell kicker="选择主队" title="以后首页默认先给它站队" :closable="false" @close="emit('close')">
    <template #meta>
      <text class="meta-note">单主队 MVP</text>
    </template>

    <scroll-view scroll-y class="favorite-team-sheet__list">
      <view
        v-for="team in teams"
        :key="team.team_id"
        class="favorite-team-sheet__row"
        :class="{ 'favorite-team-sheet__row--active': selectedTeamId === team.team_id }"
        @click="emit('change', team.team_id)"
      >
        <image :src="team.avatar_storage_url || ''" mode="aspectFit" class="favorite-team-sheet__avatar" />
        <view class="favorite-team-sheet__body">
          <text class="favorite-team-sheet__name">{{ team.team_name }}</text>
          <text class="favorite-team-sheet__note">{{ team.rank_no ? `当前积分榜第 ${team.rank_no}` : '等待排名同步' }}</text>
        </view>
      </view>
    </scroll-view>

    <view class="sheet-actions">
      <button class="primary-action primary-action--ghost" @click="emit('close')">取消</button>
      <button class="primary-action" @click="emit('confirm')">确认主队</button>
    </view>
  </HomeSheetShell>
</template>

<script setup lang="ts">
import HomeSheetShell from './HomeSheetShell.vue'
import type { SupportTeam } from '../../../types/support'

defineProps<{
  teams: SupportTeam[]
  selectedTeamId: number | null
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'change', teamId: number): void
  (e: 'confirm'): void
}>()
</script>

<style scoped lang="css">
.meta-note {
  display: inline-flex;
  align-items: center;
  justify-content: flex-end;
  flex: 0 0 auto;
  flex-shrink: 0;
  max-width: 100%;
  white-space: nowrap;
  line-height: 1;
  color: var(--fi-color-text-muted);
  font-size: 24rpx;
  font-weight: 700;
  letter-spacing: 1rpx;
  padding: 8rpx 0 0;
}

.favorite-team-sheet__list {
  margin-top: 20rpx;
  max-height: 52vh;
}
.favorite-team-sheet__row {
  padding: 18rpx 0;
  display: flex;
  align-items: center;
  gap: 16rpx;
  border-bottom: 2rpx solid #f0f1f5;
}
.favorite-team-sheet__row--active {
  background: rgba(var(--fi-primitive-red-rgb), 0.08);
}
.favorite-team-sheet__avatar {
  width: 72rpx;
  height: 72rpx;
}
.favorite-team-sheet__body {
  flex: 1;
  min-width: 0;
}
.favorite-team-sheet__name {
  color: var(--fi-color-text-strong);
  display: block;
  font-size: 36rpx;
  line-height: 1;
  font-weight: 800;
}
.favorite-team-sheet__note {
  display: block;
  margin-top: 6rpx;
  color: var(--fi-color-text-muted);
  font-size: 22rpx;
}
.sheet-actions {
  margin-top: 24rpx;
  display: flex;
  gap: 16rpx;
}
.primary-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: 1;
  padding: 20rpx 30rpx;
  border-radius: 999rpx;
  background: var(--fi-primitive-ink);
  color: var(--fi-primitive-white);
  font-size: 28rpx;
  white-space: nowrap;
  line-height: 1;
}
.primary-action--ghost {
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
}
</style>
