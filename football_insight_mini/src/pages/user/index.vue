<template>
  <view
    class="page-root"
    :class="{ 'page-root--guest': isGuestPage }"
  >
    <FiBrandNav
      open-on-current-page
      :transparent="isGuestPage"
      @open-ai="openAiFromBrandNav"
    />
    <image class="page-bg-img" :src="phoenixStadiumBgImage" mode="aspectFill" />
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
      <view
        v-if="currentUser"
        class="hero-card account-hero"
        :class="[currentMembershipMeta.heroClass, currentMembershipGuide.toneClass]"
      >
        <image class="account-hero__dots account-hero__dots--top" :src="memberCardDotsImage" mode="aspectFill" />
        <image class="account-hero__dots account-hero__dots--bottom" :src="memberCardDotsImage" mode="aspectFill" />

        <view class="hero-card__top hero-card__top--member">
          <view class="account-hero__eyebrow">
            <image class="account-hero__eyebrow-icon" :src="diamondIcon" mode="aspectFit" />
            <text class="account-hero__eyebrow-text">会员中心</text>
          </view>
        </view>

        <view v-if="currentUser" class="member-identity">
          <view class="member-identity__avatar">
            <image
              v-if="currentUser.avatar_url"
              :src="currentUser.avatar_url"
              mode="aspectFill"
              class="member-identity__avatar-image"
            />
            <text v-else class="member-identity__avatar-fallback">{{ avatarFallbackLabel }}</text>
          </view>

          <view class="member-identity__copy">
            <text class="member-identity__hello">亲爱的会员，您好！</text>
            <text class="member-identity__desc">{{ membershipHeroDescription }}</text>
          </view>

        </view>

        <view
          class="membership-card"
          :class="{ 'membership-card--actionable': canPurchaseMembership }"
          @click="handleMembershipCardAction"
        >
          <view class="membership-card__content">
            <view class="membership-card__main">
              <text class="membership-card__title">{{ membershipHeroTitle }}</text>
              <text class="membership-card__expire">{{ membershipHeroExpiryLabel }}</text>
            </view>
            <view class="membership-card__action">
              <text class="membership-card__action-text">{{ membershipHeroActionText }}</text>
              <text v-if="canPurchaseMembership" class="membership-card__action-arrow">›</text>
            </view>
          </view>
          <view class="membership-card__level">{{ currentMembershipMeta.code }}</view>
        </view>

        <view v-if="currentUserInviteCode" class="profile-banner__invite profile-banner__invite--hero">
          <view class="profile-banner__invite-body">
            <text class="profile-banner__invite-label">我的邀请码</text>
            <text class="profile-banner__invite-code">{{ currentUserInviteCode }}</text>
          </view>
          <button class="profile-banner__invite-copy" :class="currentMembershipGuide.toneClass" @click="handleCopyInviteCode">一键复制</button>
        </view>

      </view>

      <view v-if="loading" class="skeleton-stack">
        <view class="hero-card account-hero skeleton-panel skeleton-account-hero">
          <view class="hero-card__top hero-card__top--member">
            <view class="skeleton-copy-group">
              <view class="skeleton-line skeleton-line--kicker" />
              <view class="skeleton-line skeleton-line--title" />
            </view>
            <view class="skeleton-pill skeleton-pill--short" />
          </view>

          <view class="skeleton-profile-row">
            <view class="skeleton-line skeleton-line--avatar" />
            <view class="skeleton-profile-body">
              <view class="skeleton-line skeleton-line--profile-name" />
              <view class="skeleton-line skeleton-line--profile-badge" />
            </view>
            <view class="skeleton-line skeleton-line--medal" />
          </view>

          <view class="skeleton-invite-row">
            <view class="skeleton-copy-group">
              <view class="skeleton-line skeleton-line--label" />
              <view class="skeleton-line skeleton-line--invite-code" />
            </view>
            <view class="skeleton-button skeleton-button--compact" />
          </view>
        </view>

        <view class="panel skeleton-panel">
          <view class="skeleton-line skeleton-line--section" />
          <view class="skeleton-account-grid">
            <view
              v-for="index in 5"
              :key="`user-account-skeleton-${index}`"
              class="skeleton-account-cell"
            >
              <view class="skeleton-line skeleton-line--account-icon" />
              <view class="skeleton-copy-group">
                <view class="skeleton-line skeleton-line--label" />
                <view class="skeleton-line skeleton-line--account-value" />
              </view>
            </view>
          </view>
        </view>

        <view class="panel skeleton-panel">
          <view class="privilege-panel__header">
            <view class="skeleton-line skeleton-line--section" />
            <view class="skeleton-pill skeleton-pill--short" />
          </view>
          <view class="skeleton-privilege-grid">
            <view
              v-for="index in 4"
              :key="`user-privilege-skeleton-${index}`"
              class="skeleton-privilege-card"
            >
              <view class="skeleton-line skeleton-line--privilege-icon" />
              <view class="skeleton-line skeleton-line--privilege-title" />
              <view class="skeleton-line skeleton-line--privilege-caption" />
            </view>
          </view>
        </view>

        <view class="panel skeleton-panel">
          <view class="skeleton-line skeleton-line--section" />
          <view class="skeleton-line skeleton-line--body" />
          <view class="skeleton-line skeleton-line--body skeleton-line--body-short" />
        </view>
      </view>

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
              <view class="account-info-item__icon">
                <image
                  class="account-info-item__icon-image"
                  :src="accountInfoIconMap[item.iconName]"
                  mode="aspectFit"
                />
              </view>
              <view class="account-info-item__body">
                <text class="account-info-item__label">{{ item.label }}</text>
                <text class="account-info-item__value">{{ item.value }}</text>
              </view>
            </view>
          </view>
        </view>

        <view class="panel notification-email-panel">
          <view class="notification-email-panel__body">
            <text class="notification-email-panel__title">回流提醒邮箱</text>
            <text class="notification-email-panel__value">{{ notificationEmailLabel }}</text>
          </view>
          <button class="notification-email-panel__action" @click="openNotificationEmailSheet">
            {{ notificationEmail ? '编辑' : '填写' }}
          </button>
        </view>

        <view class="panel privilege-panel" :class="currentMembershipGuide.toneClass">
          <view class="privilege-panel__header">
            <view class="privilege-panel__title-row">
              <text class="privilege-panel__title">{{ currentMembershipMeta.code }} 专属权益</text>
            </view>
          </view>

          <view class="privilege-grid">
            <view
              v-for="item in membershipBenefitItems"
              :key="item.key"
              class="privilege-card"
            >
              <view class="privilege-card__icon">
                <image
                  class="privilege-card__icon-image"
                  :src="benefitIconMap[item.iconName]"
                  mode="aspectFit"
                />
              </view>
              <view class="privilege-card__body">
                <text class="privilege-card__title">{{ item.title }}</text>
                <text class="privilege-card__caption">{{ item.caption }}</text>
              </view>
            </view>
          </view>
        </view>

        <view class="panel upgrade-panel">
          <view class="upgrade-panel__title-row">
            <image class="upgrade-panel__info-icon" :src="infoIcon" mode="aspectFit" />
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

      <view v-if="miniWechatBindState" class="sheet-mask" @tap="handleSheetMaskTap">
        <view class="sheet-card" @tap.stop="consumeSheetTap">
          <view class="section-heading section-heading--compact">
            <view>
              <text class="section-kicker">首次微信登录</text>
              <text class="section-title">补充头像和昵称</text>
            </view>
            <text class="meta-note">完成后直接登录</text>
          </view>

          <text class="account-form-panel__summary">
            完成一次绑定后，后续就能直接用微信进入。邀请码和推荐码都可选。
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
              placeholder="请输入邀请码（可选）"
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
      <view v-if="notificationEmailSheetVisible" class="sheet-mask" @tap="closeNotificationEmailSheet">
        <view class="sheet-card" @tap.stop="consumeSheetTap">
          <view class="section-heading section-heading--compact">
            <view>
              <text class="section-kicker">邮箱提醒</text>
              <text class="section-title">编辑回流提醒邮箱</text>
            </view>
          </view>
          <text class="account-form-panel__summary">
            订阅回流提醒后，新增回流会发送到这个邮箱。
          </text>
          <input
            v-model="notificationEmailForm"
            class="auth-input"
            type="text"
            placeholder="请输入邮箱地址"
          />
          <view class="sheet-actions">
            <button class="primary-action primary-action--ghost" @click="closeNotificationEmailSheet">取消</button>
            <button class="primary-action" :disabled="notificationEmailSaving" @click="saveNotificationEmail">
              {{ notificationEmailSaving ? '保存中...' : '保存邮箱' }}
            </button>
          </view>
        </view>
      </view>
      </template>
    </view>
    </view>

    <FiAiChatSheet
      :visible="aiChatVisible"
      :current-user="currentAiUser"
      :ai-chat-mode="aiPublicConfig?.ai_chat_mode"
      @close="closeAiChat"
    />

    <view v-if="isGuestPage" class="guest-chant-wall">
      <view class="guest-chant-wall__meta">
        <text class="guest-chant-wall__kicker">MATCHDAY AT PHOENIX HILL</text>
        <text class="guest-chant-wall__title">成都主场的歌声</text>
      </view>

      <view class="guest-chant-wall__lyrics">
        <view class="guest-chant-wall__fade guest-chant-wall__fade--top"></view>
        <view class="guest-chant-wall__fade guest-chant-wall__fade--bottom"></view>
        <view class="guest-chant-wall__track">
          <text
            v-for="(line, index) in guestChantLoopLines"
            :key="`guest-chant-${index}`"
            class="guest-chant-wall__line"
            :class="{ 'guest-chant-wall__line--focus': index % guestChantLines.length === 2 }"
          >
            {{ line }}
          </text>
        </view>
      </view>

      <view class="guest-chant-wall__meter">
        <view
          v-for="(height, index) in guestChantMeterBars"
          :key="`guest-chant-meter-${index}`"
          class="guest-chant-wall__bar"
          :style="{ height: `${height}rpx`, animationDelay: `${index * -0.13}s` }"
        ></view>
      </view>
    </view>

    <FiLoginFloat
      v-if="isGuestPage"
      :disabled="isH5"
      :action-text="isH5 ? '小程序登录' : '去登录'"
      @action="handleMiniWechatLogin"
    />
  </view>
