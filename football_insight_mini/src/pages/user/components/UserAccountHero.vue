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
  --member-highlight: rgba(255, 232, 179, 0.92);
  --member-wash: rgba(255, 243, 214, 0.68);
  --member-border: rgba(235, 211, 155, 0.92);
  --member-eyebrow: #b91c1c;
  --member-accent: #dc2626;
  --member-accent-strong: #991b1b;
  --member-badge-from: #ef4444;
  --member-badge-to: #b91c1c;
  --member-soft-surface: rgba(255, 241, 242, 0.72);
  --member-soft-border: rgba(254, 205, 211, 0.94);
  --member-medal-ink: #b91c1c;
  --member-medal-crown: rgba(220, 38, 38, 0.62);
  --member-glow: rgba(220, 38, 38, 0.18);
  --member-privilege-border: rgba(254, 205, 211, 0.76);
  --member-privilege-bg: rgba(255, 241, 242, 0.88);
  --member-privilege-icon-bg: rgba(220, 38, 38, 0.1);
  --member-privilege-icon-color: #b91c1c;
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

.account-hero.tier-tone--v1,
.privilege-panel.tier-tone--v1 {
  --member-highlight: rgba(206, 220, 246, 0.9);
  --member-wash: rgba(228, 236, 250, 0.76);
  --member-border: rgba(197, 209, 232, 0.94);
  --member-eyebrow: #66799a;
  --member-accent: #7f95bd;
  --member-accent-strong: #5d749c;
  --member-badge-from: #90a5ca;
  --member-badge-to: #697fa7;
  --member-soft-surface: rgba(236, 242, 252, 0.84);
  --member-soft-border: rgba(203, 214, 235, 0.94);
  --member-medal-ink: #6a82ab;
  --member-medal-crown: rgba(120, 144, 181, 0.7);
  --member-glow: rgba(123, 148, 193, 0.2);
  --member-privilege-border: rgba(212, 220, 239, 0.84);
  --member-privilege-bg: rgba(239, 244, 252, 0.96);
  --member-privilege-icon-bg: rgba(121, 146, 189, 0.14);
  --member-privilege-icon-color: #6b82ab;
}

.account-hero.tier-tone--v3,
.privilege-panel.tier-tone--v3 {
  --member-highlight: rgba(255, 232, 179, 0.92);
  --member-wash: rgba(255, 243, 214, 0.68);
  --member-border: rgba(235, 211, 155, 0.92);
  --member-eyebrow: #b91c1c;
  --member-accent: #dc2626;
  --member-accent-strong: #991b1b;
  --member-badge-from: #ef4444;
  --member-badge-to: #b91c1c;
  --member-soft-surface: rgba(255, 241, 242, 0.72);
  --member-soft-border: rgba(254, 205, 211, 0.94);
  --member-medal-ink: #b91c1c;
  --member-medal-crown: rgba(220, 38, 38, 0.62);
  --member-glow: rgba(220, 38, 38, 0.18);
  --member-privilege-border: rgba(254, 205, 211, 0.76);
  --member-privilege-bg: rgba(255, 241, 242, 0.88);
  --member-privilege-icon-bg: rgba(220, 38, 38, 0.1);
  --member-privilege-icon-color: #b91c1c;
}

.account-hero.tier-tone--v4,
.privilege-panel.tier-tone--v4 {
  --member-highlight: rgba(254, 202, 202, 0.92);
  --member-wash: rgba(255, 235, 229, 0.74);
  --member-border: rgba(236, 188, 173, 0.92);
  --member-eyebrow: #ab5a4d;
  --member-accent: #d47a5d;
  --member-accent-strong: #af5843;
  --member-badge-from: #de8769;
  --member-badge-to: #b55c45;
  --member-soft-surface: rgba(255, 241, 236, 0.78);
  --member-soft-border: rgba(239, 200, 187, 0.92);
  --member-medal-ink: #c55f45;
  --member-medal-crown: rgba(205, 102, 78, 0.72);
  --member-glow: rgba(213, 122, 93, 0.22);
  --member-privilege-border: rgba(238, 205, 195, 0.8);
  --member-privilege-bg: rgba(252, 239, 235, 0.96);
  --member-privilege-icon-bg: rgba(212, 122, 93, 0.12);
  --member-privilege-icon-color: #c06049;
}

.account-hero.tier-tone--v5,
.privilege-panel.tier-tone--v5 {
  --member-highlight: rgba(228, 214, 255, 0.92);
  --member-wash: rgba(243, 236, 255, 0.76);
  --member-border: rgba(211, 194, 239, 0.92);
  --member-eyebrow: #7c58b2;
  --member-accent: #9468cf;
  --member-accent-strong: #7045ac;
  --member-badge-from: #a27ddd;
  --member-badge-to: #754db5;
  --member-soft-surface: rgba(244, 238, 255, 0.8);
  --member-soft-border: rgba(217, 203, 240, 0.94);
  --member-medal-ink: #8458c3;
  --member-medal-crown: rgba(139, 93, 199, 0.72);
  --member-glow: rgba(148, 104, 207, 0.22);
  --member-privilege-border: rgba(221, 208, 244, 0.82);
  --member-privilege-bg: rgba(246, 241, 255, 0.96);
  --member-privilege-icon-bg: rgba(148, 104, 207, 0.12);
  --member-privilege-icon-color: #8257c1;
}

