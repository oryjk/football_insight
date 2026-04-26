<template>
  <view class="page-root">
    <image class="page-bg-img" :src="bgImage" mode="aspectFill" />
    <view class="page-bg-fade"></view>
    <scroll-view scroll-y class="page-scroll">
      <view class="page">
      <view class="hero-card">
        <view class="hero-card__top">
          <view>
            <text class="eyebrow">Rankings</text>
            <text class="hero-card__title">榜单不是表格，而是联赛秩序</text>
          </view>
          <text class="meta-note meta-note--hero">球队榜 / 球员榜</text>
        </view>

        <text class="hero-card__summary">
          榜单里的进球、助攻、射门和传球等数据，展示的都是抓取时点的赛季累计值，不是单轮统计。
        </text>
      </view>

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
          <view class="section-heading section-heading--compact">
            <view>
              <text class="section-kicker">积分榜总览</text>
              <text class="section-title">完整积分榜单独看</text>
            </view>
            <text class="meta-note">独立图片入口</text>
          </view>

          <text class="standings-launcher__intro">
            这里展示的是完整积分榜图片入口，不受下方球队榜、球员榜和分类切换控制。
          </text>

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
                <button class="standings-launcher-card__info" @click.stop="toggleStandingsInfo(table.slug)">i</button>
              </view>

              <view v-if="activeStandingsInfoSlug === table.slug" class="standings-launcher-card__popover">
                <text>{{ getStandingsPreviewSummary(table) }}</text>
                <button class="standings-launcher-card__popover-close" @click.stop="activeStandingsInfoSlug = null">关闭</button>
              </view>
            </view>
          </view>
        </view>

        <view class="panel rankings-controls">
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

        <view v-if="scope === 'team' && activeTeamCategory" class="panel ranking-surface">
          <view class="ranking-list">
            <view class="section-heading section-heading--compact">
              <view>
                <text class="section-kicker">{{ activeTeamSectionKicker }}</text>
                <text class="section-title">{{ activeTeamCategory.label }}</text>
              </view>
            </view>

            <view
              v-for="entry in activeTeamCategory.entries.slice(0, 12)"
              :key="`${activeTeamCategory.slug}-${entry.team_id}`"
              class="ranking-row ranking-row--interactive"
              hover-class="ranking-row--pressed"
              hover-stay-time="100"
              @click="openRankingTeamSheet(entry)"
            >
              <text class="ranking-row__rank" :class="`ranking-row__rank--${entry.rank_no}`">#{{ entry.rank_no }}</text>
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
              v-for="entry in activePlayerCategory.entries.slice(0, 12)"
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
              <text class="section-title">{{ selectedRankingTeam.team.team_name }}</text>
            </view>
            <button class="standings-sheet__close" @click="closeRankingTeamSheet">关闭</button>
          </view>

          <view class="team-season-sheet__summary">
            <view class="team-season-sheet__summary-main">
              <image
                :src="selectedRankingTeamAvatar"
                mode="aspectFit"
                class="team-season-sheet__summary-avatar"
              />
              <view class="team-season-sheet__summary-copy">
                <text class="team-season-sheet__summary-name">{{ selectedRankingTeam.team.team_name }}</text>
                <text class="team-season-sheet__summary-meta">{{ selectedRankingTeamStandingsMeta }}</text>
                <text class="team-season-sheet__summary-note">{{ selectedRankingTeamCategoryMeta }}</text>
              </view>
            </view>
            <text class="team-season-sheet__summary-record">{{ selectedRankingTeamRecord }}</text>
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
                <text class="team-season-match-row__result" :class="`team-season-match-row__result--${match.resultTone}`">
                  {{ match.resultLabel }}
                </text>
              </view>
              <view class="team-season-match-row__body">
                <text class="team-season-match-row__team" :class="{ 'team-season-match-row__team--active': match.isHomeTeam }">
                  {{ match.homeTeamName }}
                </text>
                <text class="team-season-match-row__score">{{ match.scoreText }}</text>
                <text class="team-season-match-row__team team-season-match-row__team--away" :class="{ 'team-season-match-row__team--active': !match.isHomeTeam }">
                  {{ match.awayTeamName }}
                </text>
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
              <text class="standings-sheet__points">{{ getDisplayedPoints(selectedStandingsTable, entry) }}</text>
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
import { onShareAppMessage, onShow } from '@dcloudio/uni-app'
import FiLoading from '../../components/FiLoading.vue'
import { getAvailableRounds, getMatches, getRankings } from '../../api/insight'
import type { MatchCard, PlayerRankingCategory, RankingsViewResponse, RoundReference, StandingsTable, StandingsTableEntry, TeamRankingCategory, TeamRankingEntry } from '../../types/insight'
import { extractApiErrorMessage } from '../../utils/apiError'
import { type TeamSeasonMatch, resolveTeamSeasonMatches } from '../../utils/teamSeasonMatches'
import bgImage from '../../static/rankings/bg.jpg'
import { buildStandingsPosterColumns, buildStandingsPosterMetrics, buildStandingsPosterTeamLayout } from './poster'
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
const loading = ref(true)
const errorMessage = ref('')
const rankings = ref<RankingsViewResponse | null>(null)
const activeCategorySlug = ref('')
const categoryScrollLeft = ref(0)
const selectedStandingsPosterSlug = ref<string | null>(null)
const activeStandingsInfoSlug = ref<string | null>(null)
const posterImagePath = ref('')
const posterGenerating = ref(false)
const posterErrorMessage = ref('')
const posterCache = new Map<string, string>()
const posterLogoCache = new Map<string, string | null>()
const rounds = ref<RoundReference[]>([])
const allSeasonMatches = ref<MatchCard[] | null>(null)
const teamSeasonMatchesLoading = ref(false)
const teamSeasonMatchesErrorMessage = ref('')

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
const selectedStandingsTable = computed(() =>
  previewStandingsTables.value.find((item) => item.slug === selectedStandingsPosterSlug.value) ?? null,
)
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
const selectedRankingTeamStandingsMeta = computed(() => {
  if (selectedRankingStandingsEntry.value) {
    return `当前积分榜第 ${selectedRankingStandingsEntry.value.rank_no} · ${selectedRankingStandingsEntry.value.points} 分`
  }

  return '积分榜位置待同步'
})
const selectedRankingTeamCategoryMeta = computed(() => {
  if (!selectedRankingTeam.value) {
    return ''
  }

  return `当前${selectedRankingTeam.value.categoryLabel}第 ${selectedRankingTeam.value.team.rank_no} · ${selectedRankingTeam.value.team.score_value} ${selectedRankingTeam.value.metricLabel}`
})
const selectedRankingTeamRecord = computed(() => {
  const finishedMatches = selectedRankingTeamMatches.value.filter((match) =>
    match.resultTone === 'win' || match.resultTone === 'draw' || match.resultTone === 'loss',
  )
  const wins = finishedMatches.filter((match) => match.resultTone === 'win').length
  const draws = finishedMatches.filter((match) => match.resultTone === 'draw').length
  const losses = finishedMatches.filter((match) => match.resultTone === 'loss').length

  return `已赛 ${finishedMatches.length} 场 · ${wins}胜 ${draws}平 ${losses}负`
})

