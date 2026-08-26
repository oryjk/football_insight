<template>
  <view class="page-root" :class="{ 'page-root--guest': isGuestPage }">
    <FiBrandNav open-on-current-page :transparent="isGuestPage" @open-ai="openAiFromBrandNav" />
    <image class="page-bg-img" :src="phoenixStadiumBgImage" mode="aspectFill" :webp="true" />
    <view class="page-bg-fade"></view>
    <view class="page-scroll">
      <view class="page">
      <template v-if="systemConfigUnderReview">
        <view class="hero-card account-hero account-hero--guest">
          <view class="hero-card__top">
            <view>
              <text class="eyebrow">我的</text>
              <text class="hero-card__title">账号</text>
            </view>
          </view>
          <text class="hero-card__summary">当前版本展示基础内容。</text>
        </view>
      </template>

      <template v-else>
        <UserAccountHero
          v-if="currentUser"
          :user="currentUser"
          :hero-class="currentMembershipMeta.heroClass"
          :tone-class="currentMembershipGuide.toneClass"
          :code="currentMembershipMeta.code"
          :badge-label="currentMembershipMeta.badgeLabel"
          :can-purchase="canPurchaseMembership"
          :can-renew-v9="canRenewCurrentV9Membership"
          :invite-code="currentUserInviteCode"
          @membership-action="handleMembershipCardAction"
          @copy-invite="handleCopyInviteCode"
        />

        <UserSkeleton v-if="loading" />

        <template v-else-if="currentUser">
          <UserAccountInfoPanel
            :items="accountInfoItems"
            :icon-map="accountInfoIconMap"
            :email-label="notificationEmailLabel"
            :has-email="!!notificationEmail"
            @edit-email="openNotificationEmailSheet"
          />

          <UserMembershipPanel
            :code="currentMembershipMeta.code"
            :tone-class="currentMembershipGuide.toneClass"
            :benefit-items="membershipBenefitItems"
            :upgrade-steps="upgradeSteps"
            :rule-summary="membershipRuleSummary"
          />

          <view class="account-actions">
            <view class="account-actions__body">
              <text class="account-actions__label">账号管理</text>
              <text class="account-actions__caption">{{ loginPresentation.switchAccountCaption }}</text>
            </view>
            <button class="logout-action" @click="handleLogout">退出登录</button>
          </view>
        </template>

        <!-- #ifdef H5 -->
        <UserPasswordLoginSheet
          v-model:visible="passwordLoginSheetVisible"
          :submitting="passwordLoginSubmitting"
          @submit="handlePasswordLogin"
        />
        <!-- #endif -->

        <UserWechatBindSheet
          :bind-state="miniWechatBindState"
          @close="closeMiniWechatBindSheet"
          @bind="handleMiniWechatBind"
        />

        <UserNotificationEmailSheet
          v-model:visible="notificationEmailSheetVisible"
          :email="notificationEmail"
          :saving="notificationEmailSaving"
          @save="saveNotificationEmail"
        />
      </template>

      <!-- 设置入口常驻：不受审核模式 / 登录状态影响，任何用户可见可点。 -->
      <view class="panel settings-entry-panel" hover-class="settings-entry-panel--pressed" hover-stay-time="100" @click="openSettingsPage">
        <view class="settings-entry-panel__body">
          <text class="settings-entry-panel__title">设置</text>
          <text class="settings-entry-panel__caption">查看当前版本审核状态</text>
        </view>
        <text class="settings-entry-panel__arrow">›</text>
      </view>
    </view>
    </view>

    <FiAiChatSheet
      :visible="aiChatVisible"
      :current-user="currentAiUser"
      :ai-chat-mode="aiPublicConfig?.ai_chat_mode"
      @close="closeAiChat"
    />

    <UserGuestChantWall v-if="showGuestChantWall" />

    <FiLoginFloat
      v-if="showGuestLoginFloat"
      :action-text="loginPresentation.actionText"
      @action="handleLoginAction"
    />
  </view>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { onShow } from '@dcloudio/uni-app'