</template>

<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import { onShow } from '@dcloudio/uni-app'
import FiBrandNav from '../../components/FiBrandNav.vue'
import FiAiChatSheet from '../../components/FiAiChatSheet.vue'
import FiLoginFloat from '../../components/FiLoginFloat.vue'
import activityIcon from '../../static/user/activity.svg'
import badgeCheckIcon from '../../static/user/badge-check.svg'
import calendarCheckIcon from '../../static/user/calendar-check.svg'
import calendarDaysIcon from '../../static/user/calendar-days.svg'
import diamondIcon from '../../static/user/diamond.svg'
import historyIcon from '../../static/user/history.svg'
import infoIcon from '../../static/user/info.svg'
import logInIcon from '../../static/user/log-in.svg'
import memberCardDotsImage from '../../static/user/member-card-dots.png'
import phoenixStadiumBgImage from '../../static/user/phoenix-stadium-bg.webp'
import radarIcon from '../../static/user/radar.svg'
import ticketIcon from '../../static/user/ticket.svg'
import {
  bindMiniWechatAccount,
  getCurrentUser,
  getNotificationEmail,
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
  isValidNotificationEmail,
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
import { useAiChatSheet } from '../../composables/useAiChatSheet'

const hasLocalAccessToken = ref(Boolean(getAccessToken()))
const loading = ref(hasLocalAccessToken.value)
const currentUser = ref<CurrentUser | null>(null)
const publicConfig = ref<PublicSystemConfig | null>(null)
const systemConfigUnderReview = ref(false)
const notificationEmail = ref('')
const notificationEmailForm = ref('')
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

const accountInfoIconMap: Record<UserAccountInfoItem['iconName'], string> = {
  'badge-check': badgeCheckIcon,
  'calendar-days': calendarDaysIcon,
  'calendar-check': calendarCheckIcon,
  'log-in': logInIcon,
}
const benefitIconMap: Record<UserBenefitItem['iconName'], string> = {
  ticket: ticketIcon,
  activity: activityIcon,
  radar: radarIcon,
  history: historyIcon,
}

const isGuestPage = computed(() => !systemConfigUnderReview.value && (!hasLocalAccessToken.value || (!loading.value && !currentUser.value)))
const guestChantLines = [
  '凤凰山的灯光亮起',
  '红色看台一起呼吸',
  '为成都喊到终场',
  '每一次进攻都有人相信',
  '主场的风吹过球衣',
  '今晚继续并肩向前',
]
const guestChantLoopLines = [...guestChantLines, ...guestChantLines]
const guestChantMeterBars = [18, 34, 24, 48, 30, 62, 38, 74, 44, 58, 32, 46, 26, 36]

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

interface MembershipMeta {
  code: string
  badgeLabel: string
  heroClass: string
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
      levelHint: '邀请码会员',
      levelDescription: '通过邀请码完成首次绑定，当前按 V3 等级展示。',
    }
  }

  if (normalizedTier === 'V2') {
    return {
      code: 'V2',
      badgeLabel: 'V2 进阶会员',
      heroClass: 'account-hero--v2',
      levelHint: '进阶会员',
      levelDescription: '通过推荐好友注册已升级到 V2，回流频率按进阶档位开放。',
    }
  }

  if (Number.isFinite(tierNumber) && tierNumber >= 4) {
    return {
      code: normalizedTier,
      badgeLabel: `${normalizedTier} 推荐会员`,
      heroClass: 'account-hero--v3',
      levelHint: '推荐升级会员',
      levelDescription: `当前已经通过推荐升级到 ${normalizedTier}，回流频率和后续会员权益会按更高档位开放。`,
    }
  }

  return {
    code: normalizedTier,
    badgeLabel: `${normalizedTier} 标准会员`,
    heroClass: 'account-hero--v1',
    levelHint: '标准会员',
    levelDescription: '当前是 V1 基础会员身份。',
  }
}

