<template>
  <view class="page-root">
    <image class="page-bg-img" :src="bgImage" mode="aspectFill" />
    <view class="page-bg-fade"></view>
    <scroll-view scroll-y class="page-scroll">
      <view class="page">
      <template v-if="systemConfigUnderReview">
        <view class="hero-card account-hero">
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
      <view
        class="hero-card account-hero"
        :class="currentUser ? [currentMembershipMeta.heroClass, currentMembershipGuide.toneClass] : ''"
      >
        <view class="hero-card__top" :class="{ 'hero-card__top--member': currentUser }">
          <template v-if="currentUser">
            <view class="account-hero__eyebrow">
              <image class="account-hero__eyebrow-icon" :src="diamondIcon" mode="aspectFit" />
              <text class="account-hero__eyebrow-text">会员中心</text>
            </view>
          </template>
          <template v-else>
            <view>
              <text class="eyebrow">会员中心</text>
              <text class="hero-card__title">{{ accountHeadline }}</text>
            </view>
            <text v-if="accountHeroBadge" class="meta-note meta-note--hero">{{ accountHeroBadge }}</text>
          </template>
        </view>

        <view v-if="currentUser" class="profile-banner profile-banner--hero">
          <view class="profile-banner__main">
            <view class="profile-banner__avatar-wrap">
              <view class="profile-banner__avatar">
                <image
                  v-if="currentUser.avatar_url"
                  :src="currentUser.avatar_url"
                  mode="aspectFill"
                  class="profile-banner__avatar-image"
                />
                <text v-else class="profile-banner__avatar-fallback">{{ avatarFallbackLabel }}</text>
              </view>
              <view class="profile-banner__avatar-badge" :class="currentMembershipGuide.toneClass">{{ currentMembershipMeta.code }}</view>
            </view>

            <view class="profile-banner__body">
              <text class="profile-banner__name">{{ currentUser.display_name }}</text>
              <text class="membership-badge" :class="currentMembershipMeta.badgeClass">
                {{ currentMembershipMeta.badgeLabel }}
              </text>
            </view>
          </view>

          <view class="account-hero__medal" :class="currentMembershipGuide.toneClass">
            <view class="account-hero__medal-ring"></view>
            <view class="account-hero__medal-orbit"></view>
            <view class="account-hero__medal-star account-hero__medal-star--one"></view>
            <view class="account-hero__medal-star account-hero__medal-star--two"></view>
            <view class="account-hero__medal-shield">
              <text class="account-hero__medal-crown">⌃</text>
              <text class="account-hero__medal-code">{{ currentMembershipMeta.code }}</text>
            </view>
          </view>
        </view>

        <view v-if="currentUserInviteCode" class="profile-banner__invite profile-banner__invite--hero">
          <view class="profile-banner__invite-body">
            <text class="profile-banner__invite-label">我的邀请码</text>
            <text class="profile-banner__invite-code">{{ currentUserInviteCode }}</text>
          </view>
          <button class="profile-banner__invite-copy" :class="currentMembershipGuide.toneClass" @click="handleCopyInviteCode">一键复制</button>
        </view>

        <text v-else class="hero-card__summary">{{ accountSummary }}</text>

        <view v-if="canPurchaseMembership" class="purchase-entry" @click="navigateToPurchase">
          <view class="purchase-entry__body">
            <view class="purchase-entry__eyebrow">
              <image class="purchase-entry__eyebrow-icon" :src="diamondIcon" mode="aspectFit" />
              <text>会员升级通道</text>
            </view>
            <text class="purchase-entry__label">V6-V9 会员充值</text>
            <text class="purchase-entry__hint">V6 起解锁最近回流速览，购买后立即生效。</text>
          </view>
          <view class="purchase-entry__aside">
            <text class="purchase-entry__cta">立即开通</text>
            <text class="purchase-entry__arrow">›</text>
          </view>
        </view>
      </view>

      <FiLoading
        v-if="loading"
        title="账号状态加载中"
        caption="正在确认你当前的小程序登录状态。"
      />

      <template v-else-if="currentUser">
        <view class="panel info-panel">
          <view class="info-panel__header">
            <text class="info-panel__title">账户信息</text>
          </view>

          <view class="account-info-grid">
            <view
              v-for="item in accountInfoItems"
              :key="item.key"
              class="account-info-item"
            >
              <view class="account-info-item__icon">{{ item.iconLabel }}</view>
              <view class="account-info-item__body">
                <text class="account-info-item__label">{{ item.label }}</text>
                <text class="account-info-item__value">{{ item.value }}</text>
              </view>
            </view>
          </view>
        </view>

        <view class="panel privilege-panel" :class="currentMembershipGuide.toneClass">
          <view class="privilege-panel__header">
            <view class="privilege-panel__title-row">
              <text class="privilege-panel__crown">⌃</text>
              <text class="privilege-panel__title">{{ currentMembershipMeta.code }} 专属权益</text>
            </view>
            <text class="privilege-panel__hint">尊享更多特权</text>
          </view>

          <view class="privilege-grid">
            <view
              v-for="item in membershipBenefitItems"
              :key="item.key"
              class="privilege-card"
            >
              <view class="privilege-card__icon">{{ item.iconLabel }}</view>
              <text class="privilege-card__title">{{ item.title }}</text>
              <text class="privilege-card__caption">{{ item.caption }}</text>
            </view>
          </view>
        </view>

        <view class="panel upgrade-panel">
          <view class="upgrade-panel__title-row">
            <view class="upgrade-panel__info-icon">i</view>
            <text class="upgrade-panel__title">升级说明</text>
          </view>
          <text class="upgrade-panel__body">
            {{ membershipRuleSummary }}
          </text>
          <view v-if="upgradeSteps.length" class="upgrade-step-list">
            <view
              v-for="step in upgradeSteps"
              :key="step.key"
              class="tier-ladder-card tier-ladder-card--compact"
              :class="[step.toneClass, { 'tier-ladder-card--active': step.isCurrent }]"
            >
              <view class="tier-ladder-card__top">
                <view class="tier-ladder-card__heading">
                  <view class="upgrade-step-card__title-row">
                    <text class="tier-ladder-card__code">{{ step.code }}</text>
                    <text class="tier-ladder-card__name">{{ step.name }}</text>
                  </view>
                  <text class="tier-ladder-card__condition">{{ step.condition }}</text>
                </view>

                <view class="upgrade-step-card__aside">
                  <text class="tier-ladder-card__refresh">{{ step.refreshLabel }}</text>
                  <text v-if="step.isCurrent" class="upgrade-step-card__current">当前等级</text>
                </view>
              </view>
            </view>
          </view>
        </view>

        <view class="account-actions">
          <view class="account-actions__body">
            <text class="account-actions__label">账号管理</text>
            <text class="account-actions__caption">如需切换账号，可退出后重新使用微信登录。</text>
          </view>
          <button class="logout-action" @click="handleLogout">退出登录</button>
        </view>
      </template>

      <view v-else class="panel account-form-panel">
        <view class="section-heading section-heading--compact">
          <view>
            <text class="section-kicker">登录方式</text>
            <text class="section-title">微信登录后查看会员身份</text>
          </view>
          <text class="meta-note">V1 / V3</text>
        </view>

        <text class="account-form-panel__summary">
          邀请码会员默认为 V3，首次绑定时可选填推荐码，推荐成功后会继续升级到 V4-V9。
        </text>

        <view class="tier-preview-grid">
          <view class="tier-preview-card tier-preview-card--v3">
            <text class="tier-preview-card__eyebrow">邀请码会员</text>
            <text class="tier-preview-card__title">V3 身份徽章</text>
            <text class="tier-preview-card__body">通过邀请码完成首次绑定。</text>
          </view>
          <view class="tier-preview-card tier-preview-card--v1">
            <text class="tier-preview-card__eyebrow">标准会员</text>
            <text class="tier-preview-card__title">V1 基础身份</text>
            <text class="tier-preview-card__body">非邀请码链路默认进入 V1。</text>
          </view>
        </view>

        <view v-if="isH5" class="mini-wechat-entry mini-wechat-entry--disabled">
          <view class="mini-wechat-entry__notice">
            <text class="mini-wechat-entry__tag">H5 暂不支持登录</text>
            <text class="mini-wechat-entry__title">请在微信小程序中完成登录</text>
            <text class="mini-wechat-entry__desc">
              当前网页端仅支持浏览基础内容，会员身份、升级和邀请码相关操作请前往微信小程序处理。
            </text>
          </view>
        </view>
        <view v-else class="mini-wechat-entry">
          <button class="primary-action mini-wechat-entry__button" @click="handleMiniWechatLogin">微信一键登录</button>
          <text class="form-footnote">首次登录时补充邀请码、头像和昵称即可。</text>
        </view>
      </view>

      <view v-if="miniWechatBindState" class="sheet-mask" @tap="handleSheetMaskTap">
        <view class="sheet-card" @tap.stop="consumeSheetTap">
          <view class="section-heading section-heading--compact">
            <view>
              <text class="section-kicker">首次微信登录</text>
              <text class="section-title">补充邀请码、头像和昵称</text>
            </view>
            <text class="meta-note">完成后直接登录</text>
          </view>

          <text class="account-form-panel__summary">
            完成一次绑定后，后续就能直接用微信进入。推荐码选填，填写推荐人的邀请码即可。
          </text>

          <view class="mini-wechat-bind-form">
            <button
              class="avatar-picker"
              open-type="chooseAvatar"
              @tap.stop="consumeSheetTap"
              @chooseavatar="handleChooseAvatar"
            >
              <image
                v-if="miniWechatBindForm.avatarPreviewUrl"
                :src="miniWechatBindForm.avatarPreviewUrl"
                mode="aspectFill"
                class="avatar-picker__image"
              />
              <text v-else class="avatar-picker__placeholder">选择头像</text>
            </button>

            <input
              v-model="miniWechatBindForm.displayName"
              type="nickname"
              class="auth-input"
              placeholder="请输入昵称"
            />
            <input
              v-model="miniWechatBindForm.inviteCode"
              class="auth-input"
              placeholder="请输入邀请码"
            />
            <input
              v-model="miniWechatBindForm.referralCode"
              class="auth-input"
              placeholder="请输入推荐码（可选，填推荐人的邀请码）"
            />
          </view>

          <view class="sheet-actions">
            <button class="primary-action primary-action--ghost" @click="closeMiniWechatBindSheet">取消</button>
            <button class="primary-action" @click="handleMiniWechatBind">完成绑定</button>
          </view>
        </view>
      </view>
      </template>
    </view>
  </scroll-view>
  </view>
