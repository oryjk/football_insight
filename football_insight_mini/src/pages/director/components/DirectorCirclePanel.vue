<template>
  <view class="circle-card">
    <view v-if="loading && !circle" class="circle-state">
      <text class="circle-state__text">正在汇总好友圈共识…</text>
    </view>

    <view v-else-if="error && !circle" class="circle-state">
      <text class="circle-state__text circle-state__text--error">{{ error }}</text>
      <button class="circle-btn" hover-class="circle-pressed" @tap="emit('retry')">重试</button>
    </view>

    <template v-else-if="circle">
      <view class="circle-head">
        <text class="circle-title">{{ circle.team_name }} · 好友圈共识</text>
        <text class="circle-count">{{ circle.member_count }} 位总监参与</text>
      </view>

      <view v-if="!circle.unlocked" class="lock-box">
        <text class="lock-box__title">还差 1 位好友解锁本圈子共识</text>
        <text class="lock-box__desc">
          把方案转发到微信群，只要有 2 位及以上群友完成选人，立刻揭晓群里最受欢迎的 5 外援与最高离队率球员！
        </text>
        <!-- #ifdef MP-WEIXIN -->
        <button class="circle-btn circle-btn--primary" open-type="share" hover-class="circle-pressed">
          转发到微信群拉人 PK
        </button>
        <!-- #endif -->
        <!-- #ifndef MP-WEIXIN -->
        <button class="circle-btn circle-btn--primary" hover-class="circle-pressed" @tap="emit('share')">
          转发给好友拉人 PK
        </button>
        <!-- #endif -->
      </view>

      <template v-else>
        <view v-if="circle.top_foreign.length" class="rank-block">
          <text class="rank-title">圈子最受欢迎 5 外援</text>
          <view class="rank-list">
            <view v-for="(item, idx) in circle.top_foreign" :key="item.player.key" class="rank-row">
              <text class="rank-idx">{{ idx + 1 }}</text>
              <view class="rank-info">
                <text class="rank-name">{{ item.player.player_name }}</text>
                <text class="rank-sub">
                  {{ directorPositionLabel(item.player.position) }} · {{ item.player.current_club }}
                </text>
              </view>
              <text class="rank-rate">{{ item.pick_rate }}%（{{ item.pick_count }}人）</text>
            </view>
          </view>
        </view>

        <view v-if="circle.top_leaves.length" class="rank-block">
          <text class="rank-title">最想送走排行</text>
          <view class="rank-list">
            <view v-for="item in circle.top_leaves" :key="item.player.key" class="rank-row">
              <view class="rank-info">
                <text class="rank-name">{{ item.player.player_name }}</text>
                <text class="rank-sub">{{ directorPositionLabel(item.player.position) }}</text>
              </view>
              <text class="rank-rate rank-rate--leave">离队率 {{ item.leave_rate }}%</text>
            </view>
          </view>
        </view>
      </template>

      <view v-if="circle.plans.length" class="rank-block">
        <text class="rank-title">圈内好友方案（{{ circle.plans.length }}）</text>
        <view class="member-list">
          <view
            v-for="item in circle.plans"
            :key="item.share_code"
            class="member-row"
            :class="{ 'member-row--active': item.share_code === currentShareCode }"
            hover-class="circle-pressed"
            @tap="emit('open-plan', item.share_code)"
          >
            <view class="member-avatar">
              <FiAvatar :src="item.author.avatar_url" :size="96" :name="item.author.nickname" />
            </view>
            <view class="member-info">
              <view class="member-top">
                <text class="member-name">{{ item.author.nickname }}</text>
                <text class="member-style">{{ item.style_tag }}</text>
              </view>
              <text class="member-meta">
                {{ item.manifesto || `${item.foreign_count} 外援方案 · 认同 ${item.likes_count}` }}
              </text>
            </view>
            <text class="member-link">{{ item.share_code === currentShareCode ? '当前' : '查看' }}</text>
          </view>
        </view>
      </view>
    </template>
  </view>
</template>

<script setup lang="ts">
import FiAvatar from '../../../components/FiAvatar.vue'
import type { DirectorCircleResponse } from '../../../types/director'
import { directorPositionLabel } from '../../../utils/directorLinks'

defineProps<{
  circle: DirectorCircleResponse | null
  loading: boolean
  error: string
  currentShareCode: string
}>()

const emit = defineEmits<{
  (e: 'retry'): void
  (e: 'share'): void
  (e: 'open-plan', shareCode: string): void
}>()
</script>

<style scoped lang="css">
.circle-card {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-18);
  padding: var(--fi-space-24);
  border-radius: var(--fi-radius-lg);
  background: var(--fi-color-surface);
  border: var(--fi-border-card);
  box-shadow: var(--fi-shadow-card);
}

.circle-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--fi-space-10);
}

.circle-title {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-30);
  font-weight: var(--fi-weight-extrabold);
}

.circle-count {
  color: var(--fi-color-primary);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
}

.lock-box {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-14);
  padding: var(--fi-space-20);
  border-radius: var(--fi-radius-md);
  background: var(--fi-color-page-soft);
}

.lock-box__title {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-extrabold);
}

.lock-box__desc {
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-normal);
}

.rank-block {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-12);
}

.rank-title {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-extrabold);
}

.rank-list,
.member-list {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-10);
}

.rank-row {
  display: flex;
  align-items: center;
  gap: var(--fi-space-12);
  padding: var(--fi-space-12) var(--fi-space-16);
  border-radius: var(--fi-radius-md);
  background: var(--fi-color-page-soft);
}

.rank-idx {
  width: var(--fi-space-36);
  color: var(--fi-color-primary);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-black);
  text-align: center;
}

.rank-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-4);
}

.rank-name {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
}

.rank-sub {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-20);
}

.rank-rate {
  color: var(--fi-color-seat-desired);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
  font-variant-numeric: tabular-nums;
}

.rank-rate--leave {
  color: var(--fi-color-primary-deep);
}

.member-row {
  display: flex;
  align-items: center;
  gap: var(--fi-space-12);
  padding: var(--fi-space-14) var(--fi-space-16);
  border-radius: var(--fi-radius-md);
  background: var(--fi-color-page-soft);
  border: var(--fi-border-width) solid var(--fi-color-divider-soft);
}

.member-row--active {
  background: var(--fi-color-primary-soft);
  border-color: var(--fi-color-primary);
}

.member-avatar {
  flex-shrink: 0;
  width: var(--fi-space-64);
  height: var(--fi-space-64);
  border-radius: var(--fi-radius-round);
  overflow: hidden;
  background: var(--fi-color-surface);
  font-size: var(--fi-font-22);
}

.member-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-6);
}

.member-top {
  display: flex;
  align-items: center;
  gap: var(--fi-space-8);
}

.member-name {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
}

.member-style {
  padding: var(--fi-space-4) var(--fi-space-10);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-surface);
  color: var(--fi-color-primary);
  font-size: var(--fi-font-20);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

.member-meta {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.member-link {
  flex-shrink: 0;
  color: var(--fi-color-primary);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
}

.circle-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--fi-space-14);
  padding: var(--fi-space-32) var(--fi-space-16);
}

.circle-state__text {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-24);
}

.circle-state__text--error {
  color: var(--fi-color-primary-deep);
}

.circle-btn {
  margin: 0;
  padding: var(--fi-space-16) var(--fi-space-24);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-text-primary);
  color: var(--fi-color-surface);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

.circle-btn::after {
  border: 0;
}

.circle-btn--primary {
  background: var(--fi-color-primary);
}

.circle-pressed {
  opacity: var(--fi-opacity-pressed);
}
</style>
