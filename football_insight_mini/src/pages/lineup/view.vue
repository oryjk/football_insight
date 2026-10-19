<template>
  <page-meta :page-style="navigationPageStyle + (shareVisible ? 'overflow: hidden;' : '')" />
  <FiSecondaryNav :shareable="false" @back="goBack" />
  <view class="community-page">
    <!-- #ifdef H5 -->
    <text class="community-caption">请在微信小程序中查看分享阵容。</text>
    <!-- #endif -->
    <!-- #ifndef H5 -->
    <FiLoading v-if="loading" title="阵容加载中" caption="正在读取球迷分享的完整阵容。" />
    <text v-if="error" class="community-error">{{ error }}</text>
    <button v-if="!loading && !detail" class="community-action" @tap="reload">重新加载</button>
    <template v-if="detail">
      <view class="community-panel">
        <text class="community-title">{{ detail.team.team_name }} · {{ detail.formation }}</text>
        <view class="community-row">
          <image v-if="detail.author.avatar_url" :src="detail.author.avatar_url" class="author-avatar" mode="aspectFill" />
          <text class="community-caption">{{ detail.author.display_name }}心中的最佳阵容</text>
        </view>
        <text class="community-caption">发布 {{ lineupDateLabel(detail.created_at) }} · 更新 {{ lineupDateLabel(detail.updated_at) }}</text>
      </view>
      <LineupPitch :players="pitchPlayers" readonly />
      <view class="community-row">
        <button class="community-action" :class="{ 'community-action--active': detail.liked_by_me }" :disabled="pendingLikes.length > 0" @tap="like(returnUrl)">{{ detail.liked_by_me ? '♥ 已赞' : '♡ 点赞' }} {{ detail.likes_count }}</button>
        <button class="community-action" @tap="openShare">分享阵容</button>
      </view>
      <view class="community-row">
        <button class="community-action community-action--active" @tap="edit">我也排一套</button>
        <button class="community-action" @tap="list">球队战术板</button>
      </view>
    </template>
    <LineupShareSheet v-model:visible="shareVisible" :busy="shareBusy" :ready="Boolean(sharePayload)" :image-path="imagePath" :error="shareError" @generate="generate" @preview="preview" @save="saveImage" />
    <canvas canvas-id="lineup-share-canvas" class="lineup-canvas" :style="{ width: '750px', height: '1200px' }" />
    <canvas canvas-id="lineup-card-canvas" class="lineup-canvas" :style="{ width: '750px', height: '600px' }" />
    <!-- #endif -->
  </view>
</template>
<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { onLoad, onShareAppMessage, onShow } from '@dcloudio/uni-app'
import FiSecondaryNav from '../../components/FiSecondaryNav.vue'
import FiLoading from '../../components/FiLoading.vue'
import LineupPitch from './components/LineupPitch.vue'
import LineupShareSheet from './components/LineupShareSheet.vue'
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import { useAppSession } from '../../stores/appSession'
import { buildLineupListUrl, buildLineupPageUrl, buildLineupViewUrl, parseLineupShareCode } from '../../utils/lineupLinks'
import { createLineupDetail } from './lineupCommunity'
import { publicPitch, lineupSharePayload } from './lineupPoster'
import { lineupDateLabel } from './lineupPresentation'
import { useLineupShare } from './useLineupShare'
const { navigationPageStyle } = useNavigationLayout()
const session = useAppSession()
const state = createLineupDetail()
const { detail, loading, error, pendingLikes, like } = state
const code = ref('')
const returnUrl = computed(() => buildLineupViewUrl(code.value))
const share = useLineupShare(() => JSON.stringify([code.value, detail.value?.updated_at, session.revision.value]), () => returnUrl.value)
const { visible: shareVisible, busy: shareBusy, error: shareError, imagePath, payload: sharePayload, generate, preview, save: saveImage } = share
const pitchPlayers = computed(() => detail.value ? publicPitch(detail.value) : [])
function goBack(): void { uni.navigateBack({ fail: () => uni.switchTab({ url: '/pages/insights/index' }) }) }
function edit(): void { if (detail.value) uni.navigateTo({ url: buildLineupPageUrl(detail.value.team.team_id) }) }
function list(): void { if (detail.value) uni.navigateTo({ url: buildLineupListUrl(detail.value.team.team_id) }) }
function openShare(): void { if (detail.value) void share.open(detail.value) }
function reload(): void { if (code.value) void state.load(code.value); else error.value = '分享链接无效' }
onLoad(query => { code.value = parseLineupShareCode(query as Record<string, unknown>) })
let started = false, loadedSessionRevision = -1
onShow(() => {
  if (!started || loadedSessionRevision !== session.revision.value) {
    started = true; loadedSessionRevision = session.revision.value; reload()
  }
})
// Menu sharing is available only after a valid public detail has loaded.
watch(detail, value => {
  // #ifdef MP-WEIXIN
  if (value) uni.showShareMenu({ menus: ['shareAppMessage'] }); else uni.hideShareMenu({ hideShareItems: ['shareAppMessage', 'shareTimeline'] })
  // #endif
})
onShareAppMessage(() => sharePayload.value ?? (detail.value ? lineupSharePayload(detail.value) : {}))
onBeforeUnmount(state.dispose)
</script>
<style scoped src="./community.css"></style>

<style scoped>
.author-avatar { width: var(--fi-space-48); height: var(--fi-space-48); border-radius: var(--fi-radius-round); }
</style>
