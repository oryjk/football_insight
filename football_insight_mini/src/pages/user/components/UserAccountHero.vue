<template>
  <view class="hero-card account-hero" :class="[heroClass, toneClass]">
    <image class="account-hero__dots account-hero__dots--top" :src="memberCardDotsImage" mode="aspectFill" />
    <image class="account-hero__dots account-hero__dots--bottom" :src="memberCardDotsImage" mode="aspectFill" />

    <view class="hero-card__top hero-card__top--member">
      <view class="account-hero__eyebrow">
        <image class="account-hero__eyebrow-icon" :src="diamondIcon" mode="aspectFit" />
        <text class="account-hero__eyebrow-text">会员中心</text>
      </view>
    </view>

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
        <text class="member-identity__hello">{{ greetingText }}</text>
        <text class="member-identity__desc">{{ heroDescription }}</text>
      </view>
    </view>

    <view
      class="membership-card"
      :class="{ 'membership-card--actionable': canPurchase }"
      @click="emit('membership-action')"
    >
      <view class="membership-card__content">
        <view class="membership-card__main">
          <text class="membership-card__title">{{ heroTitle }}</text>
          <text class="membership-card__expire">{{ heroExpiryLabel }}</text>
        </view>
        <view class="membership-card__action">
          <text class="membership-card__action-text">{{ heroActionText }}</text>
          <text v-if="canPurchase" class="membership-card__action-arrow">›</text>
        </view>
      </view>
      <view class="membership-card__level">{{ code }}</view>
    </view>

    <view v-if="inviteCode" class="profile-banner__invite profile-banner__invite--hero">
      <view class="profile-banner__invite-body">
        <text class="profile-banner__invite-label">我的邀请码</text>
        <text class="profile-banner__invite-code">{{ inviteCode }}</text>
      </view>
      <button class="profile-banner__invite-copy" :class="toneClass" @click="emit('copy-invite')">一键复制</button>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import diamondIcon from '../../../static/user/diamond.svg'
import memberCardDotsImage from '../../../static/user/member-card-dots.png'
import type { CurrentUser } from '../../../types/auth'
import { formatMembershipExpiryLabel } from '../helpers'

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

const avatarFallbackLabel = computed(() => {
  const name = props.user.display_name?.trim() ?? ''
  if (!name) {
    return '会'
  }

  return name.slice(0, 1).toUpperCase()
})

const greetingText = computed(() => {
  const displayName = props.user.display_name?.trim()
  return displayName ? `${displayName}，您好！` : '亲爱的会员，您好！'
})

const heroTitle = computed(() =>
  props.code === 'V9' ? '至尊会员' : `${props.code} 会员`,
)

const heroDescription = computed(() => {
  if (props.code === 'V9') {
    return '恭喜您，您已经是我们的至尊会员'
  }

  return `恭喜您，您已经是${props.badgeLabel}`
})

const heroExpiryLabel = computed(() => {
  const label = formatMembershipExpiryLabel(props.user.membership_expires_at)

  if (label === '长期有效' || label === '已过期') {
    return label
  }

  return `${label.replace(/^有效至\s*/, '')} 到期`
})

const heroActionText = computed(() => {
  if (!props.canPurchase) {
    return '已开通'
  }

  return props.canRenewV9 ? '续费' : '升级'
})
</script>

<style scoped lang="css">
.hero-card, .panel {
  position: relative;
  z-index: 1;
  background: rgba(255,255,255,0.72);
  border-radius: var(--fi-radius-xl);
  padding: 20rpx;
  border: 2rpx solid rgba(255,255,255,0.55);
  box-shadow: 0 20rpx 48rpx rgba(26,28,36,0.06);
  backdrop-filter: blur(18rpx);
  -webkit-backdrop-filter: blur(18rpx);
}

.skeleton-account-hero {
  min-height: 330rpx;
}

.account-hero {
  background: var(--fi-primitive-white);
  border-width: 1rpx;
  border-color: rgba(229, 232, 238, 0.96);
  backdrop-filter: none;
  -webkit-backdrop-filter: none;
  animation: fi-fade-in-up 520ms cubic-bezier(0.22, 1, 0.36, 1) both;
}

.account-hero--v3 {
  background:
    radial-gradient(circle at top right, rgba(224, 195, 131, 0.34), transparent 40%),
    linear-gradient(180deg, rgba(255, 251, 241, 0.78), rgba(249, 244, 232, 0.48));
  border-color: rgba(224, 195, 131, 0.32);
}