import FiBrandNav from '../../components/FiBrandNav.vue'
import FiAiChatSheet from '../../components/FiAiChatSheet.vue'
import FiLoginFloat from '../../components/FiLoginFloat.vue'
import UserGuestChantWall from './components/UserGuestChantWall.vue'
import UserSkeleton from './components/UserSkeleton.vue'
import UserAccountHero from './components/UserAccountHero.vue'
import UserAccountInfoPanel from './components/UserAccountInfoPanel.vue'
import UserMembershipPanel from './components/UserMembershipPanel.vue'
import UserPasswordLoginSheet from './components/UserPasswordLoginSheet.vue'
import UserWechatBindSheet from './components/UserWechatBindSheet.vue'
import UserNotificationEmailSheet from './components/UserNotificationEmailSheet.vue'
import badgeCheckIcon from '../../static/user/badge-check.svg'
import calendarCheckIcon from '../../static/user/calendar-check.svg'
import calendarDaysIcon from '../../static/user/calendar-days.svg'
import logInIcon from '../../static/user/log-in.svg'
import { PHOENIX_STADIUM_BG_IMAGE_URL as phoenixStadiumBgImage } from '../../config/assets'
import {
  bindMiniWechatAccount,
  getCurrentUser,
  getNotificationEmail,
  login,
  loginWithMiniWechat,
  logout,
  updateNotificationEmail,
} from '../../api/auth'
import { getPublicSystemConfig } from '../../api/system'
import type { CurrentUser, MiniWechatBindingRequiredResponse } from '../../types/auth'
import type { PublicSystemConfig } from '../../types/system'
import { extractApiErrorMessage } from '../../utils/apiError'
import { getAccessToken } from '../../utils/authStorage'
import { loadSystemConfigUnderReview } from '../../utils/systemConfig'
import { reportPageActivity } from '../../utils/userActivity'
import {
  buildUserAccountInfoItems,
  buildUserBenefitItems,
  buildUserUpgradeSteps,
  canShowMembershipPurchaseEntry,
  formatMembershipExpiryLabel,
  formatNotificationEmailLabel,
  resolveCurrentUserInviteCode,
  resolveUserMembershipMeta,
  type UserAccountInfoItem,
  type UserBenefitItem,
  type UserUpgradeStep,
} from './helpers'
import {
  buildMembershipTierGuides,
  resolveMembershipTierGuide,
  type MembershipTierGuide,
} from '../../utils/membershipRules'
import { consumePostLoginRedirect, navigateToPostLoginTarget } from '../../utils/postLoginRedirect'
import { useAiChatSheet } from '../../composables/useAiChatSheet'
import { buildPasswordLoginPayload, resolveUserLoginPresentation } from './loginHelpers'

const hasLocalAccessToken = ref(Boolean(getAccessToken()))
const loading = ref(hasLocalAccessToken.value)
const currentUser = ref<CurrentUser | null>(null)
const publicConfig = ref<PublicSystemConfig | null>(null)
const systemConfigUnderReview = ref(false)
const notificationEmail = ref('')
const notificationEmailSheetVisible = ref(false)
const notificationEmailSaving = ref(false)
const {
  aiChatVisible,
  currentAiUser,
  aiPublicConfig,
  openAiChat,
  closeAiChat,
} = useAiChatSheet()

const miniWechatBindState = ref<MiniWechatBindingRequiredResponse | null>(null)
const passwordLoginSheetVisible = ref(false)
const passwordLoginSubmitting = ref(false)

const isH5 =
  // #ifdef H5
  true
  // #endif
  // #ifndef H5
  false
  // #endif

const loginPresentation = resolveUserLoginPresentation(isH5)

const accountInfoIconMap: Record<UserAccountInfoItem['iconName'], string> = {
  'badge-check': badgeCheckIcon,
  'calendar-days': calendarDaysIcon,
  'calendar-check': calendarCheckIcon,
  'log-in': logInIcon,
}

const isGuestPage = computed(() => !systemConfigUnderReview.value && (!hasLocalAccessToken.value || (!loading.value && !currentUser.value)))
const hasOpenSheet = computed(() =>
  passwordLoginSheetVisible.value
  || Boolean(miniWechatBindState.value)
  || notificationEmailSheetVisible.value,
)
const showGuestChantWall = computed(() => isGuestPage.value && !hasOpenSheet.value)
const showGuestLoginFloat = computed(() => isGuestPage.value && !hasOpenSheet.value)

const joinedAtLabel = computed(() => {
  if (!currentUser.value?.created_at) {
    return ''
  }

  const date = new Date(currentUser.value.created_at)
  return `${date.getFullYear()}-${date.getMonth() + 1}-${date.getDate()}`
})

