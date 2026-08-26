<template>
  <view class="page-root">
    <image class="page-bg-img" :src="bgImage" mode="aspectFill" :webp="true" />
    <view class="page-bg-fade"></view>
    <view class="page-scroll" :class="{ 'page-scroll--locked': selectedRankingTeam || selectedStandingsTable }">
      <view class="page">
      <FiLoading
        v-if="loading"
        title="榜单加载中"
        caption="足球正在转动，球队榜和球员榜马上就绪。"
      />

      <view v-else-if="errorMessage" class="state-card state-card--error">
        <text>{{ errorMessage }}</text>
      </view>

      <template v-else-if="rankings">
        <RankingsStandingsLauncher
          v-if="previewStandingsTables.length"
          :tables="previewStandingsTables"
          @open="openStandingsSheet"
        />

        <RankingsSurfacePanel
          v-if="(scope === 'team' && hasTeamCategories) || (scope === 'player' && hasPlayerCategories)"
          v-model:scope="scope"
          v-model:standings-ranking-mode="standingsRankingMode"
          v-model:active-category-slug="activeCategorySlug"
          :rankings="rankings"
          @open-team="openRankingTeamSheet"
        />
      </template>

      <RankingsTeamSeasonSheet
        v-if="selectedRankingTeam"
        :team="selectedRankingTeam.team"
        :avatar="selectedRankingTeamAvatar"
        :standing-rank-text="selectedRankingTeamStandingRankText"
        :category-score-text="selectedRankingTeamCategoryScoreText"
        :category-label-text="selectedRankingTeamCategoryLabelText"
        :matches="selectedRankingTeamMatches"
        :loading="teamSeasonMatchesLoading"
        :error-message="teamSeasonMatchesErrorMessage"
        @close="closeRankingTeamSheet"
      />

      <RankingsPosterSheet
        v-if="selectedStandingsTable"
        :table="selectedStandingsTable"
        :season="rankings?.current_season ?? currentYear"
        :round="rankings?.round_number ?? null"
        @close="closeStandingsSheet"
        @poster-ready="handlePosterReady"
      />
    </view>
  </view>
  </view>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { onShow } from '@dcloudio/uni-app'
import FiLoading from '../../components/FiLoading.vue'
import RankingsStandingsLauncher from './components/RankingsStandingsLauncher.vue'
import RankingsSurfacePanel from './components/RankingsSurfacePanel.vue'
import RankingsTeamSeasonSheet from './components/RankingsTeamSeasonSheet.vue'
import RankingsPosterSheet from './components/RankingsPosterSheet.vue'
import { getAvailableRounds, getMatches, getRankings } from '../../api/insight'
import type {
  MatchCard,
  RankingsViewResponse,
  RoundReference,
  StandingsTable,
  StandingsTableEntry,
  TeamRankingEntry,
} from '../../types/insight'
import { extractApiErrorMessage } from '../../utils/apiError'
import { resolveTeamSeasonMatches, type TeamSeasonMatch } from '../../utils/teamSeasonMatches'
import { PHOENIX_STADIUM_BG_IMAGE_URL as bgImage } from '../../config/assets'
import { sortStandingsPreviewTables } from './helpers'
import { buildStandingsPosterSharePath, buildStandingsPosterShareTitle, type StandingsRankingMode } from './poster'
import { reportPageActivity } from '../../utils/userActivity'

interface StandingsPosterSharePayload {
  title: string
  path: string
  imageUrl?: string
}

interface SelectedRankingTeamSheet {
  team: TeamRankingEntry
  categoryLabel: string
  metricLabel: string
}

