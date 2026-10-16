<template>
  <view class="page-root" :class="{ 'page-root--embedded': embedded }">
    <image v-if="!embedded" class="page-bg-img" :src="bgImage" mode="aspectFill" :webp="true" />
    <view v-if="!embedded" class="page-bg-fade"></view>
    <view class="page-scroll">
      <view class="page">
      <MatchesSkeleton v-if="loading" />
      <view v-else-if="errorMessage" class="state-card state-card--error"><text>{{ errorMessage }}</text></view>
      <template v-else>
        <MatchesSeasonProgress :rounds="rounds" :selected-round-number="selectedRoundNumber" @open-round="openRoundDialog" />
        <MatchesUpcomingPanel :sections="upcomingSections" :now-iso="pageNowIso" @open-round="openRoundDialog" @open-tech-stats="openMatchTechStats" />
        <MatchesRecentResults v-if="groupedMatches.length" :groups="groupedMatches" :now-iso="pageNowIso" @open-tech-stats="openMatchTechStats" />
      </template>
      </view>
    </view>

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
</template>

<script setup lang="ts">
import { onShow } from '@dcloudio/uni-app'
import MatchesSkeleton from './components/MatchesSkeleton.vue'
import { PHOENIX_STADIUM_BG_IMAGE_URL as bgImage } from '../../config/assets'
import MatchesSeasonProgress from './components/MatchesSeasonProgress.vue'
import MatchesUpcomingPanel from './components/MatchesUpcomingPanel.vue'
import MatchesRecentResults from './components/MatchesRecentResults.vue'
import MatchesRoundSheet from './components/MatchesRoundSheet.vue'
import MatchesTechStatsSheet from './components/MatchesTechStatsSheet.vue'
import { useMatchesPage } from './useMatchesPage'

defineProps<{ embedded?: boolean }>()

const {
  loading, errorMessage, rounds, pageNowIso, groupedMatches, upcomingSections,
  roundDialogLoading, roundDialogErrorMessage, selectedRoundNumber,
  selectedRoundMatches, selectedTechStatsMatch, loadPage, openRoundDialog,
  closeRoundDialog, openMatchTechStats, closeMatchTechStats,
} = useMatchesPage()

onShow(() => { void loadPage() })
</script>

<style scoped lang="css">
@import './matches-shared.css';
.page-root { position: relative; }
.page-scroll {
  padding-top: calc(var(--fi-brand-nav-height) + var(--fi-space-96));
  position: relative;
  z-index: 1;
}
.page-root--embedded .page-scroll {
  padding-top: 0;
}
.page {
  position: relative;
  padding: var(--fi-space-24) var(--fi-space-16) var(--fi-space-40);
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-card-gap);
}
.page-root--embedded .page {
  padding-top: 0;
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
</style>