.account-hero--v2 {
  background:
    radial-gradient(circle at top right, rgba(142, 190, 174, 0.30), transparent 40%),
    linear-gradient(180deg, rgba(244, 252, 248, 0.78), rgba(235, 246, 240, 0.48));
  border-color: rgba(142, 190, 174, 0.32);
}

.account-hero--v1 {
  background:
    radial-gradient(circle at top right, rgba(189, 198, 216, 0.28), transparent 40%),
    linear-gradient(180deg, rgba(250, 251, 255, 0.78), rgba(242, 245, 252, 0.48));
  border-color: rgba(189, 198, 216, 0.32);
}

.account-hero--guest {
  margin-top: 0;
  padding: 22rpx 24rpx;
  border-radius: var(--fi-radius-lg);
  background: rgba(21, 23, 29, 0.96);
  border: 0;
  box-shadow: 0 20rpx 46rpx rgba(16, 18, 24, 0.24);
  backdrop-filter: none;
  -webkit-backdrop-filter: none;
}

.page-root--guest .account-hero {
  animation-delay: 90ms;
}

.hero-card__top, .section-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.hero-card__top, .section-heading { align-items: flex-start; gap: 12rpx; }

.hero-card__title, .section-title {
  display: block;
  margin-top: 10rpx;
  color: #2a2c31;
  font-size: var(--fi-font-48);
  line-height: 1.08;
  font-weight: 800;
}

.hero-card__summary, .account-form-panel__summary, .user-panel__summary {
  display: block;
  margin-top: 18rpx;
  color: #6b707b;
  font-size: var(--fi-font-28);
  line-height: 1.7;
}

.profile-banner__invite {
  margin-top: 16rpx;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16rpx;
  padding: 16rpx 18rpx;
  border-radius: var(--fi-radius-md);
  border: 2rpx solid var(--page-border-warm);
  background: linear-gradient(180deg, rgba(255,255,255,0.88), rgba(249, 244, 233, 0.9));
}

.profile-banner__invite-body {
  min-width: 0;
  flex: 1;
  display: grid;
  gap: 6rpx;
}

.profile-banner__invite-label {
  color: #8e845f;
  font-size: 21rpx;
  font-weight: 700;
  line-height: 1;
}

.profile-banner__invite-code {
  color: var(--fi-primitive-ink);
  font-size: var(--fi-font-28);
  font-weight: 800;
  line-height: 1.2;
  word-break: break-all;
}

.profile-banner__invite-copy {
  flex-shrink: 0;
  padding: 0 22rpx;
  height: 64rpx;
  line-height: 64rpx;
  border-radius: var(--fi-radius-round);
  border: 2rpx solid rgba(254, 205, 211, 0.96);
  background: linear-gradient(180deg, rgba(255, 241, 242, 0.98), rgba(254, 226, 226, 0.98));
  color: #991b1b;
  font-size: var(--fi-font-24);
  font-weight: 700;
  box-sizing: border-box;
}

.hero-card__badge, .meta-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  white-space: nowrap;
  line-height: 1;
  box-sizing: border-box;
  padding: 14rpx 24rpx;
  border-radius: var(--fi-radius-round);
  border: 2rpx solid var(--page-border);
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
}

.account-hero {
  position: relative;
  overflow: hidden;
  padding: 24rpx;
  border-color: rgba(232, 234, 239, 0.92);
  background: rgba(255, 255, 255, 0.94);
  box-shadow: 0 16rpx 40rpx rgba(20, 23, 31, 0.06);
}

.account-hero::before {
  display: none;
}

.page-root--guest .account-hero::before,
.page-root--guest .account-hero--guest::before {
  display: none;
}

.page-root:not(.page-root--guest) .account-hero--guest {
  border-color: rgba(229, 232, 238, 0.96);
  box-shadow: 0 18rpx 40rpx rgba(25, 28, 36, 0.06);
}

.page-root:not(.page-root--guest) .account-hero--guest::before {
  display: none;
}

.account-hero__dots {
  position: absolute;
  z-index: 0;
  width: 300rpx;
  height: 184rpx;
  opacity: 0.42;
  pointer-events: none;
}

.account-hero__dots--top {
  top: -44rpx;
  right: -48rpx;
}