onShareAppMessage(() => ({
  title: activeTeamCategory.value
    ? `中超${activeTeamCategory.value.label}榜，看看现在谁在前面`
    : '榜单不是表格，而是联赛秩序',
  path: '/pages/rankings/index',
}))

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
  activeStandingsInfoSlug.value = null
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

function closeStandingsSheet(): void {
  selectedStandingsPosterSlug.value = null
  posterErrorMessage.value = ''
}

function toggleStandingsInfo(slug: string): void {
  activeStandingsInfoSlug.value = activeStandingsInfoSlug.value === slug ? null : slug
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
    const context = uni.createCanvasContext(canvasId)
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
      })
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
</script>

<style scoped lang="css">
.page-root { position: relative; }
.page-scroll { height: 100vh; position: relative; z-index: 1; }
.page { padding: 24rpx 16rpx 40rpx; display: flex; flex-direction: column; gap: 16rpx; }
.hero-card, .panel, .state-card {
  background: rgba(255,255,255,0.94);
  border-radius: 36rpx;
  padding: 20rpx;
  border: 2rpx solid rgba(236, 236, 241, 0.95);
  box-shadow: 0 20rpx 48rpx rgba(26,28,36,0.06);
}
.hero-card__top, .section-heading, .standings-launcher-card__header, .standings-launcher-card__footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.hero-card__top, .section-heading, .standings-launcher-card__header { align-items: flex-start; gap: 12rpx; }
.eyebrow, .section-kicker {
  margin: 0;
  color: #8f9198;
  font-size: 22rpx;
  font-weight: 700;
  letter-spacing: 3rpx;
}
.hero-card__title, .section-title, .standings-launcher-card__title {
  display: block;
  margin-top: 10rpx;
  color: #2a2c31;
  font-size: 48rpx;
  line-height: 1.08;
  font-weight: 800;
}
.section-title, .standings-launcher-card__title { font-size: 44rpx; }
.hero-card__summary, .standings-launcher__intro, .standings-launcher-card__summary {
  display: block;
  margin-top: 18rpx;
  color: #6b707b;
  font-size: 28rpx;
  line-height: 1.7;
}
.hero-card__badge, .meta-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  white-space: nowrap;
  line-height: 1;
  box-sizing: border-box;
  padding: 14rpx 24rpx;
  border-radius: 999rpx;
  border: 2rpx solid rgba(229, 223, 205, 0.92);
  background: linear-gradient(180deg, rgba(252, 250, 245, 0.98), rgba(247, 243, 232, 0.94));
  color: #93876a;
  font-size: 24rpx;
}
.meta-note {
  display: inline-flex;
  align-items: center;
  justify-content: flex-end;
  flex: 0 0 auto;
  flex-shrink: 0;
  max-width: 100%;
  white-space: nowrap;
  line-height: 1;
  color: #a3916f;
  font-size: 24rpx;
  font-weight: 700;
  letter-spacing: 1rpx;
  padding: 8rpx 0 0;
}
.meta-note::before {
  content: '';
  width: 12rpx;
  height: 12rpx;
  margin-right: 10rpx;
  border-radius: 999rpx;
  background: linear-gradient(180deg, rgba(214, 184, 131, 0.9), rgba(197, 163, 103, 0.72));
  box-shadow: 0 0 0 6rpx rgba(214, 184, 131, 0.14);
}
.meta-note--hero { padding-top: 14rpx; }
.standings-launcher { display: grid; gap: 14rpx; }
.standings-launcher__grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12rpx; }
.standings-launcher-card {
  position: relative;
  display: grid;
  gap: 12rpx;
  padding: 22rpx;
  border-radius: 24rpx;
  border: 2rpx solid rgba(230, 232, 239, 0.9);
  background: linear-gradient(180deg, rgba(252, 252, 255, 0.98), rgba(248, 249, 252, 0.96));
}
.standings-launcher-card__footer, .standings-launcher-card__summary { font-size: 22rpx; color: #757986; }
.standings-launcher-card__info {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 40rpx;
  height: 40rpx;
  border-radius: 999rpx;
  background: #f6f7fb;
  color: #6d7280;
  font-size: 24rpx;
  line-height: 1;
}
.standings-launcher-card__popover {
  position: absolute;
  right: 18rpx;
  bottom: 72rpx;
  width: 320rpx;
  padding: 20rpx;
  border-radius: 22rpx;
  background: rgba(255,255,255,0.98);
  box-shadow: 0 20rpx 44rpx rgba(18,18,18,0.12);
  display: grid;
  gap: 12rpx;
}
.standings-launcher-card__popover text, .standings-launcher-card__popover-close {
  font-size: 22rpx;
  color: #565b67;
  line-height: 1.6;
}
.standings-launcher-card__popover-close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  white-space: nowrap;
  line-height: 1;
  justify-self: flex-end;
  padding: 10rpx 16rpx;
  border-radius: 999rpx;
  background: #f6f7fb;
}
.rankings-controls {
  display: grid;
  gap: 24rpx;
  width: 100%;
  padding: 20rpx;
  overflow: hidden;
}
.scope-toggle {
  display: flex;
  align-items: stretch;
  width: 100%;
  gap: 8rpx;
  padding: 8rpx;
  border: 2rpx solid #ececf1;
  border-radius: 24rpx;
  background: #f6f7fb;
  overflow: hidden;
}
.scope-toggle__button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: 1 1 0;
  min-width: 0;
  min-height: 76rpx;
  border-radius: 18rpx;
  padding: 18rpx 22rpx;
  text-align: center;
  background: transparent;
  color: #8f9198;
  font-size: 26rpx;
  white-space: nowrap;
  line-height: 1;
  overflow: hidden;
  box-sizing: border-box;
}
.scope-toggle__button-text {
  display: block;
  width: 100%;
  text-align: center;
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
.scope-toggle__button.active, .pill-row__item.active {
  color: #121212;
  font-weight: 600;
  background: #ffffff;
  box-shadow: 0 8rpx 18rpx rgba(18, 18, 18, 0.06);
}
.pill-row {
  width: 100%;
  white-space: nowrap;
  overflow: hidden;
}
.pill-row__list {
  display: inline-flex;
  gap: 12rpx;
  min-width: max-content;
  padding-bottom: 4rpx;
}
.pill-row__item {
  background: #f1f2f6;
  border-radius: 999rpx;
  font-size: 24rpx;
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
.ranking-row__rank {
  width: 72rpx;
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
.ranking-surface .section-heading {
  justify-content: flex-start;
}
.ranking-surface .section-heading--compact {
  align-items: flex-start;
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
.standings-sheet__close {
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
  grid-template-columns: 52rpx 1fr auto;
  gap: 16rpx;
  align-items: center;
  padding: 18rpx 0;
  border-bottom: 2rpx solid #eff1f5;
}
.standings-sheet__rank, .standings-sheet__name, .standings-sheet__points { font-size: 28rpx; }
.standings-sheet__rank { color: #8f9198; }
.standings-sheet__name { color: #121212; font-weight: 700; }
.standings-sheet__points { color: #f97316; font-weight: 800; }
.standings-sheet--team-season {
  max-height: 82vh;
}
.team-season-sheet__summary {
  margin-top: 18rpx;
  padding: 24rpx;
  border-radius: 28rpx;
  border: 2rpx solid rgba(255, 145, 41, 0.16);
  background:
    radial-gradient(circle at top right, rgba(255, 145, 41, 0.12), transparent 36%),
    linear-gradient(180deg, rgba(255,255,255,0.98), rgba(250,246,239,0.92));
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20rpx;
}
.team-season-sheet__summary-main {
  display: flex;
  align-items: center;
  gap: 18rpx;
  min-width: 0;
}
.team-season-sheet__summary-avatar {
  width: 82rpx;
  height: 82rpx;
  flex: 0 0 auto;
}
.team-season-sheet__summary-copy {
  min-width: 0;
  display: grid;
  gap: 8rpx;
}
.team-season-sheet__summary-name {
  color: #121212;
  font-size: 32rpx;
  line-height: 1.2;
  font-weight: 800;
}
.team-season-sheet__summary-meta,
.team-season-sheet__summary-record,
.team-season-sheet__summary-note {
  color: #8f9198;
  font-size: 24rpx;
}
.team-season-sheet__summary-note {
  color: #9c7e45;
}
.team-season-sheet__summary-record {
  flex: 0 0 auto;
  text-align: right;
  font-weight: 700;
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
.team-season-match-row__result {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 96rpx;
  padding: 8rpx 16rpx;
  border-radius: 999rpx;
  font-size: 22rpx;
  font-weight: 800;
}
.team-season-match-row__result--win {
  background: rgba(34, 197, 94, 0.12);
  color: #15803d;
}
.team-season-match-row__result--draw {
  background: rgba(148, 163, 184, 0.16);
  color: #475569;
}
.team-season-match-row__result--loss {
  background: rgba(239, 68, 68, 0.12);
  color: #b91c1c;
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
  grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
  align-items: center;
  gap: 18rpx;
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
.team-season-match-row__team--away {
  text-align: right;
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
