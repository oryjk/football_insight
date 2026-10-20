<template>
  <page-meta :page-style="pageStyle" />
  <view class="page-root" :class="{ 'page-root--guest': isGuestPage }">
    <FiBrandNav :transparent="isGuestPage" />
    <image class="page-bg-img" :src="phoenixStadiumBgImage" mode="aspectFill" :webp="true" />
    <view class="page-bg-fade"></view>
    <view class="page-scroll">
      <view class="page">
      <UserBasicAccountPanel
        v-if="systemConfigUnderReview && currentUser"
        :user="currentUser"
        @logout="handleLogout"
      />

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
            :show-official-bind="canOpenOfficialBind"
            @edit-email="openNotificationEmailSheet"
            @bind-official="officialBind.open"
          />

          <UserSeasonRecapEntry @open="openSeasonRecap" />

          <view class="account-actions">
            <view class="account-actions__body">
              <text class="account-actions__label">账号管理</text>
              <text class="account-actions__caption">{{ loginPresentation.switchAccountCaption }}</text>
            </view>
            <button class="logout-action" @click="handleLogout">退出登录</button>
          </view>
        </template>
      </template>
      <!-- H5 测试登录入口：仅当后端配置 H5_TEST_LOGIN_USER_IDS 白名单时展示，不受审核模式影响。 -->
      <!-- #ifdef H5 -->
      <UserH5TestLoginPanel
        v-if="h5TestUsers.length"
        :users="h5TestUsers"
        :current-user-id="currentUser?.id"
        @select="handleH5TestLoginSelect"
      />
      <!-- #endif -->

      <!-- 设置入口仅管理员（后端白名单判定）可见；不受审核模式影响。 -->
      <view v-if="isReviewController" class="panel settings-entry-panel" hover-class="settings-entry-panel--pressed" hover-stay-time="100" @click="openSettingsPage">
        <view class="settings-entry-panel__body">
          <text class="settings-entry-panel__title">设置</text>
          <text class="settings-entry-panel__caption">查看当前版本审核状态</text>
        </view>
        <text class="settings-entry-panel__arrow">›</text>
      </view>
      </view>
    </view>

    <UserNotificationEmailSheet
      v-if="!systemConfigUnderReview"
      v-model:visible="notificationEmailSheetVisible"
      :email="notificationEmail"
      :saving="notificationEmailSaving"
      @save="saveNotificationEmail"
    />

    <UserOfficialBindSheet
      :visible="officialBindVisible"
      :code="officialBindCode"
      :remaining-seconds="officialBindRemainingSeconds"
      :loading="officialBindLoading"
      :error="officialBindError"
      @close="officialBind.close"
      @get-code="officialBind.getCode"
      @copy="officialBind.copyCode"
      @open-official="openUserOfficialAccount"
    />

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

    <UserGuestChantWall v-if="showGuestChantWall" />

    <FiLoginFloat
      v-if="showGuestLoginFloat"
      :action-text="loginPresentation.actionText"
      :desc="systemConfigUnderReview ? '登录后，可以关注主队并参与赛前助力。' : undefined"
      @action="handleLoginAction"
    />
  </view>
</template>

<script setup lang="ts">
import UserSeasonRecapEntry from './components/UserSeasonRecapEntry.vue'
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import { computed, onUnmounted, ref, watch } from 'vue'
import { onHide, onShow } from '@dcloudio/uni-app'
import FiBrandNav from '../../components/FiBrandNav.vue'
import FiLoginFloat from '../../components/FiLoginFloat.vue'
import UserGuestChantWall from './components/UserGuestChantWall.vue'
import UserBasicAccountPanel from './components/UserBasicAccountPanel.vue'
import UserSkeleton from './components/UserSkeleton.vue'
import UserAccountHero from './components/UserAccountHero.vue'
import UserAccountInfoPanel from './components/UserAccountInfoPanel.vue'
import UserPasswordLoginSheet from './components/UserPasswordLoginSheet.vue'
import UserWechatBindSheet from './components/UserWechatBindSheet.vue'
import UserNotificationEmailSheet from './components/UserNotificationEmailSheet.vue'
import UserOfficialBindSheet from '../../components/FiOfficialBindSheet.vue'
import UserH5TestLoginPanel from './components/UserH5TestLoginPanel.vue'
import badgeCheckIcon from '../../static/user/badge-check.svg'
import calendarDaysIcon from '../../static/user/calendar-days.svg'
import logInIcon from '../../static/user/log-in.svg'
import { PHOENIX_STADIUM_BG_IMAGE_URL as phoenixStadiumBgImage } from '../../config/assets'
import { listH5TestLoginUsers } from '../../api/auth'
import { useAppSession } from '../../stores/appSession'
import type { H5TestLoginUser, MiniWechatBindingRequiredResponse } from '../../types/auth'
import { useMiniReviewStatus } from '../../stores/miniReview'
import { useRuntimeConfig } from '../../stores/runtimeConfig'
import { reportPageActivity } from '../../utils/userActivity'
import { beginPaywallVisit, reportPaywallImpression, reportPaywallTap } from '../../composables/usePaywallTelemetry'
import { createUserAccountEffects } from './accountEffects'
import { createOfficialBindingGate } from '../../composables/officialBindingGate'
import { openUserOfficialAccount } from '../../utils/officialAccountNavigation'
import { createUserLoginFlow, type MiniWechatBindForm } from './loginFlow'
import {
  buildUserAccountInfoItems,
  canShowMembershipPurchaseEntry,
  formatNotificationEmailLabel,
  resolveCurrentUserInviteCode,
  resolveUserMembershipMeta,
  type UserAccountInfoItem,
} from './helpers'
import {
  resolveMembershipTierGuide,
  type MembershipTierGuide,
} from '../../utils/membershipRules'
import { resolveUserLoginPresentation } from './loginHelpers'

