<template>
  <view class="page-root">
    <image class="page-bg-img" :src="bgImage" mode="aspectFill" :webp="true" />
    <view class="page-bg-fade"></view>
    <scroll-view scroll-y class="page-scroll">
      <view class="page">
      <view class="hero-card">
        <view class="hero-card__top">
          <view>
            <text class="eyebrow">Matches</text>
            <text class="hero-card__title">先看赛季进度，再看下一场对阵</text>
          </view>
          <text class="meta-note meta-note--hero">赛程 / 赛果</text>
        </view>

        <text class="hero-card__summary">
          顶部先给你当前赛季打到哪里了，再把这一轮还没踢的比赛和下一轮赛程放到前面。下面仍然保留最近完赛结果，方便继续复盘。
        </text>
      </view>

      <FiLoading v-if="loading" title="赛季进度加载中" caption="轮次和即将到来的比赛正在整理。" />
      <view v-else-if="errorMessage" class="state-card state-card--error"><text>{{ errorMessage }}</text></view>
      <template v-else>
        <MatchesSeasonProgress :rounds="rounds" :selected-round-number="selectedRoundNumber" @open-round="openRoundDialog" />
        <MatchesUpcomingPanel :sections="upcomingSections" :now-iso="pageNowIso" @open-round="openRoundDialog" @open-tech-stats="openMatchTechStats" />
        <MatchesRecentResults v-if="groupedMatches.length" :groups="groupedMatches" :now-iso="pageNowIso" @open-tech-stats="openMatchTechStats" />
      </template>
      <MatchesRoundSheet
        v-if="selectedRoundNumber !== null"
        :round-number="selectedRoundNumber"
        :matches="selectedRoundMatches"
        :loading="roundDialogLoading"
        :error-message="roundDialogErrorMessage"
        :now-iso="pageNowIso"
        @close="closeRoundDialog"
        @open-tech-stats="openMatchTechStats"
      />
      <MatchesTechStatsSheet v-if="selectedTechStatsMatch" :match="selectedTechStatsMatch" @close="closeMatchTechStats" />
      </view>
    </scroll-view>
  </view>
</template>

<script setup lang="ts">
import { onShareAppMessage, onShow } from '@dcloudio/uni-app'
import FiLoading from '../../components/FiLoading.vue'
import { PHOENIX_STADIUM_BG_IMAGE_URL as bgImage } from '../../config/assets'
import MatchesSeasonProgress from './components/MatchesSeasonProgress.vue'
import MatchesUpcomingPanel from './components/MatchesUpcomingPanel.vue'
import MatchesRecentResults from './components/MatchesRecentResults.vue'
import MatchesRoundSheet from './components/MatchesRoundSheet.vue'
import MatchesTechStatsSheet from './components/MatchesTechStatsSheet.vue'
import { useMatchesPage } from './useMatchesPage'

const {
  loading, errorMessage, rounds, pageNowIso, groupedMatches, upcomingSections,
  roundDialogLoading, roundDialogErrorMessage, selectedRoundNumber,
  selectedRoundMatches, selectedTechStatsMatch, loadPage, openRoundDialog,
  closeRoundDialog, openMatchTechStats, closeMatchTechStats,
} = useMatchesPage({ progressiveLoad: false })

onShow(() => { void loadPage() })

onShareAppMessage(() => ({
  title: '中超赛程和最近赛果，看看下一场对阵',
  path: '/pages/matches/index',
}))
</script>

<style scoped lang="css">
@import './matches-shared.css';

.page-root { position: relative; }
.page-scroll { height: 100vh; position: relative; z-index: 1; }

.page {
  position: relative;
  padding: var(--fi-space-24) var(--fi-space-16) var(--fi-space-40);
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-16);
}
.page-bg-img {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: var(--fi-space-600);
  pointer-events: none;
  z-index: 0;
}
.page-bg-fade {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: var(--fi-space-600);
  background: var(--fi-component-matches-page-bg-fade-background);
  pointer-events: none;
  z-index: 0;
}

.hero-card {
  position: relative; z-index: 1; background: var(--fi-color-ticket-recent-reflux-bucket-lock-body-background-2); border-radius: var(--fi-radius-xl);
  padding: var(--fi-space-20); border: var(--fi-component-matches-match-card-border); box-shadow: var(--fi-shadow-card);
  backdrop-filter: blur(var(--fi-space-18)); -webkit-backdrop-filter: blur(var(--fi-space-18));
}
.hero-card__top { display: flex; align-items: flex-start; justify-content: space-between; gap: var(--fi-space-12); }
.hero-card__title { display: block; margin-top: var(--fi-space-10); color: var(--fi-color-tech-stat-row-label-color); font-size: var(--fi-font-48); line-height: var(--fi-leading-ticket-1-08); font-weight: var(--fi-weight-extrabold); }
.hero-card__summary { display: block; margin-top: var(--fi-space-18); color: var(--fi-color-ticket-tracked-interest-item-meta-color); font-size: var(--fi-font-28); line-height: var(--fi-leading-relaxed); }

</style>
