<template>
  <page-meta :page-style="navigationPageStyle" />
  <FiSecondaryNav @back="goBack" @share="handleH5Share" />
  <view class="director-page">
    <FiPageBackdrop :image-url="pageBackgroundImageUrl" full-page />
    <view class="director-content">
      <FiLoading v-if="loading && !detail" title="正在加载建队方案" caption="读取 5 外援名单与好友圈共识…" />

      <view v-else-if="error && !detail" class="director-error">
        <text class="director-error__text">{{ error }}</text>
        <button class="director-btn director-btn--primary" hover-class="director-btn--pressed" @tap="viewModel.loadDetail()">
          重试
        </button>
      </view>

      <template v-for="block in visibleBlocks" :key="block.key">
        <FiLayoutBlock v-if="block.key === 'header'" :background-image-url="block.background_image_url">
          <view class="director-header">
            <view class="director-header__row">
              <text class="director-header__title">总监建队方案</text>
              <text v-if="detail" class="director-header__window">{{ detail.plan.window_name }}</text>
            </view>
          </view>
        </FiLayoutBlock>

        <FiLayoutBlock
          v-else-if="block.key === 'comparison' && detail?.comparison"
          :background-image-url="block.background_image_url"
        >
          <DirectorComparisonCard
            :comparison="detail.comparison"
            :target-author-name="detail.plan.author.nickname"
          />
        </FiLayoutBlock>

        <FiLayoutBlock
          v-else-if="block.key === 'view_tabs' && detail"
          :background-image-url="block.background_image_url"
        >
          <FiSegmented
            v-model="activeTab"
            :items="tabItems"
            aria-label="方案视图切换"
          />
        </FiLayoutBlock>

        <FiLayoutBlock
          v-else-if="block.key === 'plan_overview' && detail && activeTab === 'plan'"
          :background-image-url="block.background_image_url"
        >
          <DirectorPlanOverview :plan="detail.plan" />
        </FiLayoutBlock>

        <FiLayoutBlock
          v-else-if="block.key === 'comments' && detail && activeTab === 'plan'"
          :background-image-url="block.background_image_url"
        >
          <DirectorCommentsSection
            :likes-count="detail.plan.likes_count"
            :dislikes-count="detail.plan.dislikes_count"
            :my-reaction="detail.plan.my_reaction"
            :reaction-pending="reactionPending"
            :comments-count="detail.plan.comments_count"
            :comments="detail.comments"
            :submitting="commentSubmitting"
            @react="viewModel.toggleReaction"
            @submit-comment="handleCommentSubmit"
          />
        </FiLayoutBlock>

        <FiLayoutBlock
          v-else-if="block.key === 'circle_panel' && detail && activeTab === 'circle'"
          :background-image-url="block.background_image_url"
        >
          <DirectorCirclePanel
            :circle="circle"
            :loading="circleLoading"
            :error="circleError"
            :current-share-code="detail.plan.share_code"
            @retry="viewModel.loadCircle(detail.plan.circle_code)"
            @share="handleH5Share"
            @open-plan="openOtherPlan"
          />
        </FiLayoutBlock>
      </template>
    </view>

    <view v-if="detail" class="director-dock">
      <view class="director-dock__actions">
        <template v-if="detail.plan.is_mine">
          <button
            class="director-btn director-dock__btn"
            hover-class="director-btn--pressed"
            @tap="openEditOwn"
          >
            修改方案
          </button>
          <!-- #ifdef MP-WEIXIN -->
          <button
            class="director-btn director-btn--primary director-dock__btn"
            open-type="share"
            hover-class="director-btn--pressed"
          >
            发微信群 PK
          </button>
          <!-- #endif -->
          <!-- #ifndef MP-WEIXIN -->
          <button
            class="director-btn director-btn--primary director-dock__btn"
            hover-class="director-btn--pressed"
            @tap="handleH5Share"
          >
            发微信群 PK
          </button>
          <!-- #endif -->
        </template>

        <template v-else>
          <button
            class="director-btn director-btn--primary director-dock__btn"
            hover-class="director-btn--pressed"
            @tap="openJoinPk"
          >
            我也来当总监
          </button>
          <!-- #ifdef MP-WEIXIN -->
          <button
            class="director-btn director-btn--dark director-dock__btn"
            open-type="share"
            hover-class="director-btn--pressed"
          >
            转发给群友
          </button>
          <!-- #endif -->
          <!-- #ifndef MP-WEIXIN -->
          <button
            class="director-btn director-btn--dark director-dock__btn"
            hover-class="director-btn--pressed"
            @tap="handleH5Share"
          >
            转发给群友
          </button>
          <!-- #endif -->
        </template>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from 'vue'
