<template>
  <page-meta :page-style="navigationPageStyle" />
  <FiSecondaryNav @back="goBackToInsights" @share="copyPageShareLink(teamId ? `/pages/team-board/index?teamId=${teamId}` : '/pages/insights/index')" />
  <view class="page-root">
    <FiPageBackdrop :image-url="pageBackgroundImageUrl" :fallback-image-url="phoenixStadiumBgImage" full-page />
    <scroll-view scroll-y class="page-scroll">
      <view class="page">
        <view class="page-content">
          <FiLoading
            v-if="loading"
            title="战术板加载中"
            caption="正在整理这个球队的观点帖和洞察快照。"
          />

          <view v-else-if="errorMessage" class="state-card state-card--error">
            <text>{{ errorMessage }}</text>
          </view>

          <template v-else-if="board">
            <template v-for="block in visibleBlocks" :key="block.key">
              <FiLayoutBlock v-if="block.key === 'hero'" :background-image-url="block.background_image_url">
                <TeamBoardHero :team="board.team" @back="goBackToInsights" />
              </FiLayoutBlock>

              <FiLayoutBlock v-else-if="block.key === 'launcher'" :background-image-url="block.background_image_url">
                <TeamBoardLauncher
                  :presets="composerPresets"
                  :post-count="posts.length"
                  @open-composer="openComposer"
                />
              </FiLayoutBlock>

              <FiLayoutBlock v-else-if="block.key === 'posts'" :background-image-url="block.background_image_url">
                <view class="posts-stack">
                  <view v-if="!posts.length" class="panel state-card state-card--empty">
                    <text>这个球队还没有观点帖。先从一张洞察卡开始，发布第一条数据观点。</text>
                  </view>

                  <TeamBoardPostCard
                    v-for="post in posts"
                    :key="post.post_id"
                    :post="post"
                    v-model:comment-draft="commentDrafts[post.post_id]"
                    @toggle-like="toggleLike"
                    @submit-comment="submitComment"
                  />
                </view>
              </FiLayoutBlock>
            </template>
          </template>
        </view>

        <TeamBoardComposer
          v-if="composerOpen && selectedPreset"
          :selected-preset="selectedPreset"
          v-model:post-title="postTitle"
          v-model:post-commentary="postCommentary"
          :composer-error-message="composerErrorMessage"
          @close="closeComposer"
          @publish="publishPost"
        />
      </view>
    </scroll-view>
  </view>
</template>

<script setup lang="ts">
import { buildSharePath } from '../../utils/shareSource'
import { onLoad, onShareAppMessage, onShow } from '@dcloudio/uni-app'
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import { usePageLayout } from '../../composables/usePageLayout'
import FiLayoutBlock from '../../components/FiLayoutBlock.vue'
import FiLoading from '../../components/FiLoading.vue'
import FiPageBackdrop from '../../components/FiPageBackdrop.vue'
import FiSecondaryNav from '../../components/FiSecondaryNav.vue'
import { copyPageShareLink } from '../../utils/secondaryNavigation'
import { PHOENIX_STADIUM_BG_IMAGE_URL as phoenixStadiumBgImage } from '../../config/assets'
import TeamBoardHero from './components/TeamBoardHero.vue'
import TeamBoardLauncher from './components/TeamBoardLauncher.vue'
import TeamBoardPostCard from './components/TeamBoardPostCard.vue'
import TeamBoardComposer from './components/TeamBoardComposer.vue'
import { useTeamBoardPage } from './useTeamBoardPage'

const { navigationPageStyle } = useNavigationLayout()
const { visibleBlocks, pageBackgroundImageUrl, refreshLayout } = usePageLayout('team_board')
const {
  teamId, loading, errorMessage, board, posts, composerPresets,
  composerOpen, selectedPreset, postTitle, postCommentary, composerErrorMessage, commentDrafts,
  loadBoard, openComposer, closeComposer, publishPost, submitComment, toggleLike,
} = useTeamBoardPage()

onShareAppMessage(() => ({
  title: board.value
    ? `${board.value.team.team_name}战术板，来看看这支队的数据观点`
    : '球队战术板：把洞察卡变成观点',
  path: buildSharePath(teamId.value ? `/pages/team-board/index?teamId=${teamId.value}` : '/pages/insights/index', 'team_board'),
}))

onLoad((query) => {
  const raw = query?.teamId
  teamId.value = typeof raw === 'string' ? Number(raw) : null
})

function goBackToInsights(): void {
  uni.navigateBack({
    fail: () => {
      uni.switchTab({ url: '/pages/insights/index' })
    },
  })
}

onShow(() => {
  void refreshLayout()
  void loadBoard()
})
</script>

<style scoped lang="css">
@import './team-board-shared.css';

.page-root {
  position: relative;
  min-height: 100vh;
  background: #f7f8fa;
}
.page-scroll { position: relative; z-index: 1; height: 100vh; padding-top: var(--fi-brand-nav-height); }
.page { position: relative; z-index: 1; padding: var(--fi-space-28) var(--fi-space-24) var(--fi-space-40); }
.state-card--error text { font-size: var(--fi-font-28); color: #c03a2b; }
.state-card--empty text { color: #767a84; font-size: var(--fi-font-26); }
.page-content, .posts-stack { display: flex; flex-direction: column; gap: var(--fi-space-card-gap); }
</style>
