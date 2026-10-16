<template>
  <view class="hero-card account-hero" :class="[heroClass, toneClass]">
    <view class="member-identity">
      <view class="member-identity__avatar">
        <image
          v-if="user.avatar_url"
          :src="user.avatar_url"
          mode="aspectFill"
          class="member-identity__avatar-image"
        />
        <text v-else class="member-identity__avatar-fallback">{{ avatarFallbackLabel }}</text>
      </view>

      <view class="member-identity__copy">
        <text class="member-identity__name">{{ displayName }}</text>
        <text class="member-identity__tag">{{ tagLabel }}</text>
      </view>
    </view>

    <view
      class="membership-card"
      :class="{ 'membership-card--actionable': canPurchase }"
      @click="emit('membership-action')"
    >
      <view class="membership-card__main">
        <text class="membership-card__label">会员有效期</text>
        <text class="membership-card__expire">{{ heroExpiryLabel }}</text>
      </view>
      <view v-if="canPurchase" class="membership-card__action">{{ heroActionText }}</view>
      <text v-else class="membership-card__status">已开通</text>
    </view>

    <view v-if="inviteCode" class="member-invite">
      <view class="member-invite__body">
        <text class="member-invite__label">我的邀请码</text>
        <text class="member-invite__code">{{ inviteCode }}</text>
      </view>
      <button class="member-invite__copy" @click="emit('copy-invite')">一键复制</button>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { CurrentUser } from '../../../types/auth'
import { formatMembershipCardExpiry } from '../helpers'

const props = defineProps<{
  user: CurrentUser
  heroClass: string
  toneClass: string
  code: string
  badgeLabel: string
  canPurchase: boolean
  canRenewV9: boolean
  inviteCode: string
}>()

const emit = defineEmits<{
  (e: 'membership-action'): void
  (e: 'copy-invite'): void
}>()

const displayName = computed(() => props.user.display_name?.trim() || '会员')

const avatarFallbackLabel = computed(() => displayName.value.slice(0, 1).toUpperCase())

// 等级只在这里出现一次：会员卡里不再重复写等级或"恭喜您…"整句。
const tagLabel = computed(() => (props.code === 'V9' ? 'V9 至尊会员' : props.badgeLabel))

const heroExpiryLabel = computed(() => formatMembershipCardExpiry(props.user.membership_expires_at))

const heroActionText = computed(() => (props.canRenewV9 ? '续费' : '升级'))
</script>

<style scoped lang="css">
.hero-card {
  position: relative;
  z-index: 1;
  padding: var(--fi-space-24);
  border-radius: var(--fi-radius-xl);
  border: var(--fi-border-width) solid var(--fi-color-divider);
  background: var(--fi-color-surface);
  box-shadow: 0 16rpx 40rpx rgba(var(--fi-primitive-ink-rgb), 0.06);
  animation: fi-fade-in-up 520ms cubic-bezier(0.22, 1, 0.36, 1) both;
}

.member-identity {
  display: flex;
  align-items: center;
  gap: var(--fi-space-20);
}

.member-identity__avatar {
  width: var(--fi-avatar-md);
  height: var(--fi-avatar-md);
  flex-shrink: 0;
  border-radius: var(--fi-radius-round);
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--fi-color-page);
  border: var(--fi-border-width) solid var(--fi-color-divider);
  box-sizing: border-box;
}

.member-identity__avatar-image {
  width: 100%;
  height: 100%;
}

.member-identity__avatar-fallback {
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-36);
  font-weight: var(--fi-weight-extrabold);
}

.member-identity__copy {
  min-width: 0;
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: var(--fi-space-10);
}

.member-identity__name {
  max-width: 100%;
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-34);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-tight);
  word-break: break-all;
}

.member-identity__tag {
  padding: var(--fi-space-6) var(--fi-space-16);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

.membership-card {
  margin-top: var(--fi-space-24);
  padding: var(--fi-space-28);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-24);
  border-radius: var(--fi-radius-lg);
  background: var(--fi-color-member-card-bg);
  box-sizing: border-box;
}

.membership-card__main {
  min-width: 0;
  flex: 1;
  display: grid;
  gap: var(--fi-space-14);
}

.membership-card__label {
  color: rgba(var(--fi-primitive-white-rgb), 0.62);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

.membership-card__expire {
  color: var(--fi-color-member-card-accent);
  font-size: var(--fi-font-40);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-none);
}

.membership-card__action {
  flex-shrink: 0;
  min-width: 128rpx;
  height: 68rpx;
  padding: 0 var(--fi-space-28);
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-member-card-accent);
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-none);
  box-sizing: border-box;
}

.membership-card__status {
  flex-shrink: 0;
  color: rgba(var(--fi-primitive-white-rgb), 0.62);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
}

.member-invite {
  margin-top: var(--fi-space-20);
  padding: var(--fi-space-16) var(--fi-space-20);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-16);
  border-radius: var(--fi-radius-md);
  background: var(--fi-color-page);
}

.member-invite__body {
  min-width: 0;
  flex: 1;
  display: grid;
  gap: var(--fi-space-6);
}

.member-invite__label {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

.member-invite__code {
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-snug);
  word-break: break-all;
}

.member-invite__copy {
  flex-shrink: 0;
  min-width: 148rpx;
  height: 62rpx;
  margin: 0;
  padding: 0 var(--fi-space-22);
  border: none;
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-text-primary);
  color: var(--fi-color-surface);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
  line-height: 62rpx;
  box-shadow: none;
}

.member-invite__copy::after {
  border: none;
}
</style>