import { onLoad, onShareAppMessage, onShow } from '@dcloudio/uni-app'
import FiLayoutBlock from '../../components/FiLayoutBlock.vue'
import FiLoading from '../../components/FiLoading.vue'
import FiPageBackdrop from '../../components/FiPageBackdrop.vue'
import FiSecondaryNav from '../../components/FiSecondaryNav.vue'
import FiSegmented from '../../components/FiSegmented.vue'
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import { usePageLayout } from '../../composables/usePageLayout'
import { useAppSession } from '../../stores/appSession'
import type { FiSegmentedItem } from '../../types/segmented'
import {
  buildDirectorEditUrl,
  buildDirectorViewUrl,
  directorPlanShareTitle,
  parseDirectorViewCode,
} from '../../utils/directorLinks'
import { buildSharePath } from '../../utils/shareSource'
import DirectorCirclePanel from './components/DirectorCirclePanel.vue'
import DirectorCommentsSection from './components/DirectorCommentsSection.vue'
import DirectorComparisonCard from './components/DirectorComparisonCard.vue'
import DirectorPlanOverview from './components/DirectorPlanOverview.vue'
import { buildDirectorShareText } from './helpers'
import { useDirectorView } from './useDirectorView'

const { navigationPageStyle } = useNavigationLayout()
const {
  pageBackgroundImageUrl,
  visibleBlocks,
  refreshLayout,
} = usePageLayout('director_view')
const session = useAppSession()
const viewModel = useDirectorView(session)
const {
  shareCode,
  detail,
  loading,
  error,
  circle,
  circleLoading,
  circleError,
  reactionPending,
  commentSubmitting,
} = viewModel

const activeTab = ref<'plan' | 'circle'>('plan')

const tabItems = computed<FiSegmentedItem[]>(() => [
  { key: 'plan', label: '方案详情' },
  {
    key: 'circle',
    label: circle.value ? `好友圈共识（${circle.value.member_count}）` : '好友圈共识',
  },
])

let loadedRevision = -1

onLoad((query) => {
  const parsed = parseDirectorViewCode(query as Record<string, unknown>)
  shareCode.value = parsed
})

onShow(() => {
  void refreshLayout()
  if (loadedRevision !== session.revision.value) {
    loadedRevision = session.revision.value
    void viewModel.loadDetail(shareCode.value)
  }
})

onBeforeUnmount(() => {
  viewModel.dispose()
})

async function handleCommentSubmit(text: string, onSuccess: () => void): Promise<void> {
  const ok = await viewModel.submitComment(text)
  if (ok) onSuccess()
}

function openEditOwn(): void {
  if (!detail.value) return
  uni.navigateTo({
    url: buildDirectorEditUrl({
      teamId: detail.value.plan.team_id,
      circleCode: detail.value.plan.circle_code,
    }),
  })
}

function openJoinPk(): void {
  if (!detail.value) return
  uni.navigateTo({
    url: buildDirectorEditUrl({
      teamId: detail.value.plan.team_id,
      circleCode: detail.value.plan.circle_code,
      fromCode: detail.value.plan.share_code,
    }),
  })
}

function openOtherPlan(code: string): void {
  if (!code || code === shareCode.value) return
  const url = buildDirectorViewUrl(code)
  if (!url) return
  uni.navigateTo({ url })
}

function handleH5Share(): void {
  const text = detail.value ? buildDirectorShareText(detail.value.plan) : '我来当总监：休赛期建队 PK'
  uni.setClipboardData({
    data: text,
    success: () => {
      uni.showToast({ title: '已复制分享文案，快去发群里PK吧', icon: 'none' })
    },
    fail: () => {
      uni.showToast({ title: '请在右上角菜单转发给好友', icon: 'none' })
    },
  })
}

function goBack(): void {
  uni.navigateBack({
    fail: () => {
      uni.switchTab({ url: '/pages/insights/index' })
    },
  })
}

onShareAppMessage(() => ({
  title: detail.value ? directorPlanShareTitle(detail.value.plan) : '我来当总监 · 休赛期建队方案',
  path: buildSharePath(buildDirectorViewUrl(shareCode.value), 'director'),
}))
</script>

<style scoped src="./director.css"></style>