</template>

<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import { onShow } from '@dcloudio/uni-app'
import FiLoading from '../../components/FiLoading.vue'
import diamondIcon from '../../static/user/diamond.svg'
import bgImage from '../../static/user/bg.webp'
import {
  bindMiniWechatAccount,
  getCurrentUser,
  loginWithMiniWechat,
  logout,
} from '../../api/auth'
import { getPublicSystemConfig } from '../../api/system'
import type { CurrentUser, MiniWechatBindingRequiredResponse } from '../../types/auth'
import type { PublicSystemConfig } from '../../types/system'
import { extractApiErrorMessage } from '../../utils/apiError'
import { loadSystemConfigUnderReview } from '../../utils/systemConfig'
import { reportPageActivity } from '../../utils/userActivity'
import {
  buildUserAccountInfoItems,
  buildUserBenefitItems,
  buildUserUpgradeSteps,
  formatMembershipExpiryLabel,
  resolveCurrentUserInviteCode,
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

const loading = ref(true)
const currentUser = ref<CurrentUser | null>(null)
const publicConfig = ref<PublicSystemConfig | null>(null)
const systemConfigUnderReview = ref(false)

const miniWechatBindState = ref<MiniWechatBindingRequiredResponse | null>(null)
const miniWechatBindForm = reactive({
  inviteCode: '',
  referralCode: '',
  displayName: '',
  avatarPreviewUrl: '',
})

const isH5 =
  // #ifdef H5
  true
  // #endif
  // #ifndef H5
  false
  // #endif

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

interface MembershipMeta {
  code: string
  badgeLabel: string
  heroClass: string
  badgeClass: string
  surfaceClass: string
  levelHint: string
  levelDescription: string
}

function resolveMembershipMeta(tier: string | undefined): MembershipMeta {
  const normalizedTier = tier?.trim() || 'V1'
  const tierNumber = Number.parseInt(normalizedTier.replace(/^V/i, ''), 10)

  if (normalizedTier === 'V3') {
    return {
      code: 'V3',
      badgeLabel: 'V3 邀请会员',
      heroClass: 'account-hero--v3',
      badgeClass: 'membership-badge--v3',
      surfaceClass: 'member-card--v3',
      levelHint: '邀请码会员',
      levelDescription: '通过邀请码完成首次绑定，当前按 V3 等级展示。',
    }
  }

  if (normalizedTier === 'V2') {
    return {
      code: 'V2',
      badgeLabel: 'V2 进阶会员',
      heroClass: 'account-hero--v2',
      badgeClass: 'membership-badge--v2',
      surfaceClass: 'member-card--v2',
      levelHint: '进阶会员',
      levelDescription: '通过推荐好友注册已升级到 V2，回流频率按进阶档位开放。',
    }
  }

  if (Number.isFinite(tierNumber) && tierNumber >= 4) {
    return {
      code: normalizedTier,
      badgeLabel: `${normalizedTier} 推荐会员`,
      heroClass: 'account-hero--v3',
      badgeClass: 'membership-badge--v3',
      surfaceClass: 'member-card--v3',
      levelHint: '推荐升级会员',
      levelDescription: `当前已经通过推荐升级到 ${normalizedTier}，回流频率和后续会员权益会按更高档位开放。`,
    }
  }

  return {
    code: normalizedTier,
    badgeLabel: `${normalizedTier} 标准会员`,
    heroClass: 'account-hero--v1',
    badgeClass: 'membership-badge--v1',
    surfaceClass: 'member-card--v1',
    levelHint: '标准会员',
    levelDescription: '当前是 V1 基础会员身份。',
  }
}

const currentMembershipMeta = computed(() => resolveMembershipMeta(currentUser.value?.membership_tier))
const canPurchaseMembership = computed(() =>
  Boolean(
    currentUser.value
    && currentUser.value.has_wechat_binding
    && currentUser.value.membership_tier !== 'V9',
  ),
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

const avatarFallbackLabel = computed(() => {
  const name = currentUser.value?.display_name?.trim() ?? ''
  if (!name) {
    return '会'
  }

  return name.slice(0, 1).toUpperCase()
})

const currentUserInviteCode = computed(() =>
  resolveCurrentUserInviteCode(currentUser.value?.invite_code),
)
const accountInfoItems = computed<UserAccountInfoItem[]>(() => {
  if (!currentUser.value) {
    return []
  }

  return buildUserAccountInfoItems({
    membershipCode: currentMembershipMeta.value.code,
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

const accountHeadline = computed(() => {
  if (!currentUser.value) {
    return '登录后查看你的会员身份'
  }

  return '当前会员身份'
})

const accountHeroBadge = computed(() => {
  if (!currentUser.value) {
    return isH5 ? '小程序登录' : '微信登录'
  }

  return ''
})

const accountSummary = computed(() => {
  if (!currentUser.value) {
    return isH5
      ? 'H5 端暂不支持登录，请前往微信小程序查看会员身份。'
      : '登录后会自动生成会员身份和等级徽章。'
  }

  return currentUser.value.has_wechat_binding
    ? `${currentMembershipMeta.value.code} 会员`
    : '会员身份待完善'
})

async function loadUser(): Promise<void> {
  loading.value = true
  systemConfigUnderReview.value = await loadSystemConfigUnderReview()

  if (systemConfigUnderReview.value) {
    currentUser.value = null
    publicConfig.value = null
    miniWechatBindState.value = null
    loading.value = false
    return
  }

  const [userResult, publicConfigResult] = await Promise.allSettled([
    getCurrentUser(),
    getPublicSystemConfig(),
  ])

  if (userResult.status === 'fulfilled') {
    currentUser.value = userResult.value
  } else {
    currentUser.value = null
  }

  if (publicConfigResult.status === 'fulfilled') {
    publicConfig.value = publicConfigResult.value
  }

  loading.value = false
}

async function handleLogout(): Promise<void> {
  await logout()
  currentUser.value = null
  uni.showToast({ title: '已退出登录', icon: 'success' })
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

function navigateToPurchase(): void {
  uni.navigateTo({
    url: '/pages/membership-purchase/index',
  })
}

async function handleMiniWechatLogin(): Promise<void> {
  try {
    const code = await getMiniWechatCode()
    const result = await loginWithMiniWechat(code)

    if (result.status === 'authenticated') {
      currentUser.value = result.user
      uni.showToast({ title: '微信登录成功', icon: 'success' })
      await redirectAfterLogin()
      return
    }

    miniWechatBindState.value = result
    miniWechatBindForm.inviteCode = ''
    miniWechatBindForm.referralCode = ''
    miniWechatBindForm.displayName = normalizeTextValue(result.display_name)
    miniWechatBindForm.avatarPreviewUrl = normalizeAvatarPreviewUrl(result.avatar_url)
  } catch (error) {
    uni.showToast({ title: extractApiErrorMessage(error, '微信登录失败'), icon: 'none' })
  }
}

function normalizeTextValue(value: unknown): string {
  return typeof value === 'string' ? value.trim() : ''
}

function normalizeAvatarPreviewUrl(value: unknown): string {
  const normalizedValue = normalizeTextValue(value)
  return normalizedValue
}

function consumeSheetTap(): void {}

function handleSheetMaskTap(): void {
  closeMiniWechatBindSheet()
}

function handleChooseAvatar(event: { detail?: { avatarUrl?: unknown } }): void {
  const avatarUrl = normalizeAvatarPreviewUrl(event.detail?.avatarUrl)
  if (!avatarUrl) {
    return
  }

  miniWechatBindForm.avatarPreviewUrl = avatarUrl
}

function closeMiniWechatBindSheet(): void {
  miniWechatBindState.value = null
  miniWechatBindForm.inviteCode = ''
  miniWechatBindForm.referralCode = ''
  miniWechatBindForm.displayName = ''
  miniWechatBindForm.avatarPreviewUrl = ''
}

async function handleMiniWechatBind(): Promise<void> {
  if (!miniWechatBindState.value) {
    return
  }

  try {
    const avatarDataUrl = await readAvatarAsDataUrl(miniWechatBindForm.avatarPreviewUrl)
    const result = await bindMiniWechatAccount({
      bind_token: miniWechatBindState.value.bind_token,
      invite_code: miniWechatBindForm.inviteCode.trim(),
      referral_code: miniWechatBindForm.referralCode.trim() || undefined,
      display_name: miniWechatBindForm.displayName.trim(),
      avatar_data_url: avatarDataUrl,
    })

    currentUser.value = result.user
    closeMiniWechatBindSheet()
    uni.showToast({ title: '微信绑定成功', icon: 'success' })
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

function inferMimeType(filePath: string): string {
  const lowerPath = filePath.toLowerCase()
  if (lowerPath.endsWith('.jpg') || lowerPath.endsWith('.jpeg')) {
    return 'image/jpeg'
  }
  if (lowerPath.endsWith('.webp')) {
    return 'image/webp'
  }
  return 'image/png'
}

function readAvatarAsDataUrl(filePath: string): Promise<string> {
  const normalizedPath = filePath.trim()
  if (!normalizedPath) {
    return Promise.reject(new Error('请先选择头像'))
  }

  if (normalizedPath.startsWith('data:image/')) {
    return Promise.resolve(normalizedPath)
  }

  return new Promise((resolve, reject) => {
    uni.getFileSystemManager().readFile({
      filePath: normalizedPath,
      encoding: 'base64',
      success: (result) => {
        const base64 = typeof result.data === 'string' ? result.data : ''
        if (!base64) {
          reject(new Error('头像读取失败'))
          return
        }

        resolve(`data:${inferMimeType(normalizedPath)};base64,${base64}`)
      },
      fail: (error) => {
        reject(new Error(error.errMsg || '头像读取失败'))
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
.page-root { position: relative; }
.page-scroll { height: 100vh; position: relative; z-index: 1; }

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
}
.page-bg-img {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 600rpx;
  pointer-events: none;
  z-index: 0;
}
.page-bg-fade {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 600rpx;
  background: linear-gradient(180deg, transparent 45%, rgba(247,248,250,0.55) 78%, #f7f8fa 100%);
  pointer-events: none;
  z-index: 0;
}
.hero-card, .panel {
  position: relative;
  z-index: 1;
  background: rgba(255,255,255,0.72);
  border-radius: 36rpx;
  padding: 20rpx;
  border: 2rpx solid rgba(255,255,255,0.55);
  box-shadow: 0 20rpx 48rpx rgba(26,28,36,0.06);
  backdrop-filter: blur(18rpx);
  -webkit-backdrop-filter: blur(18rpx);
}
.account-hero {
  background: linear-gradient(180deg, rgba(255,255,255,0.78), rgba(255,255,255,0.55));
  border-width: 1rpx;
  border-color: rgba(255,255,255,0.42);
  backdrop-filter: blur(24rpx);
  -webkit-backdrop-filter: blur(24rpx);
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
.hero-card__top, .section-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.hero-card__top, .section-heading { align-items: flex-start; gap: 12rpx; }
.eyebrow, .section-kicker {
  margin: 0;
  color: #8f9198;
  font-size: 22rpx;
  font-weight: 700;
  letter-spacing: 3rpx;
}
.hero-card__title, .section-title {
  display: block;
  margin-top: 10rpx;
  color: #2a2c31;
  font-size: 48rpx;
  line-height: 1.08;
  font-weight: 800;
}
.section-title { font-size: 44rpx; }
.hero-card__summary, .account-form-panel__summary, .user-panel__summary {
  display: block;
  margin-top: 18rpx;
  color: #6b707b;
  font-size: 28rpx;
  line-height: 1.7;
}
.profile-banner {
  margin-top: 24rpx;
  display: flex;
  align-items: center;
  gap: 22rpx;
}
.profile-banner__avatar {
  width: 128rpx;
  height: 128rpx;
  flex-shrink: 0;
  border-radius: 999rpx;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(180deg, rgba(27, 30, 38, 0.95), rgba(69, 77, 94, 0.9));
  box-shadow: 0 16rpx 36rpx rgba(18, 22, 30, 0.12);
}
.profile-banner__avatar-image {
  width: 100%;
  height: 100%;
}
.profile-banner__avatar-fallback {
  color: #ffffff;
  font-size: 44rpx;
  font-weight: 800;
}
.profile-banner__body {
  min-width: 0;
  flex: 1;
}
.profile-banner__identity {
  display: flex;
  align-items: center;
  gap: 12rpx;
  flex-wrap: wrap;
}
.profile-banner__name {
  color: #121212;
  font-size: 42rpx;
  font-weight: 800;
  line-height: 1.15;
}
.profile-banner__invite {
  margin-top: 16rpx;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16rpx;
  padding: 16rpx 18rpx;
  border-radius: 24rpx;
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
  color: #15161b;
  font-size: 28rpx;
  font-weight: 800;
  line-height: 1.2;
  word-break: break-all;
}
.profile-banner__invite-copy {
  flex-shrink: 0;
  padding: 0 22rpx;
  height: 64rpx;
  line-height: 64rpx;
  border-radius: 999rpx;
  border: 2rpx solid rgba(229, 210, 170, 0.96);
  background: linear-gradient(180deg, rgba(255, 247, 226, 0.98), rgba(245, 229, 191, 0.98));
  color: #7c5a1e;
  font-size: 24rpx;
  font-weight: 700;
  box-sizing: border-box;
}
.profile-banner__meta,
.profile-banner__summary {
  display: block;
  margin-top: 10rpx;
  color: #737987;
  font-size: 24rpx;
  line-height: 1.65;
}
.membership-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 12rpx 18rpx;
  border-radius: 999rpx;
  font-size: 22rpx;
  font-weight: 700;
  line-height: 1;
  white-space: nowrap;
}
.membership-badge--v3 {
  color: #7a5318;
  background: linear-gradient(135deg, rgba(255, 237, 199, 0.98), rgba(238, 212, 154, 0.98));
  border: 2rpx solid rgba(225, 189, 120, 0.95);
}
.membership-badge--v2 {
  color: #3a6b5c;
  background: linear-gradient(135deg, rgba(210, 238, 228, 0.98), rgba(185, 220, 205, 0.98));
  border: 2rpx solid rgba(142, 190, 174, 0.95);
}
.membership-badge--v1 {
  color: #556176;
  background: linear-gradient(135deg, rgba(241, 245, 252, 0.98), rgba(225, 232, 244, 0.98));
  border: 2rpx solid rgba(195, 205, 223, 0.95);
}
.tier-emblem {
  --tier-accent: #9aa6bb;
  --tier-accent-soft: rgba(217, 225, 240, 0.95);
  --tier-ink: #445065;
  position: relative;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8rpx;
}
.tier-emblem--compact {
  transform: translateY(4rpx);
}
.tier-emblem--featured {
  width: 156rpx;
}
.tier-emblem--list {
  width: 98rpx;
}
.tier-emblem__crest {
  width: 92rpx;
  height: 92rpx;
  padding: 10rpx;
  border-radius: 28rpx;
  box-sizing: border-box;
  background:
    linear-gradient(135deg, rgba(255,255,255,0.94), rgba(255,255,255,0.22)),
    linear-gradient(180deg, var(--tier-accent-soft), rgba(255,255,255,0.82));
  border: 2rpx solid rgba(255,255,255,0.78);
  box-shadow:
    0 16rpx 36rpx var(--tier-shadow),
    inset 0 -8rpx 14rpx rgba(255,255,255,0.48);
  position: relative;
  overflow: hidden;
}
.tier-emblem__crest::before,
.tier-emblem__crest::after {
  content: '';
  position: absolute;
  top: 24rpx;
  width: 22rpx;
  height: 16rpx;
  border-radius: 999rpx 999rpx 12rpx 12rpx;
  background: var(--tier-accent);
  opacity: 0.92;
}
.tier-emblem__crest::before {
  left: -4rpx;
  transform: rotate(-26deg);
}
.tier-emblem__crest::after {
  right: -4rpx;
  transform: rotate(26deg);
}
.tier-emblem__core {
  width: 100%;
  height: 100%;
  border-radius: 22rpx;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--tier-ink);
  font-size: 26rpx;
  font-weight: 900;
  letter-spacing: 1rpx;
  background:
    radial-gradient(circle at 50% 25%, rgba(255,255,255,0.86), transparent 42%),
    linear-gradient(180deg, rgba(255,255,255,0.94), var(--tier-accent-soft));
  border: 2rpx solid rgba(255,255,255,0.76);
  box-shadow: inset 0 -8rpx 12rpx rgba(255,255,255,0.34);
}
.tier-emblem--featured .tier-emblem__crest {
  width: 124rpx;
  height: 124rpx;
  border-radius: 34rpx;
}
.tier-emblem--featured .tier-emblem__core {
  border-radius: 26rpx;
  font-size: 34rpx;
}
.tier-emblem__pips {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8rpx;
  min-height: 14rpx;
}
.tier-emblem__pip {
  width: 10rpx;
  height: 10rpx;
  border-radius: 999rpx;
  background: var(--tier-accent);
  box-shadow: 0 4rpx 10rpx var(--tier-shadow);
}
.tier-tone--v1 {
  --tier-accent: #7f90aa;
  --tier-accent-soft: rgba(220, 229, 244, 0.98);
  --tier-ink: #4a586d;
  --tier-shadow: rgba(127, 144, 170, 0.28);
}
.tier-tone--v2 {
  --tier-accent: #6a9e8e;
  --tier-accent-soft: rgba(210, 238, 228, 0.98);
  --tier-ink: #3a6b5c;
  --tier-shadow: rgba(106, 158, 142, 0.28);
}
.tier-tone--v3 {
  --tier-accent: #c99642;
  --tier-accent-soft: rgba(244, 223, 176, 0.98);
  --tier-ink: #7a5318;
  --tier-shadow: rgba(201, 150, 66, 0.28);
}
.tier-tone--v4 {
  --tier-accent: #cf694e;
  --tier-accent-soft: rgba(248, 212, 198, 0.98);
  --tier-ink: #8a3c2c;
  --tier-shadow: rgba(207, 105, 78, 0.28);
}
.tier-tone--v5 {
  --tier-accent: #8e5bc7;
  --tier-accent-soft: rgba(227, 214, 248, 0.98);
  --tier-ink: #5f378f;
  --tier-shadow: rgba(142, 91, 199, 0.28);
}
.tier-tone--v6 {
  --tier-accent: #3d9c72;
  --tier-accent-soft: rgba(207, 239, 226, 0.98);
  --tier-ink: #22654a;
  --tier-shadow: rgba(61, 156, 114, 0.28);
}
.tier-tone--v7 {
  --tier-accent: #3b78c4;
  --tier-accent-soft: rgba(209, 225, 247, 0.98);
  --tier-ink: #1f4f89;
  --tier-shadow: rgba(59, 120, 196, 0.28);
}
.tier-tone--v8 {
  --tier-accent: #cb544d;
  --tier-accent-soft: rgba(246, 211, 208, 0.98);
  --tier-ink: #852d2a;
  --tier-shadow: rgba(203, 84, 77, 0.28);
}
.tier-tone--v9 {
  --tier-accent: #8f7237;
  --tier-accent-soft: rgba(241, 228, 188, 0.98);
  --tier-ink: #594419;
  --tier-shadow: rgba(143, 114, 55, 0.28);
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
  border-radius: 999rpx;
  border: 2rpx solid var(--page-border-warm);
  background: linear-gradient(180deg, rgba(252, 250, 245, 0.98), rgba(247, 243, 232, 0.94));
  color: #93876a;
  font-size: 24rpx;
}
.meta-note {
  display: inline-flex;
  align-items: center;
  justify-content: flex-end;
  flex: 0 0 auto;
  flex-shrink: 0;
  max-width: 100%;
  white-space: nowrap;
  line-height: 1;
  color: #a3916f;
  font-size: 24rpx;
  font-weight: 700;
  letter-spacing: 1rpx;
  padding: 8rpx 0 0;
}
.meta-note::before {
  content: '';
  width: 12rpx;
  height: 12rpx;
  margin-right: 10rpx;
  border-radius: 999rpx;
  background: linear-gradient(180deg, rgba(214, 184, 131, 0.9), rgba(197, 163, 103, 0.72));
  box-shadow: 0 0 0 6rpx rgba(214, 184, 131, 0.14);
}
.meta-note--hero { padding-top: 14rpx; }
.account-summary-grid {
  margin-top: 18rpx;
  display: grid;
  gap: 12rpx;
}
.briefing-card {
  border-radius: 28rpx;
  border: 2rpx solid var(--page-border);
  background: linear-gradient(180deg, rgba(255,255,255,0.98), rgba(248,250,253,0.98));
  padding: 18rpx 20rpx;
}
.briefing-card__label { color: #8f9198; font-size: 24rpx; }
.briefing-card__value {
  display: block;
  margin-top: 10rpx;
  color: #121212;
  font-size: 34rpx;
  font-weight: 800;
}
.briefing-card__subvalue {
  display: block;
  margin-top: 8rpx;
  color: #8f9198;
  font-size: 22rpx;
  line-height: 1.45;
}
.watch-list {
  margin-top: 18rpx;
  display: grid;
  gap: 12rpx;
}
.tier-preview-grid {
  margin-top: 22rpx;
  display: grid;
  gap: 14rpx;
}
.tier-preview-card {
  border-radius: 28rpx;
  padding: 22rpx 22rpx 24rpx;
  border: 2rpx solid var(--page-border);
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.98), rgba(246, 248, 252, 0.98));
}
.tier-preview-card--v3 {
  background: linear-gradient(180deg, rgba(255, 250, 240, 0.98), rgba(250, 240, 219, 0.98));
  border-color: var(--page-border-warm);
}
.tier-preview-card--v1 {
  background: linear-gradient(180deg, rgba(249, 251, 255, 0.98), rgba(239, 243, 250, 0.98));
}
.tier-preview-card__eyebrow {
  color: #7b818f;
  font-size: 22rpx;
  font-weight: 700;
  letter-spacing: 1rpx;
}
.tier-preview-card__title {
  display: block;
  margin-top: 10rpx;
  color: #15161b;
  font-size: 34rpx;
  font-weight: 800;
}
.tier-preview-card__body {
  display: block;
  margin-top: 10rpx;
  color: #6c7280;
  font-size: 24rpx;
  line-height: 1.6;
}
.mini-wechat-entry {
  margin-top: 20rpx;
  display: flex;
  flex-direction: column;
  gap: 12rpx;
}

.mini-wechat-entry--disabled {
  gap: 0;
}

.mini-wechat-entry__notice {
  padding: 24rpx 22rpx;
  border-radius: 28rpx;
  border: 2rpx solid var(--page-border-warm);
  background: linear-gradient(180deg, rgba(252, 250, 245, 0.98), rgba(246, 241, 231, 0.96));
  display: grid;
  gap: 12rpx;
}

.mini-wechat-entry__tag {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: fit-content;
  padding: 10rpx 16rpx;
  border-radius: 999rpx;
  background: rgba(155, 124, 64, 0.1);
  color: #8d6a2b;
  font-size: 21rpx;
  font-weight: 700;
  line-height: 1;
}

.mini-wechat-entry__title {
  color: #16181d;
  font-size: 30rpx;
  font-weight: 800;
  line-height: 1.2;
}

.mini-wechat-entry__desc {
  color: #716b5e;
  font-size: 23rpx;
  line-height: 1.65;
}

.mini-wechat-entry__button {
  align-self: stretch;
  background: linear-gradient(135deg, #111318, #20252f);
}
.watch-list__item {
  position: relative;
  padding-left: 24rpx;
}
.watch-list__item::before {
  content: '';
  position: absolute;
  top: 14rpx;
  left: 0;
  width: 10rpx;
  height: 10rpx;
  border-radius: 999rpx;
  background: #131313;
}
.watch-list__item text {
  color: #151515;
  font-size: 24rpx;
  line-height: 1.6;
}
.auth-form {
  margin-top: 22rpx;
  display: flex;
  flex-direction: column;
  gap: 16rpx;
}
.auth-input {
  width: 100%;
  height: 92rpx;
  padding: 0 24rpx;
  border-radius: 22rpx;
  border: 2rpx solid var(--page-border);
  background: #f7f8fb;
  font-size: 28rpx;
}
.form-footnote {
  color: #7c8089;
  font-size: 24rpx;
  line-height: 1.6;
}
.primary-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  align-self: flex-start;
  padding: 20rpx 30rpx;
  border-radius: 999rpx;
  background: #15161b;
  color: #ffffff;
  font-size: 28rpx;
  white-space: nowrap;
  line-height: 1;
}
.primary-action--ghost {
  background: #f6f7fb;
  color: #6d7280;
}
.account-actions {
  margin-top: 24rpx;
  border-radius: 28rpx;
  padding: 22rpx 24rpx;
  border: 2rpx solid var(--page-border-warm);
  background: linear-gradient(180deg, rgba(252, 252, 253, 0.98), rgba(245, 247, 250, 0.98));
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
  color: #15161b;
  font-size: 26rpx;
  font-weight: 700;
}
.account-actions__caption {
  display: block;
  margin-top: 8rpx;
  color: #7a808d;
  font-size: 23rpx;
  line-height: 1.55;
}
.logout-action {
  width: 100%;
  height: 78rpx;
  padding: 0 26rpx;
  border-radius: 999rpx;
  border: 2rpx solid var(--page-border-warm);
  background: linear-gradient(180deg, rgba(255, 249, 249, 0.98), rgba(250, 240, 240, 0.98));
  color: #8d5b5b;
  font-size: 26rpx;
  font-weight: 700;
  line-height: 78rpx;
  text-align: center;
  box-sizing: border-box;
}
.sheet-mask {
  position: fixed;
  inset: 0;
  z-index: 40;
  background: rgba(18, 20, 28, 0.36);
  backdrop-filter: blur(8rpx);
  display: flex;
  align-items: flex-end;
  animation: fi-overlay-fade-in 180ms ease both;
}
.sheet-card {
  width: 100%;
  border-radius: 36rpx 36rpx 0 0;
  background: rgba(255,255,255,0.98);
  padding: 28rpx 24rpx 40rpx;
  box-shadow: 0 -24rpx 56rpx rgba(12,14,20,0.12);
  animation: fi-sheet-up 240ms cubic-bezier(0.22, 1, 0.36, 1) both;
}
.mini-wechat-bind-form {
  margin-top: 22rpx;
  display: flex;
  flex-direction: column;
  gap: 16rpx;
}
.avatar-picker {
  width: 144rpx;
  height: 144rpx;
  padding: 0;
  border-radius: 999rpx;
  background: linear-gradient(180deg, rgba(247,248,251,0.98), rgba(239,242,248,0.98));
  border: 2rpx solid var(--page-border);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}
.avatar-picker__image {
  width: 100%;
  height: 100%;
}
.avatar-picker__placeholder {
  color: #8f9198;
  font-size: 24rpx;
}
.sheet-actions {
  margin-top: 24rpx;
  display: flex;
  gap: 16rpx;
}
.member-card {
  margin-top: 22rpx;
  border-radius: 30rpx;
  padding: 24rpx;
  border: 2rpx solid var(--page-border);
  background: linear-gradient(180deg, rgba(255,255,255,0.98), rgba(247,249,252,0.98));
}
.member-card--v3 {
  background: linear-gradient(180deg, rgba(255, 251, 242, 0.98), rgba(248, 239, 218, 0.98));
  border-color: var(--page-border-warm);
}
.member-card--v2 {
  background: linear-gradient(180deg, rgba(244, 252, 248, 0.98), rgba(232, 244, 238, 0.98));
  border-color: rgba(170, 205, 190, 0.85);
}
.member-card--v1 {
  background: linear-gradient(180deg, rgba(249, 251, 255, 0.98), rgba(239, 243, 250, 0.98));
  border-color: var(--page-border);
}
.member-card__title {
  display: block;
  margin-top: 12rpx;
  color: #15161b;
  font-size: 38rpx;
  font-weight: 800;
  line-height: 1.18;
}
.member-card__body {
  display: block;
  margin-top: 12rpx;
  color: #666d7b;
  font-size: 26rpx;
  line-height: 1.7;
}
.member-card__hero {
  display: flex;
  align-items: center;
  gap: 22rpx;
}
.member-card__content {
  min-width: 0;
  flex: 1;
}
.member-benefit-pill {
  margin-top: 18rpx;
  display: inline-flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10rpx;
  padding: 12rpx 16rpx;
  border-radius: 20rpx;
  background: rgba(255, 255, 255, 0.78);
  border: 2rpx solid var(--page-border-warm);
}
.member-benefit-pill__label {
  color: #8e845f;
  font-size: 22rpx;
  font-weight: 700;
}
.member-benefit-pill__value {
  color: #191919;
  font-size: 24rpx;
  font-weight: 700;
}
.tier-ladder-panel {
  margin-top: 24rpx;
}
.tier-ladder-list {
  margin-top: 20rpx;
  display: grid;
  gap: 14rpx;
}
.tier-ladder-card {
  border-radius: 28rpx;
  padding: 22rpx;
  border: 2rpx solid var(--page-border);
  background: linear-gradient(180deg, rgba(255,255,255,0.98), rgba(246,248,252,0.98));
}
.tier-ladder-card.tier-tone--v1 {
  --tier-pulse: rgba(127, 144, 170, 0.14);
  border-color: rgba(205, 214, 231, 0.95);
  background: linear-gradient(180deg, rgba(249, 251, 255, 0.98), rgba(238, 244, 252, 0.98));
}
.tier-ladder-card.tier-tone--v2 {
  --tier-pulse: rgba(106, 158, 142, 0.14);
  border-color: rgba(170, 205, 190, 0.95);
  background: linear-gradient(180deg, rgba(244, 252, 248, 0.98), rgba(232, 244, 238, 0.98));
}
.tier-ladder-card.tier-tone--v3 {
  --tier-pulse: rgba(201, 150, 66, 0.16);
  border-color: rgba(230, 204, 152, 0.95);
  background: linear-gradient(180deg, rgba(255, 252, 245, 0.98), rgba(249, 239, 217, 0.98));
}
.tier-ladder-card.tier-tone--v4 {
  --tier-pulse: rgba(207, 105, 78, 0.16);
  border-color: rgba(233, 191, 179, 0.95);
  background: linear-gradient(180deg, rgba(255, 250, 248, 0.98), rgba(250, 235, 229, 0.98));
}
.tier-ladder-card.tier-tone--v5 {
  --tier-pulse: rgba(142, 91, 199, 0.16);
  border-color: rgba(214, 197, 239, 0.95);
  background: linear-gradient(180deg, rgba(251, 248, 255, 0.98), rgba(242, 235, 252, 0.98));
}
.tier-ladder-card.tier-tone--v6 {
  --tier-pulse: rgba(61, 156, 114, 0.16);
  border-color: rgba(181, 223, 203, 0.95);
  background: linear-gradient(180deg, rgba(247, 253, 250, 0.98), rgba(232, 247, 240, 0.98));
}
.tier-ladder-card.tier-tone--v7 {
  --tier-pulse: rgba(59, 120, 196, 0.16);
  border-color: rgba(192, 211, 239, 0.95);
  background: linear-gradient(180deg, rgba(247, 250, 255, 0.98), rgba(233, 242, 252, 0.98));
}
.tier-ladder-card.tier-tone--v8 {
  --tier-pulse: rgba(203, 84, 77, 0.16);
  border-color: rgba(236, 193, 190, 0.95);
  background: linear-gradient(180deg, rgba(255, 249, 249, 0.98), rgba(250, 235, 235, 0.98));
}
.tier-ladder-card.tier-tone--v9 {
  --tier-pulse: rgba(214, 159, 47, 0.16);
  border-color: rgba(221, 200, 148, 0.95);
  background: linear-gradient(180deg, rgba(255, 252, 244, 0.98), rgba(246, 236, 208, 0.98));
}
.tier-ladder-card.tier-ladder-card--active {
  box-shadow: 0 16rpx 44rpx rgba(201, 150, 66, 0.22);
  transform: translateY(-2rpx);
}
.tier-ladder-card.tier-tone--v1.tier-ladder-card--active {
  box-shadow: 0 16rpx 44rpx rgba(127, 144, 170, 0.22);
}
.tier-ladder-card.tier-tone--v2.tier-ladder-card--active {
  box-shadow: 0 16rpx 44rpx rgba(106, 158, 142, 0.22);
}
.tier-ladder-card.tier-tone--v3.tier-ladder-card--active {
  box-shadow: 0 16rpx 44rpx rgba(201, 150, 66, 0.22);
}
.tier-ladder-card.tier-tone--v4.tier-ladder-card--active {
  box-shadow: 0 16rpx 44rpx rgba(207, 105, 78, 0.22);
}
.tier-ladder-card.tier-tone--v5.tier-ladder-card--active {
  box-shadow: 0 16rpx 44rpx rgba(142, 91, 199, 0.22);
}
.tier-ladder-card.tier-tone--v6.tier-ladder-card--active {
  box-shadow: 0 16rpx 44rpx rgba(61, 156, 114, 0.22);
}
.tier-ladder-card.tier-tone--v7.tier-ladder-card--active {
  box-shadow: 0 16rpx 44rpx rgba(59, 120, 196, 0.22);
}
.tier-ladder-card.tier-tone--v8.tier-ladder-card--active {
  box-shadow: 0 16rpx 44rpx rgba(203, 84, 77, 0.22);
}
.tier-ladder-card.tier-tone--v9.tier-ladder-card--active {
  box-shadow: 0 16rpx 56rpx rgba(190, 130, 0, 0.65) !important;
}
.tier-ladder-card__top {
  display: flex;
  align-items: center;
  gap: 18rpx;
}
.tier-ladder-card__heading {
  min-width: 0;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 6rpx;
}
.tier-ladder-card__code {
  color: #14161c;
  font-size: 30rpx;
  font-weight: 800;
  line-height: 1;
}
.tier-ladder-card__name {
  color: #7a808d;
  font-size: 22rpx;
  font-weight: 700;
}
.tier-ladder-card__refresh {
  align-self: flex-start;
  padding: 10rpx 14rpx;
  border-radius: 999rpx;
  background: rgba(255,255,255,0.78);
  border: 2rpx solid rgba(237, 230, 214, 0.94);
  color: #76684b;
  font-size: 22rpx;
  font-weight: 700;
  line-height: 1;
  white-space: nowrap;
}
.tier-ladder-card__condition,
.tier-ladder-card__body {
  display: block;
  margin-top: 14rpx;
  color: #1c1f25;
  font-size: 24rpx;
  line-height: 1.65;
}
.tier-ladder-card__body {
  margin-top: 8rpx;
  color: #6a707d;
}
.tier-ladder-panel__footnote {
  display: block;
  margin-top: 16rpx;
  color: #8a8f9a;
  font-size: 22rpx;
  line-height: 1.6;
}

.account-hero {
  --member-highlight: rgba(255, 232, 179, 0.92);
  --member-wash: rgba(255, 243, 214, 0.68);
  --member-border: rgba(235, 211, 155, 0.92);
  --member-eyebrow: #a87420;
  --member-accent: #b8842b;
  --member-accent-strong: #9f6c18;
  --member-badge-from: #d8ac54;
  --member-badge-to: #ad7421;
  --member-soft-surface: rgba(255, 247, 229, 0.72);
  --member-soft-border: rgba(236, 212, 167, 0.94);
  --member-medal-ink: #c08b2d;
  --member-medal-crown: rgba(201, 150, 66, 0.72);
  --member-glow: rgba(207, 162, 71, 0.22);
  --member-privilege-border: rgba(235, 220, 191, 0.76);
  --member-privilege-bg: rgba(250, 244, 232, 0.96);
  --member-privilege-icon-bg: rgba(191, 141, 53, 0.12);
  --member-privilege-icon-color: #b07b29;
  position: relative;
  overflow: hidden;
  padding: 24rpx 24rpx 28rpx;
  border-color: var(--member-border);
  box-shadow: 0 18rpx 40rpx rgba(184, 143, 52, 0.1);
}

.account-hero::before {
  content: '';
  position: absolute;
  inset: 0;
  background:
    radial-gradient(circle at 82% 18%, var(--member-highlight), transparent 34%),
    radial-gradient(circle at 78% 52%, var(--member-wash), transparent 40%);
  pointer-events: none;
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
  color: var(--member-eyebrow);
}

.account-hero__eyebrow-icon {
  width: 26rpx;
  height: 26rpx;
  flex-shrink: 0;
}

.account-hero__eyebrow-text {
  font-size: 22rpx;
  font-weight: 700;
  line-height: 1;
}

.profile-banner--hero {
  position: relative;
  z-index: 1;
  margin-top: 18rpx;
  align-items: center;
  justify-content: space-between;
  gap: 18rpx;
}

.profile-banner__main {
  min-width: 0;
  flex: 1;
  display: flex;
  align-items: center;
  gap: 18rpx;
}

.profile-banner__avatar-wrap {
  position: relative;
  flex-shrink: 0;
}

.profile-banner__avatar {
  width: 124rpx;
  height: 124rpx;
  background: linear-gradient(180deg, rgba(255,255,255,0.96), rgba(247, 241, 228, 0.94));
  border: 4rpx solid rgba(255, 255, 255, 0.92);
  box-shadow: 0 12rpx 26rpx rgba(142, 108, 31, 0.14);
}

.profile-banner__avatar-badge {
  position: absolute;
  right: -2rpx;
  bottom: -2rpx;
  min-width: 44rpx;
  height: 44rpx;
  padding: 0 12rpx;
  border-radius: 999rpx;
  border: 3rpx solid rgba(255, 255, 255, 0.94);
  background: linear-gradient(180deg, var(--member-accent), var(--member-accent-strong));
  color: #ffffff;
  font-size: 20rpx;
  font-weight: 800;
  line-height: 38rpx;
  text-align: center;
  box-sizing: border-box;
}

.profile-banner__body {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 14rpx;
}

.profile-banner__name {
  font-size: 36rpx;
  line-height: 1.06;
}

.membership-badge {
  padding: 14rpx 22rpx;
  font-size: 24rpx;
}

.account-hero__medal {
  position: relative;
  width: 214rpx;
  height: 194rpx;
  flex-shrink: 0;
}

.account-hero__medal-ring {
  position: absolute;
  top: 10rpx;
  right: 0;
  width: 194rpx;
  height: 194rpx;
  border-radius: 999rpx;
  background:
    repeating-radial-gradient(circle, rgba(255,255,255,0) 0 9rpx, rgba(255,255,255,0.38) 9rpx 12rpx),
    radial-gradient(circle, rgba(255, 241, 209, 0.56), rgba(255,255,255,0) 72%);
  opacity: 0.9;
}

.account-hero__medal-orbit {
  position: absolute;
  top: 92rpx;
  left: 12rpx;
  width: 186rpx;
  height: 46rpx;
  border-radius: 999rpx;
  border: 4rpx solid rgba(255, 255, 255, 0.76);
  border-left-color: transparent;
  border-right-color: transparent;
  transform: rotate(-8deg);
  opacity: 0.88;
}

.account-hero__medal-shield {
  position: absolute;
  top: 22rpx;
  right: 16rpx;
  width: 136rpx;
  height: 156rpx;
  clip-path: polygon(50% 0%, 88% 16%, 88% 62%, 50% 100%, 12% 62%, 12% 16%);
  background: linear-gradient(180deg, rgba(255,255,255,0.98), rgba(247, 234, 199, 0.98));
  border: 2rpx solid rgba(242, 221, 171, 0.96);
  box-shadow:
    0 18rpx 34rpx var(--member-glow),
    inset 0 -16rpx 22rpx rgba(218, 175, 78, 0.16);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}

.account-hero__medal-crown {
  color: var(--member-medal-crown);
  font-size: 24rpx;
  line-height: 1;
}

.account-hero__medal-code {
  margin-top: 4rpx;
  color: var(--member-medal-ink);
  font-size: 66rpx;
  font-weight: 900;
  line-height: 0.92;
  letter-spacing: -2rpx;
}

.account-hero__medal-star {
  position: absolute;
  width: 10rpx;
  height: 10rpx;
  border-radius: 999rpx;
  background: rgba(255, 255, 255, 0.96);
  box-shadow: 0 0 0 6rpx rgba(255,255,255,0.18);
}

.account-hero__medal-star--one {
  top: 34rpx;
  right: 174rpx;
}

.account-hero__medal-star--two {
  right: 8rpx;
  top: 100rpx;
}

.profile-banner__invite--hero {
  position: relative;
  z-index: 1;
  margin-top: 18rpx;
  padding: 18rpx 20rpx;
  border-radius: 28rpx;
  border: 1rpx solid var(--member-soft-border);
  background: linear-gradient(180deg, rgba(255,255,255,0.78), var(--member-soft-surface));
  backdrop-filter: blur(8rpx);
}

.profile-banner__invite-label {
  color: var(--member-accent-strong);
  font-size: 22rpx;
}

.profile-banner__invite-code {
  font-size: 26rpx;
}

.profile-banner__invite-copy {
  min-width: 164rpx;
  height: 68rpx;
  line-height: 68rpx;
  border: none;
  background: linear-gradient(135deg, var(--member-badge-from), var(--member-badge-to));
  color: #ffffff;
  box-shadow: 0 12rpx 22rpx rgba(170, 117, 32, 0.2);
}

.info-panel,
.privilege-panel,
.upgrade-panel {
  padding: 20rpx;
  border-radius: 32rpx;
}
.info-panel,
.privilege-panel {
  border-color: var(--page-border-warm);
}
.info-panel {
  animation: fi-fade-in-up 520ms cubic-bezier(0.22, 1, 0.36, 1) 80ms both;
}
.privilege-panel {
  animation: fi-fade-in-up 520ms cubic-bezier(0.22, 1, 0.36, 1) 160ms both;
}
.upgrade-panel {
  animation: fi-fade-in-up 520ms cubic-bezier(0.22, 1, 0.36, 1) 240ms both;
}

.info-panel__title,
.privilege-panel__title,
.upgrade-panel__title {
  color: #1b1c20;
  font-size: 28rpx;
  font-weight: 800;
  line-height: 1;
}

.account-info-grid {
  margin-top: 22rpx;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  border-radius: 28rpx;
  overflow: hidden;
  border: 2rpx solid var(--page-border-warm);
  background: linear-gradient(180deg, rgba(255,255,255,0.98), rgba(249,250,253,0.98));
}

.account-info-item {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 16rpx;
  padding: 22rpx 18rpx;
  box-sizing: border-box;
  border-top: 2rpx solid var(--page-border-warm);
}

.account-info-item:nth-child(-n + 2) {
  border-top: none;
}

.account-info-item:nth-child(even) {
  border-left: 2rpx solid var(--page-border-warm);
}

.account-info-item__icon {
  width: 66rpx;
  height: 66rpx;
  border-radius: 999rpx;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(180deg, rgba(251, 244, 230, 0.98), rgba(244, 231, 203, 0.98));
  color: #b18135;
  font-size: 26rpx;
  font-weight: 800;
  box-shadow: inset 0 0 0 2rpx rgba(236, 217, 179, 0.72);
}

.account-info-item__body {
  min-width: 0;
  flex: 1;
  display: grid;
  gap: 10rpx;
}

.account-info-item__label {
  color: #989ca7;
  font-size: 22rpx;
  line-height: 1;
}

.account-info-item__value {
  color: #17181c;
  font-size: 26rpx;
  font-weight: 800;
  line-height: 1.15;
  word-break: break-word;
}

.privilege-panel__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16rpx;
}

.privilege-panel__title-row {
  display: flex;
  align-items: center;
  gap: 10rpx;
}

.privilege-panel__crown {
  color: #af7c2b;
  font-size: 24rpx;
  line-height: 1;
}

.privilege-panel__hint {
  color: #ac9875;
  font-size: 22rpx;
  line-height: 1;
}

.privilege-grid {
  margin-top: 22rpx;
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12rpx;
}

.privilege-card {
  min-width: 0;
  padding: 18rpx 12rpx;
  border-radius: 24rpx;
  border: 2rpx solid var(--member-privilege-border);
  background: linear-gradient(180deg, rgba(255,255,255,0.96), var(--member-privilege-bg));
  display: grid;
  justify-items: center;
  align-content: start;
  gap: 10rpx;
  text-align: center;
}

.privilege-card__icon {
  width: 56rpx;
  height: 56rpx;
  border-radius: 999rpx;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--member-privilege-icon-bg);
  color: var(--member-privilege-icon-color);
  font-size: 24rpx;
  font-weight: 800;
}

.privilege-card__title {
  color: #1e2025;
  font-size: 22rpx;
  font-weight: 800;
  line-height: 1.3;
}

.privilege-card__caption {
  color: #8a8f9b;
  font-size: 20rpx;
  line-height: 1.35;
}

.upgrade-panel {
  border-color: var(--page-border-warm);
  background: linear-gradient(180deg, rgba(255,255,255,0.98), rgba(251, 247, 239, 0.98));
}

.upgrade-panel__title-row {
  display: flex;
  align-items: center;
  gap: 12rpx;
}

.upgrade-panel__info-icon {
  width: 34rpx;
  height: 34rpx;
  border-radius: 999rpx;
  border: 2rpx solid rgba(178, 130, 42, 0.58);
  color: #a27324;
  font-size: 22rpx;
  font-weight: 700;
  line-height: 30rpx;
  text-align: center;
  box-sizing: border-box;
}

.upgrade-panel__body {
  display: block;
  margin-top: 16rpx;
  color: #7a735f;
  font-size: 24rpx;
  line-height: 1.7;
}

.upgrade-step-list {
  margin-top: 22rpx;
  display: grid;
  gap: 14rpx;
}

.tier-ladder-card--compact {
  padding: 20rpx 22rpx;
  border-radius: 24rpx;
  box-shadow: none;
  transform: none;
}
.tier-ladder-card.tier-ladder-card--compact.tier-ladder-card--active {
  box-shadow: 0 8rpx 28rpx rgba(201, 150, 66, 0.18);
}
.tier-ladder-card.tier-tone--v1.tier-ladder-card--compact.tier-ladder-card--active {
  box-shadow: 0 8rpx 28rpx rgba(127, 144, 170, 0.18);
}
.tier-ladder-card.tier-tone--v2.tier-ladder-card--compact.tier-ladder-card--active {
  box-shadow: 0 8rpx 28rpx rgba(106, 158, 142, 0.18);
}
.tier-ladder-card.tier-tone--v3.tier-ladder-card--compact.tier-ladder-card--active {
  box-shadow: 0 8rpx 28rpx rgba(201, 150, 66, 0.18);
}
.tier-ladder-card.tier-tone--v4.tier-ladder-card--compact.tier-ladder-card--active {
  box-shadow: 0 8rpx 28rpx rgba(207, 105, 78, 0.18);
}
.tier-ladder-card.tier-tone--v5.tier-ladder-card--compact.tier-ladder-card--active {
  box-shadow: 0 8rpx 28rpx rgba(142, 91, 199, 0.18);
}
.tier-ladder-card.tier-tone--v6.tier-ladder-card--compact.tier-ladder-card--active {
  box-shadow: 0 8rpx 28rpx rgba(61, 156, 114, 0.18);
}
.tier-ladder-card.tier-tone--v7.tier-ladder-card--compact.tier-ladder-card--active {
  box-shadow: 0 8rpx 28rpx rgba(59, 120, 196, 0.18);
}
.tier-ladder-card.tier-tone--v8.tier-ladder-card--compact.tier-ladder-card--active {
  box-shadow: 0 8rpx 28rpx rgba(203, 84, 77, 0.18);
}
.tier-ladder-card.tier-tone--v9.tier-ladder-card--compact.tier-ladder-card--active {
  box-shadow: 0 16rpx 56rpx rgba(190, 130, 0, 0.65) !important;
}

.tier-ladder-card--compact .tier-ladder-card__top {
  align-items: flex-start;
}

.tier-ladder-card--compact .tier-ladder-card__condition {
  margin-top: 10rpx;
  font-size: 22rpx;
  line-height: 1.6;
}

.upgrade-step-card__title-row {
  display: flex;
  align-items: center;
  gap: 10rpx;
  flex-wrap: wrap;
}

.upgrade-step-card__aside {
  display: grid;
  justify-items: end;
  gap: 10rpx;
}

.upgrade-step-card__current {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 8rpx 14rpx;
  border-radius: 999rpx;
  background: rgba(255,255,255,0.78);
  border: 2rpx solid rgba(237, 230, 214, 0.94);
  color: #76684b;
  font-size: 20rpx;
  font-weight: 700;
  line-height: 1;
  white-space: nowrap;
}

.account-actions {
  margin-top: 0;
  border-radius: 28rpx;
  padding: 18rpx 20rpx;
  animation: fi-fade-in-up 520ms cubic-bezier(0.22, 1, 0.36, 1) 320ms both;
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
  --member-eyebrow: #a87420;
  --member-accent: #c79035;
  --member-accent-strong: #9f6c18;
  --member-badge-from: #ddb359;
  --member-badge-to: #b17a22;
  --member-soft-surface: rgba(255, 247, 229, 0.72);
  --member-soft-border: rgba(236, 212, 167, 0.94);
  --member-medal-ink: #c08b2d;
  --member-medal-crown: rgba(201, 150, 66, 0.72);
  --member-glow: rgba(207, 162, 71, 0.22);
  --member-privilege-border: rgba(235, 220, 191, 0.76);
  --member-privilege-bg: rgba(250, 244, 232, 0.96);
  --member-privilege-icon-bg: rgba(191, 141, 53, 0.12);
  --member-privilege-icon-color: #b07b29;
}

.account-hero.tier-tone--v4,
.privilege-panel.tier-tone--v4 {
  --member-highlight: rgba(255, 214, 198, 0.92);
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
  --member-highlight: rgba(255, 226, 156, 0.94);
  --member-wash: rgba(255, 241, 205, 0.78);
  --member-border: rgba(227, 194, 121, 0.94);
  --member-eyebrow: #9d6a16;
  --member-accent: #d8a63f;
  --member-accent-strong: #a96e14;
  --member-badge-from: #e0b04d;
  --member-badge-to: #b37215;
  --member-soft-surface: rgba(255, 246, 220, 0.84);
  --member-soft-border: rgba(237, 210, 149, 0.96);
  --member-medal-ink: #cb9225;
  --member-medal-crown: rgba(214, 159, 47, 0.76);
  --member-glow: rgba(221, 168, 58, 0.24);
  --member-privilege-border: rgba(237, 217, 172, 0.84);
  --member-privilege-bg: rgba(255, 247, 226, 0.97);
  --member-privilege-icon-bg: rgba(216, 166, 63, 0.14);
  --member-privilege-icon-color: #bb8420;
}

.purchase-entry {
  position: relative;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20rpx;
  margin-top: 22rpx;
  padding: 24rpx 24rpx 24rpx 26rpx;
  border-radius: 28rpx;
  border: 2rpx solid var(--member-soft-border);
  background:
    linear-gradient(135deg, rgba(255,255,255,0.72), rgba(255,255,255,0)),
    linear-gradient(180deg, rgba(255, 250, 238, 0.96), rgba(248, 236, 204, 0.92));
  box-shadow:
    inset 0 1rpx 0 rgba(255,255,255,0.72),
    0 14rpx 28rpx rgba(170, 126, 33, 0.12);
  backdrop-filter: blur(8rpx);
}

.purchase-entry::before,
.purchase-entry::after {
  content: '';
  position: absolute;
  border-radius: 999rpx;
  pointer-events: none;
}

.purchase-entry::before {
  top: -26rpx;
  right: -18rpx;
  width: 180rpx;
  height: 180rpx;
  background: radial-gradient(circle, rgba(255, 238, 191, 0.7), rgba(255,255,255,0) 68%);
}

.purchase-entry::after {
  left: 28rpx;
  bottom: -44rpx;
  width: 220rpx;
  height: 120rpx;
  background: radial-gradient(circle, rgba(232, 195, 111, 0.18), rgba(255,255,255,0) 72%);
}

.purchase-entry__body {
  position: relative;
  z-index: 1;
  min-width: 0;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 10rpx;
}

.purchase-entry__eyebrow {
  display: inline-flex;
  align-items: center;
  gap: 8rpx;
  color: var(--member-eyebrow);
  font-size: 22rpx;
  font-weight: 700;
  line-height: 1;
}

.purchase-entry__eyebrow-icon {
  width: 24rpx;
  height: 24rpx;
  flex-shrink: 0;
}

.purchase-entry__label {
  color: #493413;
  font-size: 34rpx;
  font-weight: 800;
  line-height: 1.08;
}

.purchase-entry__hint {
  color: rgba(104, 80, 36, 0.82);
  font-size: 23rpx;
  line-height: 1.55;
}

.purchase-entry__aside {
  position: relative;
  z-index: 1;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  justify-content: center;
  gap: 12rpx;
}

.purchase-entry__cta {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 132rpx;
  height: 52rpx;
  padding: 0 18rpx;
  border-radius: 999rpx;
  background: linear-gradient(135deg, var(--member-badge-from), var(--member-badge-to));
  color: #ffffff;
  font-size: 22rpx;
  font-weight: 700;
  line-height: 1;
  box-shadow: 0 10rpx 20rpx rgba(176, 122, 35, 0.24);
}

.purchase-entry__arrow {
  width: 54rpx;
  height: 54rpx;
  border-radius: 999rpx;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.68);
  border: 2rpx solid rgba(234, 212, 162, 0.92);
  color: var(--member-accent-strong);
  font-size: 34rpx;
  font-weight: 700;
  line-height: 1;
}
</style>