const { navigationPageStyle } = useNavigationLayout()

function openSeasonRecap() { uni.navigateTo({ url: '/pages/season-recap/index' }) }

const session = useAppSession()
// 当前用户与登录态共享自 appSession（唯一写入入口），本页只读。
const currentUser = session.currentUser
const hasAccessToken = session.hasAccessToken
// 设置入口仅对 MINI_REVIEW_CONTROL_USER_IDS 白名单用户（管理员）可见。
const isReviewController = ref(false)
const loading = ref(hasAccessToken.value)
// 公共配置共享自 runtimeConfig store（60 秒缓存，失败保留上次成功值）。
const runtimeConfig = useRuntimeConfig()
const publicConfig = runtimeConfig.config
const miniReview = useMiniReviewStatus()
// 生产构建查询期间保留会员展示限制；开发构建放开，登录和认证始终独立。
const systemConfigUnderReview = miniReview.shouldRestrictFeatures
const officialBind = createOfficialBindingGate(session, systemConfigUnderReview, computed(() => !loading.value))
const {
  visible: officialBindVisible,
  code: officialBindCode,
  remainingSeconds: officialBindRemainingSeconds,
  loading: officialBindLoading,
  error: officialBindError,
  canOpen: canOpenOfficialBind,
} = officialBind
const pageStyle = computed(() => `${navigationPageStyle.value};${officialBindVisible.value ? 'overflow: hidden;' : ''}`)
onHide(officialBind.handlePageHide)
onUnmounted(officialBind.dispose)
const notificationEmail = ref('')
const notificationEmailSheetVisible = ref(false)
const notificationEmailSaving = ref(false)

const miniWechatBindState = ref<MiniWechatBindingRequiredResponse | null>(null)
const passwordLoginSheetVisible = ref(false)
const passwordLoginSubmitting = ref(false)

// 账号读写（白名单/邮箱）收口到 effects：内部按 session revision 防护，
// 旧会话的迟到响应不写状态、不弹 toast。
const accountEffects = createUserAccountEffects(session, {
  loading,
  isReviewController,
  notificationEmail,
  notificationEmailSheetVisible,
  notificationEmailSaving,
  miniWechatBindState,
  passwordLoginSheetVisible,
  passwordLoginSubmitting,
})

// 登录/绑定/退出编排收口到 loginFlow：action 返回后、loadUser 完成后、toast/跳转前
// 都核对 revision，被更新操作替代的旧操作不提示成功、不跳转、不改新操作状态。
const loginFlow = createUserLoginFlow(
  session,
  {
    passwordLoginSheetVisible,
    passwordLoginSubmitting,
    miniWechatBindState,
  },
  () => accountEffects.loadUser(),
)

// 会话被替代（新登录/退出/失效）时，丢弃按旧会话展示的私有状态；
// 但本页自己发起的认证提交（beginAuthIntent 同步递增 revision）只清展示数据，
// 保留正在填写的认证表单（密码弹层/绑定 token），失败后可原地修改重试。
watch(session.revision, (newRevision) => {
  accountEffects.handleSessionRevisionChange(loginFlow.isOwnAuthSubmit(newRevision))
})

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
  'log-in': logInIcon,
}

const isGuestPage = computed(() => !hasAccessToken.value || (!loading.value && !currentUser.value))
const hasOpenSheet = computed(() =>
  passwordLoginSheetVisible.value
  || Boolean(miniWechatBindState.value)
  || notificationEmailSheetVisible.value
  || officialBindVisible.value,
)
// H5 测试登录面板展开时，隐藏游客歌词墙和登录浮窗，避免遮挡测试账号列表。
const h5TestUsers = ref<H5TestLoginUser[]>([])
const h5TestLoginPanelVisible = computed(() => isH5 && h5TestUsers.value.length > 0)
const showGuestChantWall = computed(() => isGuestPage.value && !hasOpenSheet.value && !h5TestLoginPanelVisible.value)
const showGuestLoginFloat = computed(() => isGuestPage.value && !hasOpenSheet.value && !h5TestLoginPanelVisible.value)