const currentYear = new Date().getFullYear()
const emit = defineEmits<{
  (event: 'page-scroll-lock-change', locked: boolean): void
}>()
const scope = ref<'team' | 'player'>('team')
const standingsRankingMode = ref<StandingsRankingMode>('with_penalty')
const loading = ref(true)
const errorMessage = ref('')
const rankings = ref<RankingsViewResponse | null>(null)
const activeCategorySlug = ref('')
const selectedStandingsPosterSlug = ref<string | null>(null)
const latestPosterImagePath = ref('')
const rounds = ref<RoundReference[]>([])
const allSeasonMatches = ref<MatchCard[] | null>(null)
const teamSeasonMatchesLoading = ref(false)
const teamSeasonMatchesErrorMessage = ref('')
const pendingAutoOpenStandingsSlug = ref<string | null>(null)
const selectedRankingTeam = ref<SelectedRankingTeamSheet | null>(null)

const hasTeamCategories = computed(() => (rankings.value?.team_categories?.length ?? 0) > 0)
const hasPlayerCategories = computed(() => (rankings.value?.player_categories?.length ?? 0) > 0)
const standingsTables = computed<StandingsTable[]>(() => rankings.value?.standings_tables ?? [])
const previewStandingsTables = computed(() => sortStandingsPreviewTables(standingsTables.value).slice(0, 2))
const selectedStandingsTable = computed(() =>
  previewStandingsTables.value.find((item) => item.slug === selectedStandingsPosterSlug.value) ?? null,
)
const selectedStandingsSharePayload = computed<StandingsPosterSharePayload | null>(() => {
  if (!selectedStandingsTable.value) {
    return null
  }

  const imageUrl = latestPosterImagePath.value || undefined
  return {
    title: buildStandingsPosterShareTitle(
      rankings.value?.current_season ?? currentYear,
      selectedStandingsTable.value,
    ),
    path: buildStandingsPosterSharePath(selectedStandingsTable.value),
    ...(imageUrl ? { imageUrl } : {}),
  }
})
const primaryStandingsTable = computed<StandingsTable | null>(() =>
  standingsTables.value.find((table) => table.slug === 'standings_with_penalty')
  ?? standingsTables.value.find((table) => table.slug === 'standings')
  ?? standingsTables.value[0]
  ?? null,
)
const standingsEntryByTeamId = computed(() =>
  new Map((primaryStandingsTable.value?.entries ?? []).map((entry) => [entry.team_id, entry])),
)
const selectedRankingStandingsEntry = computed<StandingsTableEntry | null>(() => {
  if (!selectedRankingTeam.value) {
    return null
  }

  return standingsEntryByTeamId.value.get(selectedRankingTeam.value.team.team_id) ?? null
})
const selectedRankingTeamAvatar = computed(() =>
  selectedRankingStandingsEntry.value?.avatar_storage_url
  || selectedRankingTeam.value?.team.avatar_storage_url
  || '',
)
const selectedRankingTeamMatches = computed<TeamSeasonMatch[]>(() => {
  if (!selectedRankingTeam.value || !allSeasonMatches.value) {
    return []
  }

  return resolveTeamSeasonMatches(selectedRankingTeam.value.team, allSeasonMatches.value)
})
const selectedRankingTeamStandingRankText = computed(() => {
  if (selectedRankingStandingsEntry.value) {
    return `第 ${selectedRankingStandingsEntry.value.rank_no}`
  }

  return '待同步'
})
const selectedRankingTeamCategoryScoreText = computed(() => {
  if (!selectedRankingTeam.value) {
    return '-'
  }

  return `${selectedRankingTeam.value.team.score_value} ${selectedRankingTeam.value.metricLabel}`
})
const selectedRankingTeamCategoryLabelText = computed(() => {
  if (!selectedRankingTeam.value) {
    return '当前榜单'
  }

  return selectedRankingTeam.value.categoryLabel
})

watch(
  () => Boolean(selectedRankingTeam.value || selectedStandingsTable.value),
  (locked) => {
    emit('page-scroll-lock-change', locked)
  },
)

function handlePosterReady(path: string): void {
  latestPosterImagePath.value = path
}

async function openStandingsSheet(slug: string): Promise<void> {
  selectedStandingsPosterSlug.value = slug
}

