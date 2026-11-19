<template>
  <view class="page-root" :class="{ 'page-root--embedded': embedded }">
    <FiPageBackdrop v-if="!embedded" :image-url="pageBackgroundImageUrl" />
    <view class="page-scroll">
      <view class="page">
      <MatchesSkeleton v-if="loading" />
      <view v-else-if="errorMessage" class="state-card state-card--error"><text>{{ errorMessage }}</text></view>
      <template v-else>
        <template v-for="block in matchesBlocks" :key="block.key">
          <FiLayoutBlock v-if="block.key === 'matches_progress'" :background-image-url="block.background_image_url">
            <MatchesSeasonProgress :rounds="rounds" :selected-round-number="selectedRoundNumber" @open-round="openRoundDialog" />
          </FiLayoutBlock>
          <FiLayoutBlock v-else-if="block.key === 'matches_upcoming'" :background-image-url="block.background_image_url">
            <MatchesUpcomingPanel :sections="upcomingSections" :now-iso="pageNowIso" @open-round="openRoundDialog" @open-tech-stats="openMatchTechStats" />
          </FiLayoutBlock>
          <FiLayoutBlock v-else-if="block.key === 'matches_results' && groupedMatches.length" :background-image-url="block.background_image_url">
            <MatchesRecentResults :groups="groupedMatches" :now-iso="pageNowIso" @open-tech-stats="openMatchTechStats" />
          </FiLayoutBlock>
        </template>
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
import { computed } from 'vue'
import { onShow } from '@dcloudio/uni-app'
import FiPageBackdrop from '../../components/FiPageBackdrop.vue'
import FiLayoutBlock from '../../components/FiLayoutBlock.vue'
import { usePageLayout } from '../../composables/usePageLayout'
import MatchesSkeleton from './components/MatchesSkeleton.vue'
import MatchesSeasonProgress from './components/MatchesSeasonProgress.vue'
import MatchesUpcomingPanel from './components/MatchesUpcomingPanel.vue'
import MatchesRecentResults from './components/MatchesRecentResults.vue'
import MatchesRoundSheet from './components/MatchesRoundSheet.vue'
import MatchesTechStatsSheet from './components/MatchesTechStatsSheet.vue'
import { useMatchesPage } from './useMatchesPage'

defineProps<{ embedded?: boolean }>()

const { visibleBlocks, pageBackgroundImageUrl } = usePageLayout('rankings')
const matchesBlocks = computed(() => visibleBlocks.value.filter((b) => b.key.startsWith('matches_')))

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
