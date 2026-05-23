<template>
  <view class="page-root">
    <image class="page-bg-img" :src="bgImage" mode="aspectFill" />
    <view class="page-bg-fade"></view>
    <scroll-view scroll-y class="page-scroll">
      <view class="page">
      <FiLoading
        v-if="loading"
        title="榜单加载中"
        caption="足球正在转动，球队榜和球员榜马上就绪。"
      />

      <view v-else-if="errorMessage" class="state-card state-card--error">
        <text>{{ errorMessage }}</text>
      </view>

      <template v-else>
        <view v-if="previewStandingsTables.length" class="panel standings-launcher">
          <view class="standings-launcher__grid">
            <view
              v-for="table in previewStandingsTables"
              :key="table.slug"
              class="standings-launcher-card"
              @click="openStandingsSheet(table.slug)"
            >
              <view class="standings-launcher-card__header">
                <view>
                  <text class="section-kicker">{{ table.label }}</text>
                  <text class="standings-launcher-card__title">查看完整图片</text>
                </view>
                <text class="meta-pill">查看</text>
              </view>

              <text class="standings-launcher-card__summary">{{ getStandingsPreviewSummary(table) }}</text>

              <view class="standings-launcher-card__footer">
                <text>{{ table.entries.length }} 支球队</text>
                <text class="standings-launcher-card__action">打开图片</text>
              </view>
            </view>
          </view>
        </view>

        <view class="rankings-controls">
          <view class="scope-toggle">
            <view
              class="scope-toggle__button"
              :class="{ active: scope === 'team' }"
              @click="scope = 'team'"
            >
              <text class="scope-toggle__button-text">球队榜</text>
            </view>
            <view
              class="scope-toggle__button"
              :class="{ active: scope === 'player' }"
              @click="scope = 'player'"
            >
              <text class="scope-toggle__button-text">球员榜</text>
            </view>
          </view>

          <view class="category-tabs-wrap">
            <scroll-view
              scroll-x
              class="pill-row"
              :scroll-left="categoryScrollLeft"
              scroll-with-animation
            >
              <view class="pill-row__list">
                <view
                  v-for="item in categoryOptions"
                  :id="`ranking-category-${item.slug}`"
                  :key="item.slug"
                  class="pill-row__item"
                  :class="{ active: item.slug === activeCategorySlug }"
                  @click="activeCategorySlug = item.slug"
                >
                  <text class="pill-row__item-text">{{ item.label }}</text>
                </view>
              </view>
            </scroll-view>
          </view>
        </view>

        <view v-if="scope === 'team' && activeTeamCategory" class="panel ranking-surface">
          <view class="ranking-list">
            <view class="section-heading section-heading--compact">
              <view>
                <text class="section-kicker">{{ activeTeamSectionKicker }}</text>
                <text class="section-title">{{ activeTeamCategory.label }}</text>
              </view>
              <view v-if="isStandingsTeamCategory" class="standings-mode-toggle">
                <view
                  class="standings-mode-toggle__item"
                  :class="{ active: standingsRankingMode === 'with_penalty' }"
                  @click.stop="standingsRankingMode = 'with_penalty'"
                >
                  <text>含罚分</text>
                </view>
                <view
                  class="standings-mode-toggle__item"
                  :class="{ active: standingsRankingMode === 'without_penalty' }"
                  @click.stop="standingsRankingMode = 'without_penalty'"
                >
                  <text>无罚分</text>
                </view>
              </view>
            </view>

            <view
              v-for="entry in activeTeamRankingEntries"
              :key="`${activeTeamCategory.slug}-${standingsRankingMode}-${entry.team_id}`"
              class="ranking-row ranking-row--interactive"
              hover-class="ranking-row--pressed"
              hover-stay-time="100"
              @click="openRankingTeamSheet(entry)"
            >
              <view class="ranking-row__rank-wrap">
                <text class="ranking-row__rank" :class="`ranking-row__rank--${entry.rank_no}`">#{{ entry.rank_no }}</text>
              </view>
              <image :src="entry.avatar_storage_url || ''" mode="aspectFit" class="ranking-row__avatar" />
              <view class="ranking-row__body">
                <text class="ranking-row__name">{{ entry.team_name }}</text>
                <text class="ranking-row__note">点击查看球队信息和赛程</text>
              </view>
              <view class="ranking-row__metric">
                <text class="ranking-row__metric-value">{{ entry.score_value }}</text>
                <text class="ranking-row__metric-note">{{ activeTeamMetricLabel }}</text>
              </view>
            </view>
          </view>
        </view>

        <view v-if="scope === 'player' && activePlayerCategory" class="panel ranking-surface">
          <view class="ranking-list">
            <view class="section-heading section-heading--compact">
              <view>
                <text class="section-kicker">实时球员累计榜</text>
                <text class="section-title">{{ activePlayerCategory.label }}</text>
              </view>
            </view>

            <view
              v-for="entry in activePlayerCategory.entries"
              :key="`${activePlayerCategory.slug}-${entry.player_id}`"
              class="ranking-row"
            >
              <text class="ranking-row__rank" :class="`ranking-row__rank--${entry.rank_no}`">#{{ entry.rank_no }}</text>
              <image :src="entry.avatar_storage_url || ''" mode="aspectFill" class="ranking-row__avatar ranking-row__avatar--player" />
              <view class="ranking-row__body">
                <text class="ranking-row__name">{{ entry.player_name }}</text>
                <text class="ranking-row__note">{{ entry.team_name }}</text>
              </view>
              <view class="ranking-row__metric">
                <text class="ranking-row__metric-value">{{ entry.score_value }}</text>
                <text class="ranking-row__metric-note">{{ activePlayerCategory.label }}</text>
              </view>
            </view>
          </view>
        </view>
      </template>

      <view v-if="selectedRankingTeam" class="standings-sheet-mask" @click.self="closeRankingTeamSheet">
        <view class="standings-sheet standings-sheet--team-season" @click.stop="consumeSheetTap">
          <view class="section-heading section-heading--compact">
            <view>
              <text class="section-kicker">球队赛季战绩</text>
              <view class="team-season-sheet__title">
                <image
                  :src="selectedRankingTeamAvatar"
                  mode="aspectFit"
                  class="team-season-sheet__title-avatar"
                />
                <text class="section-title team-season-sheet__title-name">{{ selectedRankingTeam.team.team_name }}</text>
              </view>
            </view>
            <button class="standings-sheet__close" @click="closeRankingTeamSheet">关闭</button>
          </view>

          <view class="team-season-sheet__summary">
            <view class="team-season-sheet__summary-main">
              <view class="team-season-sheet__summary-meta-grid">
                <view class="team-season-sheet__summary-metric">
                  <text class="team-season-sheet__summary-value">{{ selectedRankingTeamStandingRankText }}</text>
                  <text class="team-season-sheet__summary-label">积分榜排名</text>
                </view>
                <view class="team-season-sheet__summary-metric">
                  <text class="team-season-sheet__summary-value">{{ selectedRankingTeamCategoryScoreText }}</text>
                  <text class="team-season-sheet__summary-label">{{ selectedRankingTeamCategoryLabelText }}</text>
                </view>
                <view class="team-season-sheet__summary-metric">
                  <text class="team-season-sheet__summary-value">{{ selectedRankingTeamRecordText }}</text>
                  <text class="team-season-sheet__summary-label">赛季战绩</text>
                </view>
              </view>
            </view>
          </view>

          <FiLoading
            v-if="teamSeasonMatchesLoading"
            title="赛季战绩加载中"
            caption="正在整理这支球队本赛季的每场比赛。"
          />

          <view v-else-if="teamSeasonMatchesErrorMessage" class="state-card state-card--error team-season-sheet__state">
            <text>{{ teamSeasonMatchesErrorMessage }}</text>
          </view>

          <scroll-view v-else-if="selectedRankingTeamMatches.length" scroll-y class="team-season-sheet__list">
            <view
              v-for="(match, index) in selectedRankingTeamMatches"
              :key="match.matchId"
              class="team-season-match-row"
              :style="getTeamMatchRowStyle(index)"
            >
              <view class="team-season-match-row__meta">
                <text>第 {{ match.roundNumber }} 轮 · {{ match.matchDate }} {{ match.matchTime }}</text>
                <view class="team-season-match-row__meta-right">
                  <text class="team-season-match-row__venue" :class="match.isHomeTeam ? 'team-season-match-row__venue--home' : 'team-season-match-row__venue--away'">
                    {{ match.venueLabel }}
                  </text>
                  <text class="team-season-match-row__result" :class="`team-season-match-row__result--${match.resultTone}`">
                    {{ match.resultLabel }}
                  </text>
                </view>
              </view>
              <view class="team-season-match-row__body">
                <view class="team-season-match-row__side team-season-match-row__side--left">
                  <image class="team-season-match-row__avatar" :src="match.teamAvatar || ''" mode="aspectFit" />
                  <text class="team-season-match-row__team team-season-match-row__team--active">{{ match.teamName }}</text>
                </view>
                <text class="team-season-match-row__score">{{ match.scoreText }}</text>
                <view class="team-season-match-row__side team-season-match-row__side--right">
                  <text class="team-season-match-row__team team-season-match-row__team--away">{{ match.opponentName }}</text>
                  <image class="team-season-match-row__avatar" :src="match.opponentAvatar || ''" mode="aspectFit" />
                </view>
              </view>
            </view>
          </scroll-view>

          <view v-else class="team-season-sheet__empty">
            <text>这支球队当前还没有可展示的赛季比赛记录。</text>
          </view>
        </view>
      </view>

      <view v-if="selectedStandingsTable" class="standings-sheet-mask" @click.self="closeStandingsSheet">
        <view class="standings-sheet">
          <view class="section-heading section-heading--compact">
            <view>
              <text class="section-kicker">完整积分榜</text>
              <text class="section-title">{{ selectedStandingsTable.label }}</text>
            </view>
            <view class="standings-sheet__actions">
              <button
                v-if="posterImagePath"
                class="standings-sheet__share"
                open-type="share"
              >
                分享
              </button>
              <button
                v-if="posterImagePath"
                class="standings-sheet__save"
                @click="savePosterImage"
              >
                保存图片
              </button>
              <button class="standings-sheet__close" @click="closeStandingsSheet">关闭</button>
            </view>
          </view>

          <FiLoading
            v-if="posterGenerating"
            title="榜单图片生成中"
            caption="正在把当前积分榜整理成一张可保存的图片。"
          />

          <view v-else-if="posterImagePath" class="standings-sheet__poster">
            <image :src="posterImagePath" mode="widthFix" class="standings-sheet__poster-image" />
            <text class="standings-sheet__hint standings-sheet__hint--resolved">{{ standingsPosterHintText }}</text>
          </view>

          <view v-if="posterErrorMessage" class="state-card state-card--error standings-sheet__error">
            <text>{{ posterErrorMessage }}</text>
          </view>

          <view v-if="!posterImagePath" class="standings-sheet__list">
            <view v-for="entry in selectedStandingsTable.entries" :key="`${selectedStandingsTable.slug}-${entry.team_id}`" class="standings-sheet__row">
              <text class="standings-sheet__rank">{{ entry.rank_no }}</text>
              <text class="standings-sheet__name">{{ entry.team_name }}</text>
              <view class="standings-sheet__metrics">
                <view
                  v-for="metric in getStandingsFallbackMetrics(selectedStandingsTable, entry)"
                  :key="metric.label"
                  class="standings-sheet__metric"
                >
                  <text class="standings-sheet__metric-value">{{ metric.value }}</text>
                  <text class="standings-sheet__metric-label">{{ metric.label }}</text>
                </view>
              </view>
            </view>
          </view>
        </view>
      </view>

      <canvas canvas-id="standingsPosterCanvas" class="standings-poster-canvas" />
    </view>
  </scroll-view>
  </view>
