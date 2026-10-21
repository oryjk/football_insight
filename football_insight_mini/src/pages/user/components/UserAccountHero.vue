<template>
  <view class="account-hero" :class="[heroClass, toneClass]">
    <image class="account-hero__background" :src="memberStadiumBackground" mode="aspectFill" aria-hidden="true" />
    <view class="account-hero__shade" aria-hidden="true" />

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
      <text class="member-identity__name">{{ displayName }}</text>
      <text class="member-identity__label">球迷会员</text>
    </view>

    <view
      class="membership-card"
      :hover-class="canPurchase ? 'membership-card--pressed' : 'none'"
      @click="emit('membership-action')"
    >
      <view class="membership-card__main">
        <view class="membership-card__heading">
          <text class="membership-card__level">{{ code }}</text>
          <text class="membership-card__title">{{ tierTitle }}</text>
        </view>
        <text class="membership-card__expire">{{ heroExpiryLabel }}</text>
      </view>
      <view v-if="canPurchase" class="membership-card__action">
        <text>{{ heroActionText }}</text>
        <text class="membership-card__arrow" aria-hidden="true">›</text>
      </view>
      <text v-else class="membership-card__status">已开通</text>
    </view>

    <view v-if="inviteCode" class="member-invite">
      <view class="member-invite__body">
        <text class="member-invite__label">我的邀请码</text>
        <text class="member-invite__code">{{ inviteCode }}</text>
      </view>
      <button class="member-invite__copy" hover-class="member-invite__copy--pressed" @click="emit('copy-invite')">
        一键复制
      </button>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import memberStadiumBackground from '../../../static/user/member-stadium-v2.jpg'
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

// 等级代码由徽章展示，标题只保留会员名称。
const tierTitle = computed(() => (props.code === 'V9' ? '至尊会员' : props.badgeLabel.replace(/^V\d+\s*/, '')))

const heroExpiryLabel = computed(() => formatMembershipCardExpiry(props.user.membership_expires_at))

const heroActionText = computed(() => (props.canRenewV9 ? '续费' : '升级'))
</script>

<style scoped lang="css">
.account-hero {
  --member-accent: var(--fi-color-member-card-accent);
  position: relative;
  z-index: 1;
  overflow: hidden;
  border-radius: var(--fi-radius-xl);
  border: var(--fi-border-width) solid var(--fi-color-member-pass-border);
  background: var(--fi-color-member-card-bg);
  box-shadow: var(--fi-shadow-card-strong);
}

.tier-tone--v1 {
  --member-accent: var(--fi-color-member-pass-silver);
}

.tier-tone--v2 {
  --member-accent: var(--fi-color-member-pass-champagne);
}

.account-hero__background,
.account-hero__shade {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
}

.account-hero__shade {
  background: var(--fi-color-member-pass-shade);
}

.member-identity {
  position: relative;
  padding: var(--fi-space-28) var(--fi-space-28) 0;
  display: flex;
  align-items: center;
  gap: var(--fi-space-16);
}

.member-identity__avatar {
  width: 64rpx;
  height: 64rpx;
  flex-shrink: 0;
  border-radius: var(--fi-radius-round);
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--fi-color-member-pass-avatar);
  border: var(--fi-border-width) solid var(--fi-color-member-pass-border);
  box-sizing: border-box;
}

.member-identity__avatar-image {
  width: 100%;
  height: 100%;
}

.member-identity__avatar-fallback {
  color: var(--fi-color-surface);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-bold);
}

.member-identity__name {
  min-width: 0;
  flex: 1;
  color: var(--fi-color-surface);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-normal);
  word-break: break-all;
}

.member-identity__label {
  flex-shrink: 0;
  color: var(--fi-color-member-pass-muted);
  font-size: var(--fi-font-20);
  letter-spacing: 3rpx;
}

.membership-card {
  position: relative;
  padding: var(--fi-space-40) var(--fi-space-28) var(--fi-space-32);
  display: flex;
  align-items: center;
  gap: var(--fi-space-20);
  box-sizing: border-box;
}

.membership-card--pressed {
  opacity: var(--fi-opacity-pressed);
}

.membership-card__main {
  min-width: 0;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-12);
}

.membership-card__heading {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--fi-space-12);
  color: var(--member-accent);
}

.membership-card__level {
  font-size: var(--fi-font-44);
  font-weight: var(--fi-weight-black);
  font-style: italic;
  line-height: var(--fi-leading-tight);
}

.membership-card__title {
  font-size: var(--fi-font-36);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-snug);
  letter-spacing: 2rpx;
}

.membership-card__expire {
  color: var(--fi-color-member-pass-muted);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-normal);
}

.membership-card__action {
  flex-shrink: 0;
  min-width: 112rpx;
  min-height: 60rpx;
  padding: var(--fi-space-12) var(--fi-space-20);
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--fi-space-12);
  border-radius: var(--fi-radius-round);
  background: linear-gradient(110deg, var(--fi-color-member-pass-shine), var(--member-accent));
  color: var(--fi-color-member-card-bg);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-snug);
  box-sizing: border-box;
}

.membership-card__arrow {
  font-size: var(--fi-font-32);
  line-height: var(--fi-leading-none);
}

.membership-card__status {
  flex-shrink: 0;
  color: var(--member-accent);
  font-size: var(--fi-font-22);
}

.member-invite {
  position: relative;
  margin: 0 var(--fi-space-28);
  padding: var(--fi-space-18) 0 var(--fi-space-20);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-16);
  border-top: var(--fi-border-width) solid var(--fi-color-member-pass-border);
}

.member-invite__body {
  min-width: 0;
  flex: 1;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  column-gap: var(--fi-space-16);
  row-gap: var(--fi-space-6);
}

.member-invite__label {
  color: var(--fi-color-member-pass-muted);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-normal);
}

.member-invite__code {
  color: var(--fi-color-surface);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-snug);
  letter-spacing: 2rpx;
  word-break: break-all;
}

.member-invite__copy {
  flex-shrink: 0;
  min-height: 52rpx;
  margin: 0;
  padding: var(--fi-space-10) var(--fi-space-16);
  border: var(--fi-border-width) solid var(--fi-color-member-pass-border);
  border-radius: var(--fi-radius-round);
  background: transparent;
  color: var(--fi-color-member-pass-muted);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-snug);
  box-sizing: border-box;
}

.member-invite__copy--pressed {
  background: var(--fi-color-member-pass-avatar);
}

.member-invite__copy::after {
  border: none;
}
</style>
