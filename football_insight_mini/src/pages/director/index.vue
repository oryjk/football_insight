<template>
  <page-meta :page-style="pageStyle" />
  <FiSecondaryNav @back="goBack" @share="handleH5Share" />
  <view class="director-page">
    <FiPageBackdrop :image-url="pageBackgroundImageUrl" full-page />
    <view class="director-content">
      <view v-if="isPkMode" class="director-banner">
        <text class="director-banner__text">正与好友同局 PK，选完立看 1v1 分歧报告</text>
      </view>

      <FiLoading v-if="loading && !context" title="正在加载球队阵容" caption="准备现役外援与引援候选名单…" />

      <view v-else-if="error && !context" class="director-error">
        <text class="director-error__text">{{ error }}</text>
        <button class="director-btn director-btn--primary" hover-class="director-btn--pressed" @tap="draft.loadContext()">
          重新加载
        </button>
      </view>

      <template v-for="block in visibleBlocks" :key="block.key">
        <FiLayoutBlock v-if="block.key === 'header'" :background-image-url="block.background_image_url">
          <view class="director-header">
            <view class="director-header__row">
              <text class="director-header__title">我来当总监</text>
              <text v-if="context" class="director-header__window">{{ context.window.name }}</text>
            </view>
            <text class="director-header__sub">
              {{ context ? `${context.team.team_name}已有 ${context.team_plan_count} 份建队方案，1 分钟定出你的 5 外援` : '休赛期建队模拟 · 定出你的 5 外援并拉微信群友 PK' }}
            </text>
          </view>
        </FiLayoutBlock>

        <FiLayoutBlock
          v-else-if="block.key === 'team_selector' && teams.length"
          :background-image-url="block.background_image_url"
        >
          <FiTeamSelector
            :teams="teams"
            :model-value="activeTeamId"
            @update:model-value="handleSelectTeam"
          />
        </FiLayoutBlock>

        <FiLayoutBlock
          v-else-if="block.key === 'foreign_section' && context"
          :background-image-url="block.background_image_url"
        >
          <DirectorForeignSection
            :squad-foreign="context.squad_foreign"
            :foreign-stay-ids="foreignStayIds"
            :foreign-leave-ids="foreignLeaveIds"
            :foreign-ins="foreignIns"
            :quota-check="quotaCheck"
            @set-action="draft.setForeignAction"
            @remove-in="draft.removeForeignIn"
            @open-picker="draft.openCandidateSheet('foreign')"
          />
        </FiLayoutBlock>

        <FiLayoutBlock
          v-else-if="block.key === 'domestic_section' && context"
          :background-image-url="block.background_image_url"
        >
          <DirectorDomesticSection
            :squad-domestic="context.squad_domestic"
            :domestic-ins="domesticIns"
            :domestic-outs="domesticOuts"
            :max-domestic-ins="maxDomesticIns"
            :max-domestic-outs="maxDomesticOuts"
            :manifesto="manifesto"
            @open-domestic-picker="draft.openCandidateSheet('domestic')"
            @remove-domestic-in="draft.removeDomesticIn"
            @toggle-domestic-out="draft.toggleDomesticOut"
            @update:manifesto="manifesto = $event"
          />
        </FiLayoutBlock>
      </template>
    </view>

    <DirectorCandidateSheet
      v-model:visible="sheetVisible"
      :mode="sheetMode"
      :league-filter="leagueFilter"
      :search-query="searchQuery"
      :candidates="filteredCandidates"
      :loading="candidatesLoading"
      :error="candidatesError"
      :selected-keys="sheetMode === 'foreign' ? selectedForeignKeys : selectedDomesticKeys"
      :quota-reached="sheetQuotaReached"
      :quota-hint="sheetQuotaHint"
      @update:league-filter="draft.setLeagueFilter"
      @update:search-query="searchQuery = $event"
      @search="draft.fetchCandidates"
      @pick="draft.pickCandidate"
      @add-custom="draft.addCustomCandidate"
    />

    <view v-if="context" class="director-dock">
      <view class="director-dock__meta">
        <text class="director-dock__label">预计建队风格</text>
        <text class="director-dock__style">{{ previewStyleTag }}</text>
      </view>
      <button
        class="director-btn director-btn--primary"
        :class="{ 'director-btn--disabled': saving || !quotaCheck.valid }"
        hover-class="director-btn--pressed"
        @tap="handleSubmit"
      >
        {{ saving ? '保存中…' : '保存并生成方案' }}
      </button>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, watch } from 'vue'