.account-hero.tier-tone--v6,
.privilege-panel.tier-tone--v6 {
  --member-highlight: rgba(205, 245, 223, 0.92);
  --member-wash: rgba(231, 251, 240, 0.76);
  --member-border: rgba(178, 223, 200, 0.92);
  --member-eyebrow: #2f8b65;
  --member-accent: #47b283;
  --member-accent-strong: #2d8c63;
  --member-badge-from: #55bf8f;
  --member-badge-to: #2d8b63;
  --member-soft-surface: rgba(236, 252, 243, 0.8);
  --member-soft-border: rgba(189, 228, 207, 0.94);
  --member-medal-ink: #36956d;
  --member-medal-crown: rgba(61, 156, 114, 0.72);
  --member-glow: rgba(71, 178, 131, 0.22);
  --member-privilege-border: rgba(196, 231, 213, 0.82);
  --member-privilege-bg: rgba(240, 253, 246, 0.96);
  --member-privilege-icon-bg: rgba(71, 178, 131, 0.12);
  --member-privilege-icon-color: #368f69;
}

.account-hero.tier-tone--v7,
.privilege-panel.tier-tone--v7 {
  --member-highlight: rgba(209, 226, 255, 0.92);
  --member-wash: rgba(233, 242, 255, 0.76);
  --member-border: rgba(187, 209, 241, 0.92);
  --member-eyebrow: #3a69b2;
  --member-accent: #4c84d7;
  --member-accent-strong: #315fa7;
  --member-badge-from: #5c93e3;
  --member-badge-to: #3465b3;
  --member-soft-surface: rgba(238, 245, 255, 0.8);
  --member-soft-border: rgba(193, 214, 243, 0.94);
  --member-medal-ink: #3e70c0;
  --member-medal-crown: rgba(59, 120, 196, 0.72);
  --member-glow: rgba(76, 132, 215, 0.22);
  --member-privilege-border: rgba(201, 219, 244, 0.82);
  --member-privilege-bg: rgba(239, 245, 255, 0.96);
  --member-privilege-icon-bg: rgba(76, 132, 215, 0.12);
  --member-privilege-icon-color: #3f73c2;
}

.account-hero.tier-tone--v8,
.privilege-panel.tier-tone--v8 {
  --member-highlight: rgba(255, 212, 220, 0.92);
  --member-wash: rgba(255, 236, 239, 0.76);
  --member-border: rgba(237, 190, 198, 0.92);
  --member-eyebrow: #ad4b58;
  --member-accent: #d36576;
  --member-accent-strong: #b14353;
  --member-badge-from: #db7384;
  --member-badge-to: #b44757;
  --member-soft-surface: rgba(255, 239, 242, 0.8);
  --member-soft-border: rgba(239, 200, 207, 0.94);
  --member-medal-ink: #c54f62;
  --member-medal-crown: rgba(203, 84, 77, 0.72);
  --member-glow: rgba(211, 101, 118, 0.22);
  --member-privilege-border: rgba(239, 206, 213, 0.82);
  --member-privilege-bg: rgba(255, 241, 244, 0.96);
  --member-privilege-icon-bg: rgba(211, 101, 118, 0.12);
  --member-privilege-icon-color: #c24f61;
}

.account-hero.tier-tone--v9,
.privilege-panel.tier-tone--v9 {
  --member-highlight: rgba(254, 202, 202, 0.94);
  --member-wash: rgba(254, 226, 226, 0.78);
  --member-border: rgba(254, 205, 211, 0.94);
  --member-eyebrow: #b91c1c;
  --member-accent: #dc2626;
  --member-accent-strong: #991b1b;
  --member-badge-from: #ef4444;
  --member-badge-to: #b91c1c;
  --member-soft-surface: rgba(255, 241, 242, 0.84);
  --member-soft-border: rgba(254, 205, 211, 0.96);
  --member-medal-ink: #b91c1c;
  --member-medal-crown: rgba(220, 38, 38, 0.62);
  --member-glow: rgba(220, 38, 38, 0.18);
  --member-privilege-border: rgba(254, 205, 211, 0.84);
  --member-privilege-bg: rgba(255, 241, 242, 0.92);
  --member-privilege-icon-bg: rgba(220, 38, 38, 0.1);
  --member-privilege-icon-color: #b91c1c;
}
</style>