const joinedAtLabel = computed(() => {
  if (!currentUser.value?.created_at) {
    return ''
  }

  const date = new Date(currentUser.value.created_at)
  return `${date.getFullYear()}-${date.getMonth() + 1}-${date.getDate()}`
})

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
    officialWechatBound: currentUser.value.official_wechat_bound,
    joinedAtLabel: joinedAtLabel.value,
  })
})
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
  await accountEffects.loadUser()
}

function openNotificationEmailSheet(): void {
  notificationEmailSheetVisible.value = true
}

async function saveNotificationEmail(email: string): Promise<void> {
  await accountEffects.saveNotificationEmail(email)
}

async function handleLogout(): Promise<void> {
  await loginFlow.logout()
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
    reportPaywallTap('user_membership_entry')
    uni.navigateTo({
      url: '/pages/membership-purchase/index',
    })
  }
}

function reportMembershipEntryImpression(): void {
  if (canPurchaseMembership.value && !systemConfigUnderReview.value) reportPaywallImpression('user_membership_entry')
}

watch(canPurchaseMembership, reportMembershipEntryImpression)

function handleLoginAction(): void {
  if (loginPresentation.method === 'password') {
    passwordLoginSheetVisible.value = true
    return
  }

  void loginFlow.loginWithMiniWechat()
}

async function handlePasswordLogin(form: { accountIdentifier: string; password: string }): Promise<void> {
  await loginFlow.submitPasswordLogin(form)
}

async function handleMiniWechatLogin(): Promise<void> {
  await loginFlow.loginWithMiniWechat()
}

function closeMiniWechatBindSheet(): void {
  miniWechatBindState.value = null
}

async function handleMiniWechatBind(payload: MiniWechatBindForm): Promise<void> {
  await loginFlow.bindMiniWechatAccount(payload)
}

async function loadH5TestUsers(): Promise<void> {
  if (!isH5) {
    return
  }

  // 后端未配置 H5_TEST_LOGIN_USER_IDS 白名单时接口返回 403，此时直接隐藏入口。
  try {
    const result = await listH5TestLoginUsers()
    h5TestUsers.value = result.items
  } catch {
    h5TestUsers.value = []
  }
}

async function handleH5TestLoginSelect(user: H5TestLoginUser): Promise<void> {
  await loginFlow.loginAsH5TestUser(user)
}

onShow(() => {
  void officialBind.handlePageShow()
  reportPageActivity('user')
  beginPaywallVisit()
  reportMembershipEntryImpression()
  void miniReview.ensureMiniReviewReady()
  void loadUser()
  void loadH5TestUsers()
  void runtimeConfig.ensureRuntimeConfig()
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
  padding: var(--fi-space-24) var(--fi-space-16) var(--fi-space-40);
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-card-gap);
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
  padding: var(--fi-space-24) var(--fi-space-28) calc(var(--fi-space-32) + env(safe-area-inset-bottom) + var(--fi-space-100));
}

.hero-card, .panel {
  position: relative;
  z-index: 1;
  background: rgba(255,255,255,0.72);
  border-radius: var(--fi-radius-xl);
  padding: var(--fi-space-20);
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

.hero-card__top, .section-heading { align-items: flex-start; gap: var(--fi-space-12); }

.hero-card__title, .section-title {
  display: block;
  margin-top: var(--fi-space-10);
  color: #2a2c31;
  font-size: var(--fi-font-48);
  line-height: 1.08;
  font-weight: 800;
}

.hero-card__summary, .account-form-panel__summary, .user-panel__summary {
  display: block;
  margin-top: var(--fi-space-18);
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
  padding: var(--fi-space-14) var(--fi-space-24);
  border-radius: var(--fi-radius-round);
  border: 2rpx solid var(--page-border);
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
}

.account-actions {
  border-radius: var(--fi-radius-lg);
  padding: var(--fi-space-22) var(--fi-space-24);
  border: 1rpx solid rgba(229, 232, 238, 0.96);
  background: rgba(255, 255, 255, 0.94);
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: var(--fi-space-18);
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
  margin-top: var(--fi-space-8);
  color: #7a808d;
  font-size: var(--fi-font-23);
  line-height: 1.55;
}

.settings-entry-panel {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-16);
}

.settings-entry-panel--pressed {
  transform: scale(0.99);
  background: rgba(var(--fi-primitive-red-rgb), 0.04);
}

.settings-entry-panel__body {
  display: grid;
  gap: var(--fi-space-6);
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
  font-size: var(--fi-font-40);
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
  border-radius: var(--fi-radius-lg);
  padding: var(--fi-space-18) var(--fi-space-20);
  animation: fi-fade-in-up 520ms cubic-bezier(0.22, 1, 0.36, 1) 320ms both;
}
</style>
