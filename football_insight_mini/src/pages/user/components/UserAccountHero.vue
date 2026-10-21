<template>
  <view class="account-hero" :class="[heroClass, toneClass]">
    <view class="account-hero__scene">
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
        <view class="member-identity__copy">
          <text class="member-identity__name">{{ displayName }}</text>
          <text class="member-identity__caption">足球洞察 · 球迷档案</text>
        </view>
      </view>

      <view
        class="membership-card"
        :hover-class="canPurchase ? 'membership-card--pressed' : 'none'"
        @click="emit('membership-action')"
      >
        <image
          v-if="membershipTierImage"
          class="membership-card__tier-image"
          :src="membershipTierImage"
          :aria-label="code"
          mode="aspectFit"
        />
        <text v-else class="membership-card__level">{{ code }}</text>
        <view class="membership-card__main">
          <text class="membership-card__title">{{ tierTitle }}</text>
          <text class="membership-card__expire">{{ heroExpiryLabel }}</text>
        </view>
        <view v-if="canPurchase" class="membership-card__action">
          <text>{{ heroActionText }}</text>
          <text class="membership-card__arrow" aria-hidden="true">›</text>
        </view>
        <text v-else class="membership-card__status">已开通</text>
      </view>
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
import { MEMBERSHIP_TIER_IMAGES } from '../membershipImages'

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

const membershipTierImage = computed(() => MEMBERSHIP_TIER_IMAGES[props.code])

// 等级代码由独立图片展示，标题只展示名称。
const tierTitle = computed(() => (props.code === 'V9' ? '至尊会员' : props.badgeLabel.replace(/^V\d+\s*/, '')))

const heroExpiryLabel = computed(() => formatMembershipCardExpiry(props.user.membership_expires_at))

const heroActionText = computed(() => (props.canRenewV9 ? '续费' : '升级'))
</script>

<style scoped lang="css">
.account-hero {
  position: relative;
  z-index: 1;
  overflow: hidden;
  border-radius: var(--fi-radius-xl);
  background: var(--fi-color-surface);
  box-shadow: var(--fi-shadow-card);
}

.account-hero__scene {
  position: relative;
  overflow: hidden;
  padding: var(--fi-space-32) var(--fi-space-28) 0;
  border-radius: 0 0 50% 50% / 0 0 var(--fi-component-member-scene-curve) var(--fi-component-member-scene-curve);
  background: var(--fi-color-member-profile-bg);
}

.account-hero__background,
.account-hero__shade {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
}

.account-hero__background {
  opacity: 0.1;
}

.account-hero__shade {
  background: var(--fi-color-member-profile-shade);
}

.member-identity {
  position: relative;
  padding: var(--fi-space-12) var(--fi-space-8) var(--fi-space-32);
  display: flex;
  align-items: center;
  gap: var(--fi-space-24);
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
  background: var(--fi-color-surface);
  border: var(--fi-border-width) solid var(--fi-color-surface);
  box-shadow: var(--fi-shadow-soft);
  box-sizing: border-box;
}

.member-identity__avatar-image {
  width: 100%;
  height: 100%;
}

.member-identity__avatar-fallback {
  color: var(--fi-color-member-ribbon-text);
  font-size: var(--fi-font-36);
  font-weight: var(--fi-weight-bold);
}

.member-identity__copy {
  min-width: 0;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-12);
}

.member-identity__name {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-34);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-snug);
  word-break: break-all;
}

.member-identity__caption {
  color: var(--fi-color-text-caption);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-normal);
}

.membership-card {
  position: relative;
  min-height: 136rpx;
  padding: var(--fi-space-24) var(--fi-space-24) var(--fi-space-40);
  display: flex;
  align-items: center;
  gap: var(--fi-space-16);
  overflow: hidden;
  border-radius: var(--fi-radius-md) var(--fi-radius-md) 0 0;
  background: linear-gradient(105deg, var(--fi-color-member-ribbon-start), var(--fi-color-member-ribbon-end));
  color: var(--fi-color-member-ribbon-text);
  box-sizing: border-box;
}

.membership-card::before {
  content: '';
  position: absolute;
  top: -140rpx;
  left: -160rpx;
  width: 460rpx;
  height: 320rpx;
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-member-ribbon-sheen);
  transform: rotate(28deg);
  pointer-events: none;
}

.membership-card--pressed {
  opacity: var(--fi-opacity-pressed);
}

.membership-card__tier-image {
  position: relative;
  width: 64rpx;
  height: 48rpx;
  flex-shrink: 0;
}

.membership-card__level {
  position: relative;
  flex-shrink: 0;
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-black);
  line-height: var(--fi-leading-none);
}

.membership-card__main {
  position: relative;
  min-width: 0;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-8);
}

.membership-card__title {
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-snug);
}

.membership-card__expire {
  color: var(--fi-color-member-ribbon-caption);
  font-size: var(--fi-font-20);
  line-height: var(--fi-leading-normal);
}

.membership-card__action {
  position: relative;
  flex-shrink: 0;
  min-height: 64rpx;
  display: flex;
  align-items: center;
  gap: var(--fi-space-12);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-snug);
}

.membership-card__arrow {
  font-size: var(--fi-font-32);
  line-height: var(--fi-leading-none);
}

.membership-card__status {
  position: relative;
  flex-shrink: 0;
  color: var(--fi-color-member-ribbon-caption);
  font-size: var(--fi-font-22);
}

.member-invite {
  position: relative;
  padding: var(--fi-space-20) var(--fi-space-32) var(--fi-space-24);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-16);
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
  color: var(--fi-color-text-caption);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-normal);
}

.member-invite__code {
  color: var(--fi-color-text-primary);
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
  border: var(--fi-border-divider);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-surface);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-snug);
  box-sizing: border-box;
}

.member-invite__copy--pressed {
  background: var(--fi-color-page);
}

.member-invite__copy::after {
  border: none;
}
</style>