import { onLoad, onShareAppMessage, onShow } from '@dcloudio/uni-app'
import FiLayoutBlock from '../../components/FiLayoutBlock.vue'
import FiLoading from '../../components/FiLoading.vue'
import FiPageBackdrop from '../../components/FiPageBackdrop.vue'
import FiSecondaryNav from '../../components/FiSecondaryNav.vue'
import FiTeamSelector from '../../components/FiTeamSelector.vue'
import { useInsightsTeams } from '../../composables/useInsightsTeams'
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import { usePageLayout } from '../../composables/usePageLayout'
import { useAppSession } from '../../stores/appSession'
import {
  buildDirectorEditUrl,
  buildDirectorViewUrl,
  directorEditShareTitle,
  parseDirectorEditQuery,
} from '../../utils/directorLinks'
import { buildSharePath } from '../../utils/shareSource'
import DirectorCandidateSheet from './components/DirectorCandidateSheet.vue'
import DirectorDomesticSection from './components/DirectorDomesticSection.vue'
import DirectorForeignSection from './components/DirectorForeignSection.vue'
import { useDirectorDraft } from './useDirectorDraft'

const { navigationPageStyle } = useNavigationLayout()
const {
  pageBackgroundImageUrl,
  visibleBlocks,
  refreshLayout,
} = usePageLayout('director')
const session = useAppSession()
const teamBrowser = useInsightsTeams(session)
const { teams } = teamBrowser

const draft = useDirectorDraft(session)
const {
  routeCtx,
  activeTeamId,
  context,
  loading,
  error,
  saving,
  foreignStayIds,
  foreignLeaveIds,
  foreignIns,
  domesticIns,
  domesticOuts,
  manifesto,
  maxDomesticIns,
  maxDomesticOuts,
  quotaCheck,
  previewStyleTag,
  sheetVisible,
  sheetMode,
  leagueFilter,
  searchQuery,
  filteredCandidates,
  candidatesLoading,
  candidatesError,
  selectedForeignKeys,
  selectedDomesticKeys,
  sheetQuotaReached,
  sheetQuotaHint,
} = draft

const pageStyle = computed(() =>
  navigationPageStyle.value + (sheetVisible.value ? 'overflow: hidden;' : ''),
)

const isPkMode = computed(() => Boolean(routeCtx.value.fromCode || routeCtx.value.circleCode))

let initialLoaded = false
let loadedRevision = -1

onLoad((query) => {
  const parsed = parseDirectorEditQuery(query as Record<string, unknown>)
  draft.setRouteContext(parsed)
  void teamBrowser.load().then(() => {
    if (!parsed.teamId && teamBrowser.selectedTeamId.value && !initialLoaded) {
      activeTeamId.value = teamBrowser.selectedTeamId.value
    }
    if (!initialLoaded) {
      initialLoaded = true
      loadedRevision = session.revision.value
      void draft.loadContext(activeTeamId.value)
    }
  })
})

onShow(() => {
  void refreshLayout()
  if (initialLoaded && loadedRevision !== session.revision.value) {
    loadedRevision = session.revision.value
    void draft.loadContext(activeTeamId.value)
  }
})

watch(
  () => teamBrowser.selectedTeamId.value,
  (teamId) => {
    if (!initialLoaded && !routeCtx.value.teamId && teamId) {
      activeTeamId.value = teamId
    }
  },
)

onBeforeUnmount(() => {
  draft.dispose()
  teamBrowser.dispose()
})

function handleSelectTeam(teamId: number): void {
  if (teamId === activeTeamId.value) return
  teamBrowser.selectTeam(teamId)
  // 切换球队时脱离原球队的好友 PK 链，避免跨队错配 circle_code
  if (routeCtx.value.teamId && routeCtx.value.teamId !== teamId) {
    draft.setRouteContext({ teamId, circleCode: '', fromCode: '' })
  }
  void draft.loadContext(teamId)
}

async function handleSubmit(): Promise<void> {
  const result = await draft.submitPlan()
  if (!result) return
  const viewUrl = buildDirectorViewUrl(result.plan.share_code)
  if (!viewUrl) return
  uni.showToast({ title: '方案已生成', icon: 'success' })
  uni.redirectTo({
    url: viewUrl,
    fail: () => {
      uni.navigateTo({ url: viewUrl })
    },
  })
}

function handleH5Share(): void {
  uni.showToast({ title: '请在右上角菜单分享给好友', icon: 'none' })
}

function goBack(): void {
  uni.navigateBack({
    fail: () => {
      uni.switchTab({ url: '/pages/insights/index' })
    },
  })
}

onShareAppMessage(() => ({
  title: directorEditShareTitle(context.value?.team.team_name),
  path: buildSharePath(
    buildDirectorEditUrl({
      teamId: context.value?.team.team_id ?? activeTeamId.value,
      circleCode: routeCtx.value.circleCode,
      fromCode: routeCtx.value.fromCode,
    }),
    'director',
  ),
}))
</script>

<style scoped src="./director.css"></style>