async function openSharedStandingsPoster(slug: string): Promise<void> {
  const table = previewStandingsTables.value.find((item) => item.slug === slug) ?? null
  if (!table) {
    pendingAutoOpenStandingsSlug.value = slug
    return
  }

  pendingAutoOpenStandingsSlug.value = null
  await openStandingsSheet(table.slug)
}

function closeStandingsSheet(): void {
  selectedStandingsPosterSlug.value = null
}

async function ensureAllSeasonMatchesLoaded(): Promise<void> {
  if (allSeasonMatches.value || teamSeasonMatchesLoading.value) {
    return
  }

  teamSeasonMatchesLoading.value = true
  teamSeasonMatchesErrorMessage.value = ''

  try {
    const currentSeason = rankings.value?.current_season ?? currentYear
    const availableRounds = rounds.value.length ? rounds.value : await getAvailableRounds(currentSeason)

    if (!rounds.value.length) {
      rounds.value = availableRounds
    }

    const responses = await Promise.all(
      availableRounds.map((round) =>
        getMatches({ mode: 'round', season: currentSeason, roundNumber: round.round_number }),
      ),
    )

    const matchMap = new Map<number, MatchCard>()
    for (const response of responses) {
      for (const match of response.matches) {
        matchMap.set(match.match_id, match)
      }
    }

    allSeasonMatches.value = Array.from(matchMap.values())
  } catch (error) {
    teamSeasonMatchesErrorMessage.value = extractApiErrorMessage(error, '球队赛季战绩加载失败，请稍后重试。')
  } finally {
    teamSeasonMatchesLoading.value = false
  }
}

async function openRankingTeamSheet(entry: TeamRankingEntry): Promise<void> {
  selectedRankingTeam.value = {
    team: entry,
    categoryLabel: activeTeamCategoryLabel(),
    metricLabel: isStandingsCategory() ? '积分' : '总计',
  }
  await ensureAllSeasonMatchesLoaded()
}

function isStandingsCategory(): boolean {
  return activeCategorySlug.value === 'standings'
}

function activeTeamCategoryLabel(): string {
  const category = rankings.value?.team_categories.find((item) => item.slug === activeCategorySlug.value)
  return category?.label ?? '球队榜'
}

function closeRankingTeamSheet(): void {
  selectedRankingTeam.value = null
  teamSeasonMatchesErrorMessage.value = ''
}

async function loadPage(): Promise<void> {
  loading.value = true
  errorMessage.value = ''

  try {
    const response = await getRankings({
      mode: 'live',
      season: currentYear,
      roundNumber: null,
    })

    rankings.value = response

    if (pendingAutoOpenStandingsSlug.value) {
      await openSharedStandingsPoster(pendingAutoOpenStandingsSlug.value)
    }
  } catch (error) {
    errorMessage.value = extractApiErrorMessage(error, '榜单加载失败，请稍后重试。')
  } finally {
    loading.value = false
  }
}

onShow(() => {
  reportPageActivity('rankings')
  void loadPage()
})

defineExpose({
  getStandingsPosterSharePayload: () => selectedStandingsSharePayload.value,
  openSharedStandingsPoster,
})
</script>

<style scoped lang="css">
.page-root { position: relative; }
.page-scroll {
  padding-top: var(--fi-brand-nav-height);
  position: relative;
  z-index: 1;
  box-sizing: border-box;
}
.page-scroll--locked {
  height: 100vh;
  overflow: hidden;
}
.page { padding: var(--fi-space-24) var(--fi-space-16) var(--fi-space-40); display: flex; flex-direction: column; gap: var(--fi-space-16); }
.state-card {
  background: rgba(255, 255, 255, 0.94);
  border-radius: var(--fi-radius-xl);
  padding: var(--fi-space-20);
  border: var(--fi-border-card);
  box-shadow: var(--fi-shadow-card);
}
.state-card--error text { font-size: var(--fi-font-28); color: #c03a2b; }
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
  background: linear-gradient(180deg, transparent 45%, rgba(247, 248, 250, 0.55) 78%, var(--fi-color-page-soft) 100%);
  pointer-events: none;
  z-index: 0;
}
</style>