const membershipExpiresAtLabel = computed(() =>
  formatMembershipExpiryLabel(currentUser.value?.membership_expires_at),
)
const notificationEmailLabel = computed(() =>
  formatNotificationEmailLabel(notificationEmail.value),
)

const currentMembershipMeta = computed(() => resolveUserMembershipMeta(currentUser.value?.membership_tier))
const canPurchaseMembership = computed(() =>
  canShowMembershipPurchaseEntry(
    Boolean(currentUser.value),
    Boolean(currentUser.value?.has_wechat_binding),
    currentUser.value?.membership_tier,
    currentUser.value?.membership_expires_at,
  ),
)
const canRenewCurrentV9Membership = computed(() =>
  currentMembershipMeta.value.code === 'V9'
  && Boolean(currentUser.value?.membership_expires_at?.trim()),
)
const membershipTierGuides = computed<MembershipTierGuide[]>(() =>
  buildMembershipTierGuides(publicConfig.value?.membership_tier_rules),
)
const membershipRuleSummary = computed(() => {
  const codes = membershipTierGuides.value.map((item) => item.code).join('、')

  if (!codes) {
    return '会员升级条件和对应刷新速度以后端当前配置为准。'
  }

  return `当前开放 ${codes}。现阶段会员权益只影响余票监控刷新频率，升级条件和对应刷新速度以后端当前配置为准。`
})
const currentMembershipGuide = computed<MembershipTierGuide>(() =>
  resolveMembershipTierGuide(
    currentMembershipMeta.value.code,
    publicConfig.value?.membership_tier_rules,
    currentUser.value?.ticket_watch_poll_interval_seconds,
  ),
)

const currentUserInviteCode = computed(() =>
  resolveCurrentUserInviteCode(currentUser.value?.invite_code),
)
const accountInfoItems = computed<UserAccountInfoItem[]>(() => {
  if (!currentUser.value) {
    return []
  }

  return buildUserAccountInfoItems({
    hasWechatBinding: currentUser.value.has_wechat_binding,
    joinedAtLabel: joinedAtLabel.value,
    membershipExpiresAtLabel: membershipExpiresAtLabel.value,
  })
})
const membershipBenefitItems = computed<UserBenefitItem[]>(() =>
  buildUserBenefitItems({
    membershipCode: currentMembershipMeta.value.code,
    refreshLabel: currentMembershipGuide.value.refreshLabel,
  }),
)
const upgradeSteps = computed<UserUpgradeStep[]>(() =>
  buildUserUpgradeSteps(membershipTierGuides.value, currentMembershipMeta.value.code),
)

function openSettingsPage() {
  uni.navigateTo({
    url: '/pages/user/settings/index',
    fail: (error) => {
      console.error('[user] navigate to settings failed', error)
      uni.showToast({ title: error?.errMsg || '打开设置失败', icon: 'none' })
    },
  })
}

async function loadUser(): Promise<void> {
  hasLocalAccessToken.value = Boolean(getAccessToken())
  loading.value = hasLocalAccessToken.value
  systemConfigUnderReview.value = await loadSystemConfigUnderReview()

  if (systemConfigUnderReview.value) {
    currentUser.value = null
    publicConfig.value = null
    miniWechatBindState.value = null
    loading.value = false
    return
  }

  if (!hasLocalAccessToken.value) {
    currentUser.value = null
    publicConfig.value = null
    notificationEmail.value = ''
    miniWechatBindState.value = null
    loading.value = false
    return
  }

  const [userResult, publicConfigResult, notificationEmailResult] = await Promise.allSettled([
    getCurrentUser(),
    getPublicSystemConfig(),
    getNotificationEmail(),
  ])

  if (userResult.status === 'fulfilled') {
    currentUser.value = userResult.value
  } else {
    currentUser.value = null
  }

  if (notificationEmailResult.status === 'fulfilled') {
    notificationEmail.value = notificationEmailResult.value.email?.trim() || ''
  } else {
    notificationEmail.value = ''
  }

  if (publicConfigResult.status === 'fulfilled') {
    publicConfig.value = publicConfigResult.value
  }

  loading.value = false
}

function openNotificationEmailSheet(): void {
  notificationEmailSheetVisible.value = true
}

