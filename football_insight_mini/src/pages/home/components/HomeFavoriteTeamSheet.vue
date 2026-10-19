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
  font-size: var(--fi-font-24);
  font-weight: 700;
  letter-spacing: 1rpx;
  padding: var(--fi-space-8) 0 0;
}

.favorite-team-sheet__list {
  margin-top: 20rpx;
  max-height: 52vh;
}
.favorite-team-sheet__row {
  padding: var(--fi-space-18) 0;
  display: flex;
  align-items: center;
  gap: var(--fi-space-16);
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
  font-size: var(--fi-font-36);
  line-height: 1;
  font-weight: 800;
}
.favorite-team-sheet__note {
  display: block;
  margin-top: 6rpx;
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}
.sheet-actions {
  margin-top: 24rpx;
  display: flex;
  gap: var(--fi-space-16);
}
.primary-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: 1;
  padding: var(--fi-space-20) var(--fi-space-30);
  border-radius: var(--fi-radius-round);
  background: var(--fi-primitive-ink);
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-28);
  white-space: nowrap;
  line-height: 1;
}
.primary-action--ghost {
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
}
</style>