</template>

<script setup lang="ts">
import { computed, getCurrentInstance, nextTick, ref, watch } from 'vue'
import { onShow } from '@dcloudio/uni-app'
import FiLoading from '../../components/FiLoading.vue'
import { getAvailableRounds, getMatches, getRankings } from '../../api/insight'
import type { MatchCard, PlayerRankingCategory, RankingsViewResponse, RoundReference, StandingsTable, StandingsTableEntry, TeamRankingCategory, TeamRankingEntry } from '../../types/insight'
import { extractApiErrorMessage } from '../../utils/apiError'
import { type TeamSeasonMatch, resolveTeamSeasonMatches } from '../../utils/teamSeasonMatches'
import bgImage from '../../static/rankings/bg.webp'
import { buildStandingsFallbackMetrics, buildStandingsPosterColumns, buildStandingsPosterMetrics, buildStandingsPosterSharePath, buildStandingsPosterShareTitle, buildStandingsPosterTeamLayout, buildStandingsRankingEntries, type StandingsRankingMode } from './poster'
import { reportPageActivity } from '../../utils/userActivity'

const standingsPosterHintText = '图片右下角已附公众号二维码水印，可直接保存或转发。'
const standingsPosterTitleText = '中超积分榜'
const standingsPosterBrandText = '足球洞察 · 何止编程'
const standingsPosterSourceText = '数据来自当前抓取时点的赛季累计值'
const standingsPosterQrCaption = '微信扫码查看'
const standingsPosterQrModules = [
  '111111101101101011000101101111111',
  '100000100000000011010110101000001',
  '101110100111010000011110101011101',
  '101110101111101101111010001011101',
  '101110101100101000111111001011101',
  '100000101100011011110110101000001',
  '111111101010101010101010101111111',
  '000000001101001111000111000000000',
  '100010111101110011111111111111001',
  '000110010011011110101011000101111',
  '110011110010000101001111100001010',
  '100000000101101011101111001000001',
  '111110110011111100101001001111010',
  '111000001110010011010001010101100',
  '110010100010110010110111110000010',
  '100010001111000001111101011110000',
  '001101100000011101001111101010110',
  '111101011011010111100111010001110',
  '111100100110101100101001010001010',
  '000101010000000100011011010000011',
  '110010110101011010000011001111010',
  '110100010011101001111001100101110',
  '001110111111010101101111010100010',
  '000111000011100111100100010011001',
  '111000101110110011011101111110000',
  '000000001101001100000001100010111',
  '111111101010101101001110101011010',
  '100000100101010001110100100011010',
  '101110101101111100111000111111001',
  '101110100110101010010111101011010',
  '101110100001001011010001101110000',
  '100000100101110001011110101100000',
  '111111101010110101101111111011101',
]
const instance = getCurrentInstance()
const scope = ref<'team' | 'player'>('team')
const standingsRankingMode = ref<StandingsRankingMode>('with_penalty')
const loading = ref(true)
const errorMessage = ref('')
const rankings = ref<RankingsViewResponse | null>(null)
const activeCategorySlug = ref('')
const categoryScrollLeft = ref(0)
const selectedStandingsPosterSlug = ref<string | null>(null)
const posterImagePath = ref('')
const posterGenerating = ref(false)
const posterErrorMessage = ref('')
const posterCache = new Map<string, string>()
const posterLogoCache = new Map<string, string | null>()
const rounds = ref<RoundReference[]>([])
const allSeasonMatches = ref<MatchCard[] | null>(null)
const teamSeasonMatchesLoading = ref(false)
const teamSeasonMatchesErrorMessage = ref('')
const pendingAutoOpenStandingsSlug = ref<string | null>(null)

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