.account-hero__dots--bottom {
  left: -78rpx;
  bottom: 82rpx;
  opacity: 0.22;
  transform: scaleX(-1);
}

.hero-card__top--member {
  position: relative;
  z-index: 1;
  justify-content: flex-start;
}

.account-hero__eyebrow {
  display: inline-flex;
  align-items: center;
  gap: 10rpx;
  color: #8b92a0;
}

.account-hero__eyebrow-icon {
  width: 26rpx;
  height: 26rpx;
  flex-shrink: 0;
}

.account-hero__eyebrow-text {
  font-size: var(--fi-font-22);
  font-weight: 700;
  line-height: 1;
}

.member-identity {
  position: relative;
  z-index: 1;
  margin-top: 22rpx;
  display: flex;
  align-items: center;
  gap: 18rpx;
}

.member-identity__avatar {
  width: 104rpx;
  height: 104rpx;
  flex-shrink: 0;
  border-radius: var(--fi-radius-round);
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #eef1f5;
  border: 5rpx solid rgba(255, 255, 255, 0.98);
  box-shadow: 0 12rpx 26rpx rgba(18, 22, 30, 0.1);
}

.member-identity__avatar-image {
  width: 100%;
  height: 100%;
}

.member-identity__avatar-fallback {
  color: #2c3139;
  font-size: 38rpx;
  font-weight: 900;
}

.member-identity__copy {
  min-width: 0;
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 10rpx;
}

.member-identity__hello {
  max-width: 100%;
  color: #171a20;
  font-size: 34rpx;
  font-weight: 900;
  line-height: 1.1;
  word-break: break-all;
}

.member-identity__desc {
  color: #7b828e;
  font-size: 23rpx;
  font-weight: 600;
  line-height: 1.35;
}

.membership-card {
  position: relative;
  z-index: 1;
  margin-top: 24rpx;
  min-height: 154rpx;
  padding: 28rpx 30rpx;
  border-radius: 30rpx;
  overflow: hidden;
  background:
    radial-gradient(circle at 88% 18%, rgba(232, 196, 119, 0.12), transparent 34%),
    #34313b;
  box-shadow: 0 18rpx 36rpx rgba(22, 24, 31, 0.18);
  box-sizing: border-box;
}

.membership-card::before {
  content: '';
  position: absolute;
  right: -26rpx;
  top: -46rpx;
  width: 230rpx;
  height: 230rpx;
  border-radius: var(--fi-radius-round);
  border: 2rpx solid rgba(221, 186, 111, 0.28);
}

.membership-card--actionable {
  cursor: pointer;
}

.membership-card__content {
  position: relative;
  z-index: 1;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 24rpx;
}

.membership-card__main {
  min-width: 0;
  flex: 1;
  display: grid;
  gap: 14rpx;
}

.membership-card__title {
  color: #e9c477;
  font-size: 42rpx;
  font-weight: 900;
  line-height: 1;
}

.membership-card__expire {
  color: rgba(255, 255, 255, 0.78);
  font-size: 23rpx;
  font-weight: 600;
  line-height: 1;
}

.membership-card__action {
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  gap: 4rpx;
  color: rgba(255, 255, 255, 0.88);
  font-size: 25rpx;
  font-weight: 800;
  line-height: 1;
}

.membership-card__action-arrow {
  font-size: 34rpx;
  font-weight: 700;
  line-height: 1;
}

.membership-card__level {
  position: absolute;
  right: 30rpx;
  bottom: 16rpx;
  z-index: 1;
  color: rgba(232, 196, 119, 0.12);
  font-size: 64rpx;
  font-weight: 900;
  line-height: 1;
  letter-spacing: 0;
}

.profile-banner__invite--hero {
  position: relative;
  z-index: 1;
  margin-top: 20rpx;
  padding: 16rpx 20rpx;
  border-radius: var(--fi-radius-md);
  border: 0;
  background: rgba(246, 247, 249, 0.92);
  backdrop-filter: none;
}

.profile-banner__invite-label {
  color: #8b92a0;
  font-size: var(--fi-font-22);
}

.profile-banner__invite-code {
  font-size: var(--fi-font-26);
}

.profile-banner__invite-copy {
  min-width: 148rpx;
  height: 62rpx;
  line-height: 62rpx;
  border: none;
  background: #15171d;
  color: var(--fi-primitive-white);
  box-shadow: none;
}
</style>