async function saveNotificationEmail(email: string): Promise<void> {
  notificationEmailSaving.value = true
  try {
    const result = await updateNotificationEmail(email)
    notificationEmail.value = result.email?.trim() || email
    notificationEmailSheetVisible.value = false
    uni.showToast({ title: '邮箱已保存', icon: 'success' })
  } catch (error) {
    uni.showToast({ title: extractApiErrorMessage(error, '邮箱保存失败'), icon: 'none' })
  } finally {
    notificationEmailSaving.value = false
  }
}

async function handleLogout(): Promise<void> {
  await logout()
  uni.showToast({ title: '已退出登录', icon: 'success' })
  await loadUser()
}

function handleCopyInviteCode(): void {
  if (!currentUserInviteCode.value) {
    return
  }

  uni.setClipboardData({
    data: currentUserInviteCode.value,
    success: () => {
      uni.showToast({ title: '邀请码已复制', icon: 'success' })
    },
    fail: () => {
      uni.showToast({ title: '复制失败，请稍后重试', icon: 'none' })
    },
  })
}

function handleMembershipCardAction(): void {
  if (canPurchaseMembership.value) {
    uni.navigateTo({
      url: '/pages/membership-purchase/index',
    })
  }
}

function openAiFromBrandNav(): void {
  void openAiChat()
}

function handleLoginAction(): void {
  if (loginPresentation.method === 'password') {
    passwordLoginSheetVisible.value = true
    return
  }

  void handleMiniWechatLogin()
}

async function handlePasswordLogin(form: { accountIdentifier: string; password: string }): Promise<void> {
  passwordLoginSubmitting.value = true
  try {
    const result = await login(buildPasswordLoginPayload(form))
    hasLocalAccessToken.value = Boolean(getAccessToken())
    currentUser.value = result.user
    passwordLoginSheetVisible.value = false
    await loadUser()
    uni.showToast({ title: '登录成功', icon: 'success' })
    await redirectAfterLogin()
  } catch (error) {
    uni.showToast({ title: extractApiErrorMessage(error, '登录失败'), icon: 'none' })
  } finally {
    passwordLoginSubmitting.value = false
  }
}

async function handleMiniWechatLogin(): Promise<void> {
  try {
    const code = await getMiniWechatCode()
    const result = await loginWithMiniWechat(code)

    if (result.status === 'authenticated') {
      // 登录态标记必须立刻刷新，否则 isGuestPage 仍按未登录渲染（请先登录浮窗 + 歌词墙不消失）。
      hasLocalAccessToken.value = Boolean(getAccessToken())
      currentUser.value = result.user
      uni.showToast({ title: '微信登录成功', icon: 'success' })
      await loadUser()
      await redirectAfterLogin()
      return
    }

    miniWechatBindState.value = result
  } catch (error) {
    uni.showToast({ title: extractApiErrorMessage(error, '微信登录失败'), icon: 'none' })
  }
}

function closeMiniWechatBindSheet(): void {
  miniWechatBindState.value = null
}

async function handleMiniWechatBind(payload: {
  bindToken: string
  inviteCode: string
  referralCode: string
  displayName: string
  avatarDataUrl: string
}): Promise<void> {
  try {
    const result = await bindMiniWechatAccount({
      bind_token: payload.bindToken,
      invite_code: payload.inviteCode || undefined,
      referral_code: payload.referralCode || undefined,
      display_name: payload.displayName,
      avatar_data_url: payload.avatarDataUrl,
    })

    currentUser.value = result.user
    hasLocalAccessToken.value = Boolean(getAccessToken())
    closeMiniWechatBindSheet()
    uni.showToast({ title: '微信绑定成功', icon: 'success' })
    await loadUser()
    await redirectAfterLogin()
  } catch (error) {
    uni.showToast({ title: extractApiErrorMessage(error, '绑定失败'), icon: 'none' })
  }
}

async function redirectAfterLogin(): Promise<void> {
  const target = consumePostLoginRedirect()
  if (!target || target.url === '/pages/user/index') {
    return
  }

  try {
    await navigateToPostLoginTarget(target)
  } catch (error) {
    uni.showToast({ title: extractApiErrorMessage(error, '登录后跳转失败'), icon: 'none' })
  }
}