const selectedRankingTeam = ref<SelectedRankingTeamSheet | null>(null)

const teamCategories = computed<TeamRankingCategory[]>(() => rankings.value?.team_categories ?? [])
const playerCategories = computed<PlayerRankingCategory[]>(() => rankings.value?.player_categories ?? [])
const standingsTables = computed<StandingsTable[]>(() => rankings.value?.standings_tables ?? [])
const primaryStandingsTable = computed<StandingsTable | null>(() =>
  standingsTables.value.find((table) => table.slug === 'standings_with_penalty')
  ?? standingsTables.value.find((table) => table.slug === 'standings')
  ?? standingsTables.value[0]
  ?? null,
)
const previewStandingsTables = computed<StandingsTable[]>(() => {
  return [...standingsTables.value]
    .sort((left, right) => priority(left.slug) - priority(right.slug))
    .slice(0, 2)
})
const categoryOptions = computed(() =>
  scope.value === 'team'
    ? teamCategories.value.map((item) => ({ slug: item.slug, label: item.label }))
    : playerCategories.value.map((item) => ({ slug: item.slug, label: item.label })),
)
const activeTeamCategory = computed(() => teamCategories.value.find((item) => item.slug === activeCategorySlug.value) ?? null)
const activePlayerCategory = computed(() => playerCategories.value.find((item) => item.slug === activeCategorySlug.value) ?? null)
const isStandingsTeamCategory = computed(() => activeTeamCategory.value?.slug === 'standings')
const activeTeamRankingEntries = computed<TeamRankingEntry[]>(() => {
  if (!activeTeamCategory.value) {
    return []
  }

  if (!isStandingsTeamCategory.value) {
    return activeTeamCategory.value.entries
  }

  return buildStandingsRankingEntries(standingsTables.value, standingsRankingMode.value)
})
const selectedStandingsTable = computed(() =>
  previewStandingsTables.value.find((item) => item.slug === selectedStandingsPosterSlug.value) ?? null,
)
const selectedStandingsSharePayload = computed<StandingsPosterSharePayload | null>(() => {
  if (!selectedStandingsTable.value) {
    return null
  }

  const imageUrl = posterImagePath.value || undefined
  return {
    title: buildStandingsPosterShareTitle(rankings.value?.current_season ?? new Date().getFullYear(), selectedStandingsTable.value),
    path: buildStandingsPosterSharePath(selectedStandingsTable.value),
    ...(imageUrl ? { imageUrl } : {}),
  }
})
const activeTeamSectionKicker = computed(() =>
  activeTeamCategory.value?.slug === 'standings' ? '实时球队排名' : '实时球队累计榜',
)
const activeTeamMetricLabel = computed(() =>
  activeTeamCategory.value?.slug === 'standings' ? '积分' : '总计',
)
const activeTeamEntryLabel = computed(() =>
  activeTeamCategory.value?.slug === 'standings' ? '积分榜' : activeTeamCategory.value?.label ?? '',
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
const selectedRankingTeamRecordParts = computed(() => {
  const finishedMatches = selectedRankingTeamMatches.value.filter((match) =>
    match.resultTone === 'win' || match.resultTone === 'draw' || match.resultTone === 'loss',
  )
  const wins = finishedMatches.filter((match) => match.resultTone === 'win').length
  const draws = finishedMatches.filter((match) => match.resultTone === 'draw').length
  const losses = finishedMatches.filter((match) => match.resultTone === 'loss').length

  return {
    finishedMatches: finishedMatches.length,
    wins,
    draws,
    losses,
  }
})
const selectedRankingTeamRecordText = computed(() => {
  const record = selectedRankingTeamRecordParts.value
  return `${record.wins}胜 ${record.draws}平 ${record.losses}负`
})

watch(
  categoryOptions,
  (items) => {
    if (!items.length) {
      activeCategorySlug.value = ''
      return
    }

    if (!items.some((item) => item.slug === activeCategorySlug.value)) {
      activeCategorySlug.value = items[0]?.slug ?? ''
    }
  },
  { immediate: true },
)

watch(activeCategorySlug, () => {
  void centerActiveCategory()
})

function priority(slug: string): number {
  if (slug === 'standings_with_penalty') {
    return 0
  }

  if (slug === 'standings_without_penalty') {
    return 1
  }

  return 2
}

function getStandingsPreviewSummary(table: StandingsTable): string {
  if (table.slug === 'standings_without_penalty') {
    return `榜首是 ${table.entries[0]?.team_name ?? '暂无数据'}，按理论积分重新排序。`
  }

  const impactedCount = table.entries.filter((item) => item.points_adjustment !== 0).length
  return impactedCount > 0
    ? `${impactedCount} 支球队当前存在积分调整，查看完整实际积分榜。`
    : '当前所有球队积分与理论积分一致。'
}

function getDisplayedPoints(table: StandingsTable, entry: StandingsTableEntry): number {
  return table.slug === 'standings_without_penalty' ? entry.points_without_penalty : entry.points
}

function getStandingsFallbackMetrics(table: StandingsTable, entry: StandingsTableEntry) {
  return buildStandingsFallbackMetrics(table, entry)
}

function buildPosterSubtitle(table: StandingsTable): string {
  const season = rankings.value?.current_season ?? new Date().getFullYear()
  const round = rankings.value?.round_number ?? null
  return round
    ? String(season) + ' ' + table.label + ' · 第 ' + String(round) + ' 轮'
    : String(season) + ' ' + table.label
}

function truncatePosterTeamName(name: string): string {
  return name.length > 10 ? name.slice(0, 9) + '…' : name
}

function hasRectShape(value: unknown): value is { left: number; width: number } {
  return !!value && typeof value === 'object'
    && typeof (value as { left?: unknown }).left === 'number'
    && typeof (value as { width?: unknown }).width === 'number'
}

function hasScrollLeft(value: unknown): value is { scrollLeft: number } {
  return !!value && typeof value === 'object'
    && typeof (value as { scrollLeft?: unknown }).scrollLeft === 'number'
}

async function centerActiveCategory(): Promise<void> {
  if (!instance || !activeCategorySlug.value) {
    return
  }

  await nextTick()

  const query = uni.createSelectorQuery().in(instance)
  query.select('.pill-row').boundingClientRect()
  query.select('.pill-row').scrollOffset(() => {})
  query.select(`#ranking-category-${activeCategorySlug.value}`).boundingClientRect()
  query.exec((result) => {
    const [rawScrollRect, rawScrollOffset, rawPillRect] = (result ?? []) as unknown[]

    if (!hasRectShape(rawScrollRect) || !hasScrollLeft(rawScrollOffset) || !hasRectShape(rawPillRect)) {
      return
    }

    const scrollRect = rawScrollRect
    const scrollOffset = rawScrollOffset
    const pillRect = rawPillRect
    const delta = (pillRect.left + pillRect.width / 2) - (scrollRect.left + scrollRect.width / 2)
    const nextScrollLeft = Math.max(0, Math.round(scrollOffset.scrollLeft + delta))

    if (nextScrollLeft !== categoryScrollLeft.value) {
      categoryScrollLeft.value = nextScrollLeft
    }
  })
}

async function openStandingsSheet(slug: string): Promise<void> {
  selectedStandingsPosterSlug.value = slug
  posterErrorMessage.value = ''
  posterImagePath.value = ''

  if (posterCache.has(slug)) {
    posterImagePath.value = posterCache.get(slug) ?? ''
    return
  }

  await nextTick()
  const table = previewStandingsTables.value.find((item) => item.slug === slug) ?? null
  if (!table) {
    return
  }

  await generatePoster(table)
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
  posterErrorMessage.value = ''
}

function consumeSheetTap(): void {}

function getTeamMatchRowStyle(index: number) {
  return {
    '--team-match-delay': `${100 + index * 55}ms`,
  }
}

async function ensureAllSeasonMatchesLoaded(): Promise<void> {
  if (allSeasonMatches.value || teamSeasonMatchesLoading.value) {
    return
  }

  teamSeasonMatchesLoading.value = true
  teamSeasonMatchesErrorMessage.value = ''

  try {
    const currentSeason = rankings.value?.current_season ?? new Date().getFullYear()
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

async function openRankingTeamSheet(team: TeamRankingEntry): Promise<void> {
  selectedRankingTeam.value = {
    team,
    categoryLabel: activeTeamCategory.value?.label ?? '球队榜',
    metricLabel: activeTeamMetricLabel.value,
  }
  await ensureAllSeasonMatchesLoaded()
}

function closeRankingTeamSheet(): void {
  selectedRankingTeam.value = null
  teamSeasonMatchesErrorMessage.value = ''
}

async function generatePoster(table: StandingsTable): Promise<void> {
  const canvasId = 'standingsPosterCanvas'
  const width = 1080
  const rowHeight = 60
  const headerHeight = 248
  const footerHeight = 196
  const qrSize = 128
  const maxRows = Math.min(table.entries.length, 16)
  const height = headerHeight + footerHeight + rowHeight * maxRows
  posterGenerating.value = true

  try {
    const context = uni.createCanvasContext(canvasId, instance)
    const posterEntries = table.entries.slice(0, maxRows)
    const posterLogoPaths = await resolvePosterLogoPaths(posterEntries)

    context.setFillStyle('#f3f3f6')
    context.fillRect(0, 0, width, height)

    context.setFillStyle('#ffffff')
    roundRect(context, 36, 36, width - 72, height - 72, 34)
    context.fill()

    context.setFillStyle('#121212')
    context.setFontSize(48)
    context.fillText(String(rankings.value?.current_season ?? new Date().getFullYear()) + ' ' + standingsPosterTitleText, 88, 120)

    context.setFillStyle('#8f9198')
    context.setFontSize(28)
    context.fillText(buildPosterSubtitle(table), 88, 166)
    context.fillText(table.note, 88, 204)

    context.setStrokeStyle('#ececf1')
    context.setLineWidth(2)
    context.beginPath()
    context.moveTo(88, 228)
    context.lineTo(width - 88, 228)
    context.stroke()

    context.setFillStyle('#8f9198')
    context.setFontSize(24)
    buildStandingsPosterColumns(table).forEach((column) => {
      context.fillText(column.label, column.x, 270)
    })

    context.setFillStyle('#121212')
    context.setFontSize(28)

    posterEntries.forEach((entry, index) => {
      const y = 324 + index * rowHeight
      const teamLayout = buildStandingsPosterTeamLayout(Boolean(posterLogoPaths.get(entry.team_id)))
      context.setFillStyle(index < 3 ? '#f97316' : '#121212')
      context.fillText(String(entry.rank_no), 88, y)
      drawPosterTeamLogo(context, posterLogoPaths.get(entry.team_id) ?? null, teamLayout.logoX, y - 22, teamLayout.logoSize)
      context.setFillStyle('#121212')
      context.fillText(truncatePosterTeamName(entry.team_name), teamLayout.nameX, y)
      buildStandingsPosterMetrics(table, entry).forEach((metric) => {
        if (metric.highlight) {
          context.setFillStyle(String(entry.points_adjustment > 0 ? '#16a34a' : '#dc2626'))
        } else {
          context.setFillStyle('#121212')
        }

        context.setFontSize(metric.compact ? 20 : 28)
        context.fillText(metric.value, metric.x, y)
      })
      context.setFillStyle('#121212')
      context.setFontSize(28)

      context.setStrokeStyle('#f0f1f5')
      context.setLineWidth(1)
      context.beginPath()
      context.moveTo(88, y + 24)
      context.lineTo(width - 88, y + 24)
      context.stroke()
    })

    context.setFillStyle('#8f9198')
    context.setFontSize(24)
    context.fillText(standingsPosterBrandText, 88, height - 70)
    context.fillText(standingsPosterSourceText, 88, height - 36)

    drawPosterQrCode(context, width - qrSize - 88, height - footerHeight + 24, qrSize)
    context.setFillStyle('#8f9198')
    context.setFontSize(20)
    context.fillText(standingsPosterQrCaption, width - qrSize - 88, height - footerHeight + 176)

    await new Promise<void>((resolve) => {
      context.draw(false, () => resolve())
    })

    const tempFilePath = await new Promise<string>((resolve, reject) => {
      uni.canvasToTempFilePath({
        canvasId,
        width,
        height,
        destWidth: width,
        destHeight: height,
        success: (result) => resolve(result.tempFilePath),
        fail: (error) => reject(error),
      }, instance)
    })

    posterCache.set(table.slug, tempFilePath)
    posterImagePath.value = tempFilePath
  } catch (error) {
    posterErrorMessage.value = extractApiErrorMessage(error, '积分榜图片生成失败，请稍后重试。')
  } finally {
    posterGenerating.value = false
  }
}

async function resolvePosterLogoPaths(entries: StandingsTableEntry[]): Promise<Map<number, string | null>> {
  const resolved = await Promise.all(entries.map(async (entry) => {
    return [entry.team_id, await resolvePosterLogoPath(entry.avatar_storage_url)] as const
  }))

  return new Map(resolved)
}

async function resolvePosterLogoPath(src: string | null): Promise<string | null> {
  if (!src) {
    return null
  }

  if (posterLogoCache.has(src)) {
    return posterLogoCache.get(src) ?? null
  }

  try {
    const result = await new Promise<UniApp.GetImageInfoSuccessData>((resolve, reject) => {
      uni.getImageInfo({
        src,
        success: resolve,
        fail: reject,
      })
    })

    posterLogoCache.set(src, result.path)
    return result.path
  } catch {
    posterLogoCache.set(src, null)
    return null
  }
}

function roundRect(
  context: UniApp.CanvasContext,
  x: number,
  y: number,
  width: number,
  height: number,
  radius: number,
): void {
  context.beginPath()
  context.moveTo(x + radius, y)
  context.arcTo(x + width, y, x + width, y + height, radius)
  context.arcTo(x + width, y + height, x, y + height, radius)
  context.arcTo(x, y + height, x, y, radius)
  context.arcTo(x, y, x + width, y, radius)
  context.closePath()
}

function drawPosterQrCode(
  context: UniApp.CanvasContext,
  x: number,
  y: number,
  size: number,
): void {
  const quietZone = 4
  const moduleCount = standingsPosterQrModules.length
  const totalCount = moduleCount + quietZone * 2
  const cellSize = size / totalCount

  context.setFillStyle('#ffffff')
  context.fillRect(x, y, size, size)
  context.setFillStyle('#121212')

  standingsPosterQrModules.forEach((row, rowIndex) => {
    for (let columnIndex = 0; columnIndex < row.length; columnIndex += 1) {
      if (row[columnIndex] !== '1') {
        continue
      }

      context.fillRect(
        x + (columnIndex + quietZone) * cellSize,
        y + (rowIndex + quietZone) * cellSize,
        cellSize,
        cellSize,
      )
    }
  })
}

function drawPosterTeamLogo(
  context: UniApp.CanvasContext,
  logoPath: string | null,
  x: number,
  y: number,
  size: number,
): void {
  if (!logoPath || size <= 0) {
    return
  }

  const radius = size / 2
  context.save()
  context.beginPath()
  context.arc(x + radius, y + radius, radius, 0, Math.PI * 2, false)
  context.clip()
  context.drawImage(logoPath, x, y, size, size)
  context.restore()
}

function savePosterImage(): void {
  if (!posterImagePath.value) {
    return
  }

  uni.saveImageToPhotosAlbum({
    filePath: posterImagePath.value,
    success: () => {
      uni.showToast({ title: '已保存到相册', icon: 'success' })
    },
    fail: (error) => {
      uni.showToast({
        title: extractApiErrorMessage(error, '保存失败'),
        icon: 'none',
      })
    },
  })
}

async function loadPage(): Promise<void> {
  loading.value = true
  errorMessage.value = ''

  try {
    const response = await getRankings({
      mode: 'live',
      season: new Date().getFullYear(),
      roundNumber: null,
    })

    rankings.value = response

    const options = response.team_categories.length ? response.team_categories : response.player_categories
    if (!options.some((item) => item.slug === activeCategorySlug.value)) {
      activeCategorySlug.value = options[0]?.slug ?? ''
    }

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
.page-scroll { height: 100vh; position: relative; z-index: 1; }
.page { padding: 128rpx 16rpx 40rpx; display: flex; flex-direction: column; gap: 16rpx; }
.panel, .state-card {
  background: rgba(255,255,255,0.94);
  border-radius: 36rpx;
  padding: 20rpx;
  border: 2rpx solid rgba(236, 236, 241, 0.95);
  box-shadow: 0 20rpx 48rpx rgba(26,28,36,0.06);
}
.section-heading, .standings-launcher-card__header, .standings-launcher-card__footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.section-heading, .standings-launcher-card__header { align-items: flex-start; gap: 12rpx; }
.section-kicker {
  margin: 0;
  color: #8f9198;
  font-size: 22rpx;
  font-weight: 700;
  letter-spacing: 3rpx;
}
.section-title, .standings-launcher-card__title {
  display: block;
  margin-top: 10rpx;
  color: #2a2c31;
  font-size: 48rpx;
  line-height: 1.08;
  font-weight: 800;
}
.section-title, .standings-launcher-card__title { font-size: 44rpx; }
.standings-launcher-card__summary {
  display: block;
  margin-top: 18rpx;
  color: #6b707b;
  font-size: 28rpx;
  line-height: 1.7;
}
.meta-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  white-space: nowrap;
  line-height: 1;
  box-sizing: border-box;
  padding: 14rpx 24rpx;
  border-radius: 999rpx;
  border: 2rpx solid rgba(232, 233, 238, 0.95);
  background: #f6f7fb;
  color: #6d7280;
  font-size: 24rpx;
}
.standings-launcher {
  display: grid;
  gap: 0;
  padding-top: 4rpx;
  padding-bottom: 8rpx;
}
.standings-launcher__grid {
  display: grid;
  gap: 0;
  border-top: 2rpx solid rgba(235, 236, 241, 0.86);
}
.standings-launcher-card {
  position: relative;
  display: grid;
  gap: 8rpx;
  padding: 22rpx 0;
  border-bottom: 2rpx solid rgba(235, 236, 241, 0.86);
  background: transparent;
}
.standings-launcher-card__header {
  align-items: center;
}
.standings-launcher-card__title {
  font-size: 32rpx;
  margin-top: 8rpx;
  line-height: 1.18;
}
.standings-launcher-card__summary {
  margin-top: 0;
  font-size: 24rpx;
  line-height: 1.5;
  color: #747986;
}
.standings-launcher-card__footer {
  margin-top: 4rpx;
  color: #9a9ea8;
  font-size: 22rpx;
}
.standings-launcher-card__action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: #121212;
  font-size: 22rpx;
  font-weight: 800;
  line-height: 1;
  white-space: nowrap;
}
.rankings-controls {
  display: grid;
  gap: 18rpx;
  width: 100%;
  padding: 2rpx 4rpx 8rpx;
  overflow: hidden;
}
.scope-toggle {
  display: flex;
  align-items: flex-end;
  width: 100%;
  gap: 34rpx;
  padding: 0 8rpx;
  border-bottom: 2rpx solid rgba(231, 232, 238, 0.9);
}
.standings-mode-toggle {
  display: inline-grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 2rpx;
  flex: 0 0 auto;
  padding: 3rpx;
  border-radius: 999rpx;
  background: rgba(242, 243, 247, 0.92);
}
.standings-mode-toggle__item {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 96rpx;
  padding: 12rpx 16rpx;
  border-radius: 999rpx;
  color: #7b818d;
  font-size: 22rpx;
  line-height: 1;
  white-space: nowrap;
}
.standings-mode-toggle__item.active {
  color: #121212;
  font-weight: 700;
  background: #ffffff;
  box-shadow: 0 6rpx 14rpx rgba(18,18,18,0.05);
}
.scope-toggle__button {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: 0 0 auto;
  min-width: 0;
  min-height: 68rpx;
  padding: 12rpx 0 20rpx;
  text-align: center;
  background: transparent;
  color: #7f8490;
  font-size: 28rpx;
  font-weight: 700;
  white-space: nowrap;
  line-height: 1;
  overflow: hidden;
  box-sizing: border-box;
}
.scope-toggle__button::after {
  content: '';
  position: absolute;
  left: 50%;
  bottom: -2rpx;
  width: 0;
  height: 4rpx;
  border-radius: 999rpx;
  background: #15161b;
  transform: translateX(-50%);
  transition: width 160ms ease;
}
.scope-toggle__button-text {
  display: block;
  width: 100%;
  text-align: center;
}
.category-tabs-wrap {
  display: grid;
  overflow: hidden;
}
.pill-row__item {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: fit-content;
  border-radius: 18rpx;
  padding: 18rpx 22rpx;
  text-align: center;
  background: transparent;
  color: #8f9198;
  font-size: 26rpx;
  white-space: nowrap;
  line-height: 1;
  box-sizing: border-box;
}
.scope-toggle__button.active {
  color: #121212;
  font-weight: 900;
}
.scope-toggle__button.active::after {
  width: 100%;
}
.pill-row__item.active {
  color: #ffffff;
  font-weight: 800;
  background: #15161b;
}
.pill-row {
  width: 100%;
  white-space: nowrap;
  overflow: hidden;
}
.pill-row__list {
  display: inline-flex;
  gap: 10rpx;
  min-width: max-content;
  padding: 0 8rpx 4rpx;
}
.pill-row__item {
  background: rgba(239, 240, 244, 0.92);
  border-radius: 999rpx;
  font-size: 24rpx;
  color: #696f7a;
}
.pill-row__item-text {
  display: block;
  text-align: center;
}
.ranking-surface {
  display: grid;
  gap: 22rpx;
  width: 100%;
  padding: 20rpx;
  overflow: hidden;
}
.ranking-list {
  display: grid;
  gap: 0;
  overflow: hidden;
}
.ranking-row {
  display: grid;
  grid-template-columns: 72rpx 68rpx minmax(0, 1fr) 112rpx;
  gap: 16rpx;
  align-items: center;
  width: 100%;
  overflow: hidden;
  padding: 16rpx 0;
  border-bottom: 2rpx solid #eff1f5;
}
.ranking-row--interactive {
  padding: 14rpx 12rpx;
  border-radius: 24rpx;
  transition: transform 180ms ease, background-color 180ms ease;
}
.ranking-row--pressed {
  transform: scale(0.99);
  background: rgba(243, 244, 247, 0.9);
}
.ranking-row__rank-wrap {
  display: flex;
  align-items: center;
  gap: 6rpx;
  flex-shrink: 0;
}
.ranking-row__rank {
  color: #8f9198;
  font-size: 24rpx;
  font-weight: 700;
  white-space: nowrap;
}
.ranking-row__rank--1 { color: #dc2626; }
.ranking-row__rank--2 { color: #2563eb; }
.ranking-row__rank--3 { color: #16a34a; }
.ranking-row__avatar {
  width: 68rpx;
  height: 68rpx;
  border-radius: 999rpx;
  background: #f5f6fa;
  flex-shrink: 0;
}
.ranking-row__avatar--player { border-radius: 999rpx; }
.ranking-row__body,
.ranking-row__metric {
  display: grid;
  min-width: 0;
}
.ranking-row__body {
  align-content: center;
}
.ranking-row__metric {
  justify-items: end;
  align-content: center;
  width: 112rpx;
  min-width: 112rpx;
  text-align: right;
  flex-shrink: 0;
  overflow: hidden;
}
.ranking-row__name {
  color: #121212;
  font-size: 30rpx;
  font-weight: 700;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.ranking-row__note,
.ranking-row__metric-note {
  color: #8f9198;
  font-size: 22rpx;
  white-space: nowrap;
}
.ranking-row__metric-value {
  color: #121212;
  font-size: 36rpx;
  font-weight: 800;
  white-space: nowrap;
}
.ranking-row:last-child {
  border-bottom: 0;
  padding-bottom: 0;
}
.ranking-surface .section-heading--compact {
  align-items: flex-start;
}
.ranking-surface .section-heading--compact > view:first-child {
  min-width: 0;
}
.ranking-surface .section-title {
  font-size: 50rpx;
}
.standings-sheet-mask {
  position: fixed;
  inset: 0;
  z-index: 20;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  padding: 28rpx;
  background: rgba(21, 22, 27, 0.32);
  animation: fi-overlay-fade-in 180ms ease both;
}
.standings-sheet {
  width: 100%;
  max-height: 76vh;
  overflow-y: auto;
  padding: 28rpx;
  border-radius: 32rpx;
  background: #ffffff;
  animation: fi-sheet-up 240ms cubic-bezier(0.22, 1, 0.36, 1) both;
}
.standings-sheet__actions {
  display: flex;
  align-items: center;
  gap: 12rpx;
}
.standings-sheet__close,
.standings-sheet__share {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  white-space: nowrap;
  line-height: 1;
  padding: 10rpx 18rpx;
  border-radius: 999rpx;
  background: #f6f7fb;
  font-size: 24rpx;
  color: #6d7280;
}
.standings-sheet__share {
  background: rgba(21, 22, 27, 0.92);
  color: #ffffff;
}
.standings-sheet__save {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  white-space: nowrap;
  line-height: 1;
  padding: 10rpx 18rpx;
  border-radius: 999rpx;
  background: rgba(255, 106, 0, 0.12);
  font-size: 24rpx;
  color: #ff6a00;
}
.standings-sheet__poster {
  margin-top: 18rpx;
}
.standings-sheet__poster-image {
  width: 100%;
  border-radius: 24rpx;
}
.standings-sheet__hint {
  display: block;
  margin-top: 14rpx;
  color: transparent;
  font-size: 0;
  line-height: 0;
}
.standings-sheet__hint--resolved {
  color: #767a84;
  font-size: 24rpx;
  line-height: 1.6;
}
.standings-sheet__error {
  margin-top: 18rpx;
}
.standings-sheet__list { margin-top: 18rpx; display: grid; gap: 12rpx; }
.standings-sheet__row {
  display: grid;
  grid-template-columns: 52rpx minmax(0, 1fr) auto;
  gap: 16rpx;
  align-items: center;
  padding: 18rpx 0;
  border-bottom: 2rpx solid #eff1f5;
}
.standings-sheet__rank, .standings-sheet__name, .standings-sheet__metric-value { font-size: 28rpx; }
.standings-sheet__rank { color: #8f9198; }
.standings-sheet__name { color: #121212; font-weight: 700; }
.standings-sheet__metrics {
  display: grid;
  grid-template-columns: repeat(3, 64rpx);
  gap: 10rpx;
  justify-content: end;
}
.standings-sheet__metric {
  display: grid;
  justify-items: end;
  gap: 4rpx;
  min-width: 0;
}
.standings-sheet__metric-value { color: #121212; font-weight: 800; line-height: 1; }
.standings-sheet__metric:first-child .standings-sheet__metric-value { color: #f97316; }
.standings-sheet__metric-label {
  color: #8f9198;
  font-size: 20rpx;
  line-height: 1;
  white-space: nowrap;
}
.standings-sheet--team-season {
  max-height: 82vh;
}
.team-season-sheet__title {
  display: flex;
  align-items: center;
  gap: 14rpx;
  min-width: 0;
  margin-top: 8rpx;
}
.team-season-sheet__title-avatar {
  width: 56rpx;
  height: 56rpx;
  flex: 0 0 auto;
}
.team-season-sheet__title-name {
  min-width: 0;
  margin-top: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.team-season-sheet__summary {
  margin-top: 18rpx;
  padding: 22rpx 24rpx;
  border-radius: 24rpx;
  border: 2rpx solid rgba(232, 233, 238, 0.95);
  background: #ffffff;
  display: block;
}
.team-season-sheet__summary-main {
  display: block;
  min-width: 0;
}
.team-season-sheet__summary-meta-grid {
  display: grid;
  grid-template-columns: minmax(0, 0.9fr) minmax(0, 1fr) minmax(160rpx, 1.45fr);
  align-items: center;
  gap: 12rpx;
  min-width: 0;
}
.team-season-sheet__summary-metric {
  min-width: 0;
  display: grid;
  gap: 6rpx;
  padding-left: 14rpx;
  border-left: 2rpx solid rgba(235, 236, 241, 0.95);
}
.team-season-sheet__summary-metric:first-child {
  border-left: 0;
  padding-left: 0;
}
.team-season-sheet__summary-value {
  color: #121212;
  font-size: 28rpx;
  line-height: 1.1;
  font-weight: 800;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.team-season-sheet__summary-label {
  color: #8f9198;
  font-size: 20rpx;
  line-height: 1.15;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.team-season-sheet__list {
  margin-top: 22rpx;
  max-height: 52vh;
}
.team-season-sheet__state,
.team-season-sheet__empty {
  margin-top: 22rpx;
}
.team-season-sheet__empty {
  padding: 28rpx 12rpx;
  color: #8f9198;
  font-size: 26rpx;
  text-align: center;
}
.team-season-match-row {
  padding: 20rpx 0;
  border-top: 2rpx solid #f0f1f5;
  display: grid;
  gap: 14rpx;
  opacity: 0;
  transform: translateY(14rpx);
  animation: team-season-row-enter 320ms cubic-bezier(0.24, 0.88, 0.28, 1) both;
  animation-delay: var(--team-match-delay, 100ms);
}
.team-season-match-row:first-child {
  border-top: none;
}
.team-season-match-row__meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16rpx;
  color: #8f9198;
  font-size: 22rpx;
}
.team-season-match-row__meta-right {
  display: inline-flex;
  align-items: center;
  gap: 8rpx;
}
.team-season-match-row__venue {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 56rpx;
  padding: 4rpx 12rpx;
  border-radius: 999rpx;
  font-size: 20rpx;
  font-weight: 800;
}
.team-season-match-row__venue--home {
  background: rgba(220, 38, 38, 0.12);
  color: #dc2626;
}
.team-season-match-row__venue--away {
  background: rgba(34, 197, 94, 0.12);
  color: #15803d;
}

.team-season-match-row__result {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 56rpx;
  padding: 4rpx 12rpx;
  border-radius: 999rpx;
  font-size: 20rpx;
  font-weight: 800;
}

.team-season-match-row__result--win {
  background: rgba(220, 38, 38, 0.12);
  color: #dc2626;
}

.team-season-match-row__result--draw {
  background: rgba(234, 179, 8, 0.12);
  color: #b45309;
}

.team-season-match-row__result--loss {
  background: rgba(34, 197, 94, 0.12);
  color: #15803d;
}

.team-season-match-row__result--live {
  background: rgba(249, 115, 22, 0.12);
  color: #d97706;
}

.team-season-match-row__result--scheduled {
  background: rgba(59, 130, 246, 0.12);
  color: #2563eb;
}

.team-season-match-row__body {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: center;
  gap: 18rpx;
}

.team-season-match-row__side {
  display: flex;
  align-items: center;
  gap: 12rpx;
}
.team-season-match-row__side--left {
  justify-content: flex-start;
}
.team-season-match-row__side--right {
  justify-content: flex-end;
}

.team-season-match-row__avatar {
  width: 36rpx;
  height: 36rpx;
  flex-shrink: 0;
}

.team-season-match-row__team {
  color: #7b818d;
  font-size: 28rpx;
  font-weight: 700;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.team-season-match-row__team--active {
  color: #121212;
}

.team-season-match-row__score {
  color: #121212;
  font-size: 40rpx;
  line-height: 1;
  font-weight: 800;
}
.standings-poster-canvas {
  position: fixed;
  left: -9999rpx;
  top: -9999rpx;
  width: 1080px;
  height: 1600px;
  pointer-events: none;
}
.state-card--error text { font-size: 28rpx; color: #c03a2b; }
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
  background: linear-gradient(180deg, transparent 45%, rgba(247,248,250,0.55) 78%, #f7f8fa 100%);
  pointer-events: none;
  z-index: 0;
}
@keyframes team-season-row-enter {
  from {
    opacity: 0;
    transform: translateY(14rpx);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