const currentMembershipMeta = computed(() => resolveMembershipMeta(currentUser.value?.membership_tier))
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
const membershipHeroTitle = computed(() =>
  currentMembershipMeta.value.code === 'V9' ? '至尊会员' : `${currentMembershipMeta.value.code} 会员`,
)
const membershipHeroDescription = computed(() => {
  if (currentMembershipMeta.value.code === 'V9') {
    return '恭喜您，您已经是我们的至尊会员'
  }

  return `恭喜您，您已经是${currentMembershipMeta.value.badgeLabel}`
})
const membershipHeroExpiryLabel = computed(() => {
  const label = membershipExpiresAtLabel.value

  if (label === '长期有效' || label === '已过期') {
    return label
  }

  return `${label.replace(/^有效至\s*/, '')} 到期`
})
const membershipHeroActionText = computed(() => {
  if (!canPurchaseMembership.value) {
    return '已开通'
  }

  return canRenewCurrentV9Membership.value ? '续费' : '升级'
})
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
  notificationEmailForm.value = notificationEmail.value
  notificationEmailSheetVisible.value = true
}

function closeNotificationEmailSheet(): void {
  if (notificationEmailSaving.value) {
    return
  }

  notificationEmailSheetVisible.value = false
}

async function saveNotificationEmail(): Promise<void> {
  const email = notificationEmailForm.value.trim()
  if (!isValidNotificationEmail(email)) {
    uni.showToast({ title: '请填写有效邮箱', icon: 'none' })
    return
  }

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

function handleMembershipCardAction(): void {
  if (canPurchaseMembership.value) {
    navigateToPurchase()
  }
}

function openAiFromBrandNav(): void {
  void openAiChat()
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
      invite_code: miniWechatBindForm.inviteCode.trim() || undefined,
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
.page-root {
  position: relative;
  min-height: 100vh;
  background: #f7f8fa;
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

@keyframes guest-chant-scroll {
  from { transform: translateY(0); }
  to { transform: translateY(-50%); }
}

@keyframes guest-chant-pulse {
  0%, 100% {
    opacity: 0.38;
    transform: scaleY(0.56);
  }
  45% {
    opacity: 0.92;
    transform: scaleY(1);
  }
  72% {
    opacity: 0.56;
    transform: scaleY(0.74);
  }
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
    linear-gradient(180deg, rgba(246, 247, 244, 0.24) 0%, rgba(247, 248, 250, 0.74) 32%, #f7f8fa 58%, #f7f8fa 100%);
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
.guest-chant-wall {
  position: fixed;
  left: 44rpx;
  right: 44rpx;
  bottom: calc(178rpx + env(safe-area-inset-bottom));
  height: 520rpx;
  z-index: 30;
  display: flex;
  flex-direction: column;
  justify-content: flex-end;
  pointer-events: none;
}
.guest-chant-wall__meta {
  margin-bottom: 28rpx;
  text-align: center;
  text-shadow: 0 10rpx 26rpx rgba(0, 0, 0, 0.45);
}
.guest-chant-wall__kicker {
  display: block;
  color: rgba(255, 255, 255, 0.54);
  font-size: 20rpx;
  font-weight: 800;
  letter-spacing: 4rpx;
}
.guest-chant-wall__title {
  display: block;
  margin-top: 10rpx;
  color: rgba(255, 255, 255, 0.94);
  font-size: 42rpx;
  font-weight: 900;
  line-height: 1.15;
}
.guest-chant-wall__lyrics {
  position: relative;
  height: 338rpx;
  overflow: hidden;
}
.guest-chant-wall__track {
  display: flex;
  flex-direction: column;
  align-items: center;
  animation: guest-chant-scroll 20s linear infinite;
}
.guest-chant-wall__line {
  display: block;
  width: 100%;
  height: 68rpx;
  color: rgba(255, 255, 255, 0.42);
  font-size: 31rpx;
  font-weight: 700;
  line-height: 68rpx;
  text-align: center;
  text-shadow: 0 8rpx 24rpx rgba(0, 0, 0, 0.42);
}
.guest-chant-wall__line--focus {
  color: rgba(255, 255, 255, 0.94);
  font-size: 39rpx;
  font-weight: 900;
}
.guest-chant-wall__fade {
  position: absolute;
  left: 0;
  right: 0;
  height: 92rpx;
  z-index: 1;
  pointer-events: none;
}
.guest-chant-wall__fade--top {
  top: 0;
  background: linear-gradient(180deg, rgba(17, 23, 15, 0.84), rgba(17, 23, 15, 0));
}
.guest-chant-wall__fade--bottom {
  bottom: 0;
  background: linear-gradient(0deg, rgba(17, 23, 15, 0.72), rgba(17, 23, 15, 0));
}
.guest-chant-wall__meter {
  height: 88rpx;
  margin: 18rpx auto 0;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  gap: 10rpx;
  opacity: 0.52;
}
.guest-chant-wall__bar {
  width: 8rpx;
  border-radius: 999rpx;
  background: rgba(255, 255, 255, 0.64);
  box-shadow: 0 0 18rpx rgba(255, 255, 255, 0.18);
  transform-origin: bottom center;
  animation: guest-chant-pulse 1.45s ease-in-out infinite;
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
.skeleton-stack {
  position: relative;
  z-index: 1;
  display: grid;
  gap: 16rpx;
}
.skeleton-panel {
  position: relative;
  overflow: hidden;
}
.skeleton-panel::before {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(
    90deg,
    rgba(255, 255, 255, 0) 0%,
    rgba(255, 255, 255, 0.66) 48%,
    rgba(255, 255, 255, 0) 100%
  );
  transform: translateX(-100%);
  animation: skeleton-shimmer 1.35s ease-in-out infinite;
  pointer-events: none;
  z-index: 2;
}
.skeleton-copy-group {
  display: grid;
  gap: 10rpx;
  min-width: 0;
}
.skeleton-line,
.skeleton-pill,
.skeleton-button,
.skeleton-account-cell,
.skeleton-privilege-card {
  background: linear-gradient(180deg, rgba(248, 246, 239, 0.96), rgba(235, 230, 216, 0.9));
  border: 2rpx solid rgba(232, 222, 198, 0.72);
}
.skeleton-line {
  border-radius: 999rpx;
}
.skeleton-line--kicker {
  width: 150rpx;
  height: 24rpx;
}
.skeleton-line--title {
  width: 330rpx;
  max-width: 100%;
  height: 44rpx;
  border-radius: 22rpx;
}
.skeleton-line--section {
  width: 220rpx;
  max-width: 100%;
  height: 38rpx;
  border-radius: 22rpx;
}
.skeleton-line--label {
  width: 112rpx;
  height: 22rpx;
}
.skeleton-line--body {
  width: 90%;
  height: 26rpx;
  margin-top: 20rpx;
}
.skeleton-line--body-short {
  width: 62%;
}
.skeleton-pill {
  width: 150rpx;
  height: 52rpx;
  border-radius: 999rpx;
  flex-shrink: 0;
}
.skeleton-pill--short {
  width: 112rpx;
}
.skeleton-button {
  width: 150rpx;
  height: 60rpx;
  border-radius: 999rpx;
  flex-shrink: 0;
}
.skeleton-button--compact {
  width: 160rpx;
  height: 58rpx;
}
.skeleton-account-hero {
  min-height: 330rpx;
}
.skeleton-profile-row {
  margin-top: 28rpx;
  display: flex;
  align-items: center;
  gap: 22rpx;
}
.skeleton-line--avatar {
  width: 122rpx;
  height: 122rpx;
  border-radius: 999rpx;
  flex-shrink: 0;
}
.skeleton-profile-body {
  flex: 1;
  min-width: 0;
  display: grid;
  gap: 18rpx;
}
.skeleton-line--profile-name {
  width: 260rpx;
  height: 46rpx;
  border-radius: 22rpx;
}
.skeleton-line--profile-badge {
  width: 180rpx;
  height: 46rpx;
  border-radius: 999rpx;
}
.skeleton-line--medal {
  width: 146rpx;
  height: 146rpx;
  border-radius: 42rpx;
  flex-shrink: 0;
  background: linear-gradient(180deg, rgba(246, 229, 180, 0.98), rgba(220, 181, 91, 0.82));
}
.skeleton-invite-row {
  margin-top: 26rpx;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 18rpx;
  padding: 18rpx 20rpx;
  border-radius: 28rpx;
  border: 2rpx solid rgba(232, 212, 166, 0.82);
  background: rgba(255, 252, 244, 0.74);
}
.skeleton-line--invite-code {
  width: 310rpx;
  height: 34rpx;
  border-radius: 18rpx;
}
.skeleton-account-grid {
  margin-top: 24rpx;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  border-radius: 28rpx;
  overflow: hidden;
  border: 2rpx solid rgba(234, 226, 209, 0.92);
}
.skeleton-account-cell {
  min-height: 104rpx;
  padding: 20rpx;
  display: flex;
  align-items: center;
  gap: 16rpx;
  border-width: 0 2rpx 2rpx 0;
  border-radius: 0;
}
.skeleton-account-cell:nth-child(2n) {
  border-right-width: 0;
}
.skeleton-account-cell:nth-last-child(1) {
  grid-column: span 2;
  border-bottom-width: 0;
}
.skeleton-line--account-icon {
  width: 54rpx;
  height: 54rpx;
  border-radius: 999rpx;
  flex-shrink: 0;
}
.skeleton-line--account-value {
  width: 130rpx;
  height: 30rpx;
  border-radius: 18rpx;
}
.skeleton-privilege-grid {
  margin-top: 24rpx;
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12rpx;
}
.skeleton-privilege-card {
  min-height: 150rpx;
  padding: 18rpx 12rpx;
  border-radius: 24rpx;
  display: grid;
  justify-items: center;
  align-content: center;
  gap: 12rpx;
}
.skeleton-line--privilege-icon {
  width: 52rpx;
  height: 52rpx;
  border-radius: 999rpx;
}
.skeleton-line--privilege-title {
  width: 82rpx;
  height: 26rpx;
}
.skeleton-line--privilege-caption {
  width: 98rpx;
  height: 22rpx;
}
@keyframes skeleton-shimmer {
  0% {
    transform: translateX(-100%);
  }
  100% {
    transform: translateX(100%);
  }
}
.account-hero {
  background: #ffffff;
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
  border-radius: 28rpx;
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
  border: 2rpx solid rgba(254, 205, 211, 0.96);
  background: linear-gradient(180deg, rgba(255, 241, 242, 0.98), rgba(254, 226, 226, 0.98));
  color: #991b1b;
  font-size: 24rpx;
  font-weight: 700;
  box-sizing: border-box;
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
  --tier-accent: #dc2626;
  --tier-accent-soft: rgba(254, 226, 226, 0.98);
  --tier-ink: #991b1b;
  --tier-shadow: rgba(220, 38, 38, 0.24);
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
  border: 2rpx solid var(--page-border);
  background: #f6f7fb;
  color: #6d7280;
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
  color: #8f9198;
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
  background: #9aa0aa;
  box-shadow: 0 0 0 6rpx rgba(154, 160, 170, 0.12);
}
.meta-note--hero { padding-top: 14rpx; }
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
.notification-email-panel {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 18rpx;
  border-color: rgba(229, 232, 238, 0.96);
  background: rgba(255, 255, 255, 0.94);
}
.notification-email-panel__body {
  min-width: 0;
  display: grid;
  gap: 10rpx;
}
.notification-email-panel__title {
  color: #15161b;
  font-size: 30rpx;
  font-weight: 800;
}
.notification-email-panel__value {
  color: #747985;
  font-size: 24rpx;
  line-height: 1.4;
  word-break: break-all;
}
.notification-email-panel__action {
  flex-shrink: 0;
  min-width: 128rpx;
  height: 68rpx;
  padding: 0 26rpx;
  border-radius: 999rpx;
  background: #15161b;
  color: #ffffff;
  font-size: 26rpx;
  font-weight: 800;
  line-height: 68rpx;
  box-shadow: none;
}
.notification-email-panel__action::after {
  border: none;
}
.logout-action {
  width: 100%;
  height: 78rpx;
  padding: 0 26rpx;
  border-radius: 999rpx;
  border: 0;
  background: #15161b;
  color: #ffffff;
  font-size: 26rpx;
  font-weight: 800;
  line-height: 78rpx;
  text-align: center;
  box-sizing: border-box;
  box-shadow: none;
}
.logout-action::after {
  border: none;
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
  --tier-pulse: rgba(220, 38, 38, 0.14);
  border-color: rgba(254, 205, 211, 0.95);
  background: linear-gradient(180deg, rgba(255, 241, 242, 0.98), rgba(254, 226, 226, 0.98));
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
  --tier-pulse: rgba(185, 28, 28, 0.16);
  border-color: rgba(252, 165, 165, 0.95);
  background: linear-gradient(180deg, rgba(255, 241, 242, 0.98), rgba(254, 202, 202, 0.72));
}
.tier-ladder-card.tier-ladder-card--active {
  box-shadow: 0 16rpx 44rpx rgba(220, 38, 38, 0.18);
  transform: translateY(-2rpx);
}
.tier-ladder-card.tier-tone--v1.tier-ladder-card--active {
  box-shadow: 0 16rpx 44rpx rgba(127, 144, 170, 0.22);
}
.tier-ladder-card.tier-tone--v2.tier-ladder-card--active {
  box-shadow: 0 16rpx 44rpx rgba(106, 158, 142, 0.22);
}
.tier-ladder-card.tier-tone--v3.tier-ladder-card--active {
  box-shadow: 0 16rpx 44rpx rgba(220, 38, 38, 0.18);
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
  font-size: 22rpx;
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
  border-radius: 999rpx;
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
  color: #171a20;
  font-size: 34rpx;
  font-weight: 900;
  line-height: 1.1;
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
  border-radius: 999rpx;
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
  border-radius: 24rpx;
  border: 0;
  background: rgba(246, 247, 249, 0.92);
  backdrop-filter: none;
}

.profile-banner__invite-label {
  color: #8b92a0;
  font-size: 22rpx;
}

.profile-banner__invite-code {
  font-size: 26rpx;
}

.profile-banner__invite-copy {
  min-width: 148rpx;
  height: 62rpx;
  line-height: 62rpx;
  border: none;
  background: #15171d;
  color: #ffffff;
  box-shadow: none;
}

.info-panel,
.privilege-panel,
.upgrade-panel {
  padding: 22rpx;
  border-radius: 28rpx;
}
.info-panel,
.privilege-panel {
  border-color: rgba(229, 232, 238, 0.96);
  background: rgba(255, 255, 255, 0.94);
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
  font-size: 30rpx;
  font-weight: 800;
  line-height: 1;
}

.account-info-grid {
  margin-top: 22rpx;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12rpx;
}

.account-info-item {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 14rpx;
  padding: 18rpx;
  box-sizing: border-box;
  border-radius: 22rpx;
  border: 1rpx solid rgba(232, 235, 241, 0.98);
  background: rgba(248, 249, 251, 0.78);
}

.account-info-item__icon {
  width: 38rpx;
  height: 38rpx;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.account-info-item__icon-image {
  width: 36rpx;
  height: 36rpx;
}

.account-info-item__body {
  min-width: 0;
  flex: 1;
  display: grid;
  gap: 10rpx;
}

.account-info-item__label {
  color: #969ca8;
  font-size: 22rpx;
  line-height: 1;
}

.account-info-item__value {
  color: #17181c;
  font-size: 25rpx;
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

.privilege-grid {
  margin-top: 22rpx;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12rpx;
}

.privilege-card {
  min-width: 0;
  padding: 18rpx;
  border-radius: 22rpx;
  border: 1rpx solid rgba(232, 235, 241, 0.98);
  background: rgba(248, 249, 251, 0.78);
  display: flex;
  align-items: center;
  gap: 14rpx;
  text-align: left;
}

.privilege-card__icon {
  grid-area: icon;
  width: 38rpx;
  height: 38rpx;
  display: flex;
  align-items: center;
  justify-content: center;
}

.privilege-card__icon-image {
  width: 36rpx;
  height: 36rpx;
}

.privilege-card__body {
  min-width: 0;
  flex: 1;
  display: grid;
  gap: 10rpx;
}

.privilege-card__title {
  display: block;
  color: #1e2025;
  font-size: 25rpx;
  font-weight: 800;
  line-height: 1.15;
}

.privilege-card__caption {
  display: block;
  color: #8a8f9b;
  font-size: 22rpx;
  line-height: 1;
}

.upgrade-panel {
  border-color: rgba(229, 232, 238, 0.96);
  background: rgba(255, 255, 255, 0.94);
}

.upgrade-panel__title-row {
  display: flex;
  align-items: center;
  gap: 12rpx;
}

.upgrade-panel__info-icon {
  width: 36rpx;
  height: 36rpx;
  flex-shrink: 0;
}

.upgrade-panel__body {
  display: block;
  margin-top: 16rpx;
  color: #737986;
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
  border: 0 !important;
  box-shadow: none;
  transform: none;
}
.tier-ladder-card.tier-ladder-card--compact.tier-ladder-card--active {
  box-shadow: none;
}
.tier-ladder-card.tier-tone--v1.tier-ladder-card--compact {
  background: #f3f5f8 !important;
}
.tier-ladder-card.tier-tone--v2.tier-ladder-card--compact {
  background: #edf2f3 !important;
}
.tier-ladder-card.tier-tone--v3.tier-ladder-card--compact {
  background: #f1eadc !important;
}
.tier-ladder-card.tier-tone--v4.tier-ladder-card--compact {
  background: #eadfd9 !important;
}
.tier-ladder-card.tier-tone--v5.tier-ladder-card--compact {
  background: #ded8e8 !important;
}
.tier-ladder-card.tier-tone--v6.tier-ladder-card--compact {
  background: #d5e2da !important;
}
.tier-ladder-card.tier-tone--v7.tier-ladder-card--compact {
  background: #cad8e9 !important;
}
.tier-ladder-card.tier-tone--v8.tier-ladder-card--compact {
  background: #c95656 !important;
}
.tier-ladder-card.tier-tone--v9.tier-ladder-card--compact {
  background: #9e2f2f !important;
}
.tier-ladder-card.tier-tone--v1.tier-ladder-card--compact.tier-ladder-card--active {
  background: #e6ebf1 !important;
  box-shadow: none;
}
.tier-ladder-card.tier-tone--v2.tier-ladder-card--compact.tier-ladder-card--active {
  background: #dde8e6 !important;
  box-shadow: none;
}
.tier-ladder-card.tier-tone--v3.tier-ladder-card--compact.tier-ladder-card--active {
  background: #e6dcc7 !important;
  box-shadow: none;
}
.tier-ladder-card.tier-tone--v4.tier-ladder-card--compact.tier-ladder-card--active {
  background: #dfcfc7 !important;
  box-shadow: none;
}
.tier-ladder-card.tier-tone--v5.tier-ladder-card--compact.tier-ladder-card--active {
  background: #d0c8df !important;
  box-shadow: none;
}
.tier-ladder-card.tier-tone--v6.tier-ladder-card--compact.tier-ladder-card--active {
  background: #c6d8cc !important;
  box-shadow: none;
}
.tier-ladder-card.tier-tone--v7.tier-ladder-card--compact.tier-ladder-card--active {
  background: #b8cae1 !important;
  box-shadow: none;
}
.tier-ladder-card.tier-tone--v8.tier-ladder-card--compact.tier-ladder-card--active {
  background: #b94141 !important;
  box-shadow: none;
}
.tier-ladder-card.tier-tone--v9.tier-ladder-card--compact.tier-ladder-card--active {
  background: #842525 !important;
  box-shadow: none !important;
}

.tier-ladder-card.tier-tone--v8.tier-ladder-card--compact .tier-ladder-card__code,
.tier-ladder-card.tier-tone--v8.tier-ladder-card--compact .tier-ladder-card__name,
.tier-ladder-card.tier-tone--v8.tier-ladder-card--compact .tier-ladder-card__condition,
.tier-ladder-card.tier-tone--v8.tier-ladder-card--compact .tier-ladder-card__refresh,
.tier-ladder-card.tier-tone--v9.tier-ladder-card--compact .tier-ladder-card__code,
.tier-ladder-card.tier-tone--v9.tier-ladder-card--compact .tier-ladder-card__name,
.tier-ladder-card.tier-tone--v9.tier-ladder-card--compact .tier-ladder-card__condition,
.tier-ladder-card.tier-tone--v9.tier-ladder-card--compact .tier-ladder-card__refresh {
  color: rgba(255, 255, 255, 0.94);
}

.tier-ladder-card.tier-tone--v8.tier-ladder-card--compact .tier-ladder-card__refresh,
.tier-ladder-card.tier-tone--v9.tier-ladder-card--compact .tier-ladder-card__refresh {
  background: rgba(255, 255, 255, 0.94);
  border-color: transparent;
  color: #8f2525;
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
  background: #15171d;
  border: 0;
  color: #ffffff;
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
