<template>
  <page-meta :page-style="navigationPageStyle" />
  <FiSecondaryNav @back="goBackToInsights" @share="copyPageShareLink(teamId ? `/pages/team-board/index?teamId=${teamId}` : '/pages/insights/index')" />
  <view class="page-root">
    <image class="page-bg-img" :src="phoenixStadiumBgImage" mode="aspectFill" :webp="true" />
    <view class="page-bg-fade"></view>
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
            <TeamBoardHero :team="board.team" @back="goBackToInsights" />
            <TeamBoardLauncher
              :presets="composerPresets"
              :post-count="posts.length"
              @open-composer="openComposer"
            />

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
import { onLoad, onShareAppMessage, onShow } from '@dcloudio/uni-app'
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import FiLoading from '../../components/FiLoading.vue'
import FiSecondaryNav from '../../components/FiSecondaryNav.vue'
import { copyPageShareLink } from '../../utils/secondaryNavigation'
import { PHOENIX_STADIUM_BG_IMAGE_URL as phoenixStadiumBgImage } from '../../config/assets'
import TeamBoardHero from './components/TeamBoardHero.vue'
import TeamBoardLauncher from './components/TeamBoardLauncher.vue'
import TeamBoardPostCard from './components/TeamBoardPostCard.vue'
import TeamBoardComposer from './components/TeamBoardComposer.vue'
import { useTeamBoardPage } from './useTeamBoardPage'

const { navigationPageStyle } = useNavigationLayout()
const {
  teamId, loading, errorMessage, board, posts, composerPresets,
  composerOpen, selectedPreset, postTitle, postCommentary, composerErrorMessage, commentDrafts,
  loadBoard, openComposer, closeComposer, publishPost, submitComment, toggleLike,
} = useTeamBoardPage()

onShareAppMessage(() => ({
  title: board.value
    ? `${board.value.team.team_name}战术板，来看看这支队的数据观点`
    : '球队战术板：把洞察卡变成观点',
  path: teamId.value ? `/pages/team-board/index?teamId=${teamId.value}` : '/pages/insights/index',
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
  background: linear-gradient(180deg, rgba(246, 247, 244, 0.24) 0%, rgba(247, 248, 250, 0.74) 32%, #f7f8fa 58%, #f7f8fa 100%);
  pointer-events: none;
  z-index: 0;
}
.page-scroll { height: 100vh; padding-top: var(--fi-brand-nav-height); }
.page { position: relative; z-index: 1; padding: var(--fi-space-28) var(--fi-space-24) var(--fi-space-40); }
.state-card--error text { font-size: 28rpx; color: #c03a2b; }
.state-card--empty text { color: #767a84; font-size: 26rpx; }
.page-content { display: flex; flex-direction: column; gap: var(--fi-space-card-gap); }
</style>