function getMiniWechatCode(): Promise<string> {
  return new Promise((resolve, reject) => {
    uni.login({
      success: (result) => {
        if (!result.code) {
          reject(new Error('微信登录 code 获取失败'))
          return
        }

        resolve(result.code)
      },
      fail: (error) => {
        reject(new Error(error.errMsg || '微信登录失败'))
      },
    })
  })
}

onShow(() => {
  reportPageActivity('user')
  void loadUser()
})
</script>

<style scoped lang="css">
.page-root {
  position: relative;
  min-height: 100vh;
  background: var(--fi-color-page-soft);
}

.page-root--guest {
  background: #11170f;
}

.page-scroll {
  padding-top: var(--fi-brand-nav-height);
  position: relative;
  z-index: 1;
  min-height: calc(100vh - var(--fi-brand-nav-height));
  box-sizing: border-box;
}

@keyframes fi-fade-in-up {
  from { opacity: 0; transform: translateY(30rpx); }
  to   { opacity: 1; transform: translateY(0); }
}

.page {
  --page-border: rgba(231, 234, 240, 0.95);
  --page-border-soft: rgba(238, 240, 245, 0.96);
  --page-border-warm: rgba(234, 226, 209, 0.92);
  position: relative;
  padding: 24rpx 16rpx 40rpx;
  display: flex;
  flex-direction: column;
  gap: 16rpx;
  min-height: calc(100vh - var(--fi-brand-nav-height));
  box-sizing: border-box;
}

.page-bg-img {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100vh;
  pointer-events: none;
  z-index: 0;
}

.page-bg-fade {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100vh;
  background:
    linear-gradient(180deg, rgba(246, 247, 244, 0.24) 0%, rgba(247, 248, 250, 0.74) 32%, var(--fi-color-page-soft) 58%, var(--fi-color-page-soft) 100%);
  pointer-events: none;
  z-index: 0;
}

.page-root--guest .page-bg-fade {
  background:
    linear-gradient(180deg, rgba(0, 0, 0, 0.18) 0%, rgba(247, 248, 250, 0.58) 22%, rgba(20, 20, 18, 0.12) 48%, rgba(0, 0, 0, 0.34) 100%),
    linear-gradient(0deg, rgba(0, 0, 0, 0.08), rgba(255, 255, 255, 0.04));
}

.page-root--guest .page {
  justify-content: flex-end;
  padding: 24rpx 28rpx calc(12rpx + env(safe-area-inset-bottom));
}

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

.account-actions {
  margin-top: 24rpx;
  border-radius: var(--fi-radius-lg);
  padding: 22rpx 24rpx;
  border: 1rpx solid rgba(229, 232, 238, 0.96);
  background: rgba(255, 255, 255, 0.94);
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 18rpx;
}

.account-actions__body {
  min-width: 0;
  flex: 1;
}

.account-actions__label {
  color: var(--fi-primitive-ink);
  font-size: var(--fi-font-26);
  font-weight: 700;
}

.account-actions__caption {
  display: block;
  margin-top: 8rpx;
  color: #7a808d;
  font-size: 23rpx;
  line-height: 1.55;
}

.settings-entry-panel {
  margin-top: 24rpx;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16rpx;
}

.settings-entry-panel--pressed {
  transform: scale(0.99);
  background: rgba(var(--fi-primitive-red-rgb), 0.04);
}

.settings-entry-panel__body {
  display: grid;
  gap: 6rpx;
  min-width: 0;
}

.settings-entry-panel__title {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-30);
  font-weight: 700;
}

.settings-entry-panel__caption {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}

.settings-entry-panel__arrow {
  color: var(--fi-color-text-muted);
  font-size: 40rpx;
  line-height: 1;
  font-weight: 700;
}

.logout-action {
  width: 100%;
  height: 78rpx;
  padding: 0 26rpx;
  border-radius: var(--fi-radius-round);
  border: 0;
  background: var(--fi-primitive-ink);
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-26);
  font-weight: 800;
  line-height: 78rpx;
  text-align: center;
  box-sizing: border-box;
  box-shadow: none;
}

.logout-action::after {
  border: none;
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

.hero-card__top--member {
  position: relative;
  z-index: 1;
  justify-content: flex-start;
}

.account-actions {
  margin-top: 0;
  border-radius: var(--fi-radius-lg);
  padding: 18rpx 20rpx;
  animation: fi-fade-in-up 520ms cubic-bezier(0.22, 1, 0.36, 1) 320ms both;
}
</style>
