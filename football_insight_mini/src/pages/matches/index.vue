<template>
  <view class="page-root">
    <image class="page-bg-img" :src="bgImage" mode="aspectFill" />
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

      <FiLoading
        v-if="loading"
        title="赛季进度加载中"
        caption="轮次和即将到来的比赛正在整理。"
      />

      <view v-else-if="errorMessage" class="state-card state-card--error">
        <text>{{ errorMessage }}</text>
      </view>

      <template v-else>
        <view class="panel">
          <view class="section-heading">
            <view>
              <text class="section-kicker">Season Progress</text>
              <text class="section-title">赛季进度</text>
            </view>
            <text class="meta-note">{{ seasonProgressSummary.completedRounds }} / {{ seasonProgressSummary.totalRounds }} 轮已完赛</text>
          </view>

          <view class="season-progress">
            <view
              v-for="(row, rowIndex) in seasonProgressRows"
              :key="rowIndex"
              class="season-progress__row"
            >
              <view class="season-progress__track" />
              <view
                class="season-progress__fill"
                :style="{ width: row.fillWidth }"
              />
              <view
                v-for="round in row.rounds"
                :key="round.round_number"
                class="season-progress__item"
                :class="`season-progress__item--${round.status}`"
                @click="openRoundDialog(round.round_number)"
              >
                <button
                  class="season-progress__dot"
                  :class="[
                    `season-progress__dot--${round.status}`,
                    { 'season-progress__dot--selected': selectedRoundNumber === round.round_number },
                  ]"
                >
                  <view class="season-progress__dot-inner" />
                </button>
                <text class="season-progress__dot-number">{{ round.round_number }}</text>
              </view>
            </view>
          </view>

        </view>

        <view class="panel">
          <view class="section-heading">
            <view>
              <text class="section-kicker">Upcoming</text>
              <text class="section-title">即将到来</text>
            </view>
          </view>

          <view v-if="upcomingSections.length" class="upcoming-sections">
            <view v-for="section in upcomingSections" :key="section.title" class="upcoming-round">
              <view class="section-heading section-heading--compact">
                <view>
                  <text class="section-kicker">{{ section.title }}</text>
                  <text class="section-title">第 {{ section.roundNumber }} 轮</text>
                </view>
                <button class="meta-pill meta-pill--button" @click="openRoundDialog(section.roundNumber)">查看整轮</button>
              </view>

              <view class="match-stack">
                <view
                  v-for="match in section.matches"
                  :key="match.match_id"
                  class="match-card match-card--full"
                  :class="`match-card--${resolveDisplayStatus(match)}`"
                >
                  <view class="match-card__meta">
                    <text>{{ match.match_date }}</text>
                    <text>{{ match.match_time }}</text>
                  </view>

                  <view class="match-card__scoreboard">
                    <view class="match-card__team">
                      <image :src="match.home_team_avatar || ''" mode="aspectFit" class="match-card__team-logo" />
                      <text class="match-card__team-name">{{ match.home_team_name }}</text>
                    </view>
                    <view class="match-card__score-stack">
                      <text class="match-card__score" :class="{ 'match-card__score--upcoming': !shouldShowMatchScore(match) }">
                        {{ formatMatchScore(match) }}
                      </text>
                      <text v-if="shouldShowLiveStatus(match)" class="match-card__status-tag">进行中</text>
                    </view>
                    <view class="match-card__team match-card__team--away">
                      <text class="match-card__team-name">{{ match.away_team_name }}</text>
                      <image :src="match.away_team_avatar || ''" mode="aspectFit" class="match-card__team-logo" />
                    </view>
                  </view>

                  <view v-if="hasTechStats(match)" class="match-card__footer">
                    <view />
                    <button class="match-card__tech-link" @click="openMatchTechStats(match)">点击查看技术统计</button>
                  </view>
                </view>
              </view>
            </view>
          </view>

          <view v-else class="state-card state-card--empty">
            <text>当前没有可展示的即将到来比赛。</text>
          </view>
        </view>

        <template v-if="groupedMatches.length">
          <view
            v-for="group in groupedMatches"
            :key="group.roundNumber"
            class="panel"
          >
            <view class="section-heading section-heading--compact">
              <view>
                <text class="section-kicker">最近完赛</text>
                <text class="section-title">第 {{ group.roundNumber }} 轮</text>
              </view>
            </view>

            <view class="match-stack">
              <view
                v-for="match in group.items"
                :key="match.match_id"
                class="match-card match-card--full"
                :class="`match-card--${resolveDisplayStatus(match)}`"
              >
                <view class="match-card__meta">
                  <text>{{ match.match_date }}</text>
                  <text>{{ match.match_time }}</text>
                </view>

                <view class="match-card__scoreboard">
                  <view class="match-card__team">
                    <image :src="match.home_team_avatar || ''" mode="aspectFit" class="match-card__team-logo" />
                    <text class="match-card__team-name">{{ match.home_team_name }}</text>
                  </view>
                  <view class="match-card__score-stack">
                    <text class="match-card__score">{{ match.home_score }} : {{ match.away_score }}</text>
                    <text v-if="shouldShowLiveStatus(match)" class="match-card__status-tag">进行中</text>
                  </view>
                  <view class="match-card__team match-card__team--away">
                    <text class="match-card__team-name">{{ match.away_team_name }}</text>
                    <image :src="match.away_team_avatar || ''" mode="aspectFit" class="match-card__team-logo" />
                  </view>
                </view>

                <view v-if="hasTechStats(match)" class="match-card__footer">
                  <view />
                  <button class="match-card__tech-link" @click="openMatchTechStats(match)">点击查看技术统计</button>
                </view>
              </view>
            </view>
          </view>
        </template>
      </template>

      <view v-if="selectedRoundNumber !== null" class="schedule-dialog" @click.self="closeRoundDialog">
        <view class="schedule-dialog__sheet">
          <view class="section-heading section-heading--compact">
            <view>
              <text class="section-kicker">Round Fixtures</text>
              <text class="section-title">第 {{ selectedRoundNumber }} 轮对阵</text>
            </view>
            <button class="schedule-dialog__close" @click="closeRoundDialog">关闭</button>
          </view>

          <FiLoading
            v-if="roundDialogLoading"
            title="整轮对阵加载中"
            caption="这轮比赛马上整理好。"
          />

          <view v-else-if="roundDialogErrorMessage" class="state-card state-card--error">
            <text>{{ roundDialogErrorMessage }}</text>
          </view>

          <view v-else class="match-stack">
            <view
              v-for="match in selectedRoundMatches"
              :key="match.match_id"
              class="match-card match-card--full"
            >
              <view class="match-card__meta">
                <text>{{ match.match_date }}</text>
                <text>{{ match.match_time }}</text>
              </view>

              <view class="match-card__scoreboard">
                <view class="match-card__team">
                  <image :src="match.home_team_avatar || ''" mode="aspectFit" class="match-card__team-logo" />
                  <text class="match-card__team-name">{{ match.home_team_name }}</text>
                </view>
                <view class="match-card__score-stack">
                  <text class="match-card__score" :class="{ 'match-card__score--upcoming': !shouldShowMatchScore(match) }">
                    {{ formatMatchScore(match) }}
                  </text>
                  <text v-if="shouldShowLiveStatus(match)" class="match-card__status-tag">进行中</text>
                </view>
                <view class="match-card__team match-card__team--away">
                  <text class="match-card__team-name">{{ match.away_team_name }}</text>
                  <image :src="match.away_team_avatar || ''" mode="aspectFit" class="match-card__team-logo" />
                </view>
              </view>

              <view v-if="hasTechStats(match)" class="match-card__footer">
                <view />
                <button class="match-card__tech-link" @click="openMatchTechStats(match)">点击查看技术统计</button>
              </view>
            </view>
          </view>
        </view>
      </view>

      <view
        v-if="selectedTechStatsMatch"
        class="sheet-mask sheet-mask--tech-stats"
        @click.self="closeMatchTechStats"
      >
        <view class="sheet-card tech-stats-sheet">
          <view class="section-heading section-heading--compact">
            <view>
              <text class="section-kicker">比赛技术统计</text>
              <text class="section-title">技术统计</text>
            </view>
            <button class="tech-stats-sheet__close" @click="closeMatchTechStats">关闭</button>
          </view>

          <view class="tech-stats-sheet__summary">
            <text class="tech-stats-sheet__teams">
              {{ selectedTechStatsMatch.home_team_name }} {{ selectedTechStatsMatch.home_score }} : {{ selectedTechStatsMatch.away_score }} {{ selectedTechStatsMatch.away_team_name }}
            </text>
            <text class="tech-stats-sheet__meta">
              第 {{ selectedTechStatsMatch.round_number }} 轮 · {{ selectedTechStatsMatch.match_date }} {{ selectedTechStatsMatch.match_time }}
            </text>
          </view>

          <view class="tech-stats-sheet__list">
            <view
              v-for="(stat, index) in selectedTechStatsRows"
              :key="stat.key"
              class="tech-stat-row"
              :style="getTechStatRowStyle(index)"
            >
              <text class="tech-stat-row__value">{{ stat.homeValue }}</text>
              <view class="tech-stat-row__track tech-stat-row__track--home">
                <view
                  class="tech-stat-row__fill tech-stat-row__fill--home"
                  :style="{ width: `${stat.homeBarPercent}%` }"
                />
              </view>
              <text class="tech-stat-row__label">{{ stat.label }}</text>
              <view class="tech-stat-row__track tech-stat-row__track--away">
                <view
                  class="tech-stat-row__fill tech-stat-row__fill--away"
                  :style="{ width: `${stat.awayBarPercent}%` }"
                />
              </view>
              <text class="tech-stat-row__value tech-stat-row__value--away">{{ stat.awayValue }}</text>
            </view>
          </view>

          <text class="tech-stats-sheet__footnote">当前展示雷速提供的比赛技术统计，后续会继续补充更多指标。</text>
        </view>
      </view>
    </view>
  </scroll-view>
  </view>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { onShareAppMessage, onShow } from '@dcloudio/uni-app'
import FiLoading from '../../components/FiLoading.vue'
import bgImage from '../../static/matches/bg.jpg'
import { getAvailableRounds, getMatches } from '../../api/insight'
import type { MatchCard, RoundReference } from '../../types/insight'
import { extractApiErrorMessage } from '../../utils/apiError'
import {
  buildCurrentRoundSectionTitle,
  formatMatchScoreboardText,
  hasMatchTechStats,
  resolveMatchTechStats,
  resolveMatchDisplayStatus,
  shouldShowLiveStatusTag,
} from './helpers'

interface UpcomingSection {
  title: string
  roundNumber: number
  matches: MatchCard[]
}

const season = new Date().getFullYear()
const loading = ref(true)
const roundDialogLoading = ref(false)
const errorMessage = ref('')
const roundDialogErrorMessage = ref('')
const rounds = ref<RoundReference[]>([])
const matches = ref<MatchCard[]>([])
const selectedRoundNumber = ref<number | null>(null)
const selectedRoundMatches = ref<MatchCard[]>([])
const selectedTechStatsMatch = ref<MatchCard | null>(null)
const pageNowIso = ref(new Date().toISOString())
const selectedTechStatsRows = computed(() =>
  selectedTechStatsMatch.value ? resolveMatchTechStats(selectedTechStatsMatch.value) : [],
)

onShareAppMessage(() => ({
  title: '中超赛程和最近赛果，看看下一场对阵',
  path: '/pages/matches/index',
}))

const groupedMatches = computed(() => {
  const groups = new Map<number, MatchCard[]>()

  matches.value
    .filter((match) => resolveDisplayStatus(match) === 'finished')
    .forEach((match) => {
      const list = groups.get(match.round_number) ?? []
      list.push(match)
      groups.set(match.round_number, list)
    })

  return Array.from(groups.entries())
    .sort((left, right) => right[0] - left[0])
    .map(([roundNumber, items]) => ({
      roundNumber,
      items,
    }))
})

const seasonProgressSummary = computed(() => {
  const completedRounds = rounds.value.filter((round) => round.status === 'completed').length
  const currentRound = rounds.value.find((round) => round.status === 'current') ?? null

  return {
    completedRounds,
    totalRounds: rounds.value.length,
    currentRoundNumber: currentRound?.round_number ?? null,
  }
})

const seasonProgressFillWidth = computed(() => {
  const total = seasonProgressSummary.value.totalRounds
  if (total <= 1) return '0%'
  const completed = seasonProgressSummary.value.completedRounds
  return `${(completed / total) * 100}%`
})

const seasonProgressRows = computed(() => {
  const perRow = 15
  const rows: { rounds: RoundReference[]; fillWidth: string }[] = []
  for (let i = 0; i < rounds.value.length; i += perRow) {
    const rowRounds = rounds.value.slice(i, i + perRow)
    const completedInRow = rowRounds.filter((r) => r.status === 'completed' || r.status === 'current').length
    const fillWidth = rowRounds.length > 0 ? `${(completedInRow / rowRounds.length) * 100}%` : '0%'
    rows.push({ rounds: rowRounds, fillWidth })
  }
  return rows
})

const upcomingSections = computed<UpcomingSection[]>(() => {
  const currentRound = rounds.value.find((round) => round.status === 'current') ?? null

  if (!currentRound) {
    const firstUpcoming = rounds.value.find((round) => round.status === 'upcoming') ?? null
    if (!firstUpcoming) {
      return []
    }

    return [
      {
        title: '下一轮',
        roundNumber: firstUpcoming.round_number,
        matches: matchesForRound(firstUpcoming.round_number).filter((match) => resolveDisplayStatus(match) !== 'finished'),
      },
    ].filter((section) => section.matches.length > 0)
  }

  const pendingCurrentRoundMatches = matchesForRound(currentRound.round_number)
    .filter((match) => resolveDisplayStatus(match) !== 'finished')
  if (pendingCurrentRoundMatches.length > 0) {
    return [
      {
        title: buildCurrentRoundSectionTitle(
          pendingCurrentRoundMatches.map((match) => ({ status: resolveDisplayStatus(match) })),
        ),
        roundNumber: currentRound.round_number,
        matches: pendingCurrentRoundMatches,
      },
    ]
  }

  const nextRound = rounds.value.find((round) => round.round_number > currentRound.round_number) ?? null
  if (!nextRound) {
    return []
  }

  return [
    {
      title: '下一轮',
      roundNumber: nextRound.round_number,
      matches: matchesForRound(nextRound.round_number).filter((match) => resolveDisplayStatus(match) !== 'finished'),
    },
  ].filter((section) => section.matches.length > 0)
})

function matchesForRound(roundNumber: number): MatchCard[] {
  return matches.value.filter((match) => match.round_number === roundNumber)
}

function resolveDisplayStatus(match: MatchCard): MatchCard['status'] {
  return resolveMatchDisplayStatus(match, pageNowIso.value)
}

function formatMatchScore(match: MatchCard): string {
  return formatMatchScoreboardText(match, pageNowIso.value)
}

function shouldShowMatchScore(match: MatchCard): boolean {
  return formatMatchScore(match).includes(' : ')
}

function shouldShowLiveStatus(match: MatchCard): boolean {
  return shouldShowLiveStatusTag(match, pageNowIso.value)
}

function hasTechStats(match: MatchCard): boolean {
  return hasMatchTechStats(match)
}

function openMatchTechStats(match: MatchCard): void {
  if (!hasTechStats(match)) {
    uni.showToast({ title: '这场比赛暂时还没有技术统计', icon: 'none' })
    return
  }

  selectedTechStatsMatch.value = match
}

function closeMatchTechStats(): void {
  selectedTechStatsMatch.value = null
}

function getTechStatRowStyle(index: number) {
  return {
    '--tech-stat-delay': `${120 + index * 70}ms`,
  }
}

async function loadPage(): Promise<void> {
  loading.value = true
  errorMessage.value = ''
  pageNowIso.value = new Date().toISOString()

  try {
    const [roundsResponse, liveMatchesResponse] = await Promise.all([
      getAvailableRounds(season),
      getMatches({ mode: 'live', season, roundNumber: null }),
    ])

    rounds.value = roundsResponse
    matches.value = liveMatchesResponse.matches

    const futureRoundNumbers = roundsResponse
      .filter((round) => round.status === 'current' || round.status === 'upcoming')
      .map((round) => round.round_number)
      .slice(0, 2)

    if (futureRoundNumbers.length) {
      const futureResponses = await Promise.all(
        futureRoundNumbers.map((roundNumber) =>
          getMatches({ mode: 'round', season, roundNumber }),
        ),
      )

      const merged = new Map<number, MatchCard>()
      liveMatchesResponse.matches.forEach((match) => merged.set(match.match_id, match))
      futureResponses.forEach((response) => {
        response.matches.forEach((match) => merged.set(match.match_id, match))
      })
      matches.value = Array.from(merged.values())
    }
  } catch (error) {
    errorMessage.value = extractApiErrorMessage(error, '赛程加载失败，请稍后重试。')
  } finally {
    loading.value = false
  }
}

async function openRoundDialog(roundNumber: number): Promise<void> {
  selectedRoundNumber.value = roundNumber
  roundDialogLoading.value = true
  roundDialogErrorMessage.value = ''

  try {
    const response = await getMatches({ mode: 'round', season, roundNumber })
    selectedRoundMatches.value = response.matches
  } catch (error) {
    roundDialogErrorMessage.value = extractApiErrorMessage(error, '这轮对阵暂时没有加载出来，请稍后重试。')
  } finally {
    roundDialogLoading.value = false
  }
}

function closeRoundDialog(): void {
  selectedRoundNumber.value = null
  selectedRoundMatches.value = []
  roundDialogErrorMessage.value = ''
}

onShow(() => {
  pageNowIso.value = new Date().toISOString()
  void loadPage()
})
</script>

<style scoped lang="css">
.page-root { position: relative; }
.page-scroll { height: 100vh; position: relative; z-index: 1; }

@keyframes dot-blink {
  0%, 100% { opacity: 1; transform: scale(1.3); }
  50% { opacity: 0.35; transform: scale(1.05); }
}

@keyframes dot-blink-strong {
  0%, 100% { opacity: 1; transform: scale(1.5); }
  50% { opacity: 0.35; transform: scale(1.15); }
}

.page {
  position: relative;
  padding: 24rpx 16rpx 40rpx;
  display: flex;
  flex-direction: column;
  gap: 16rpx;
}
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
.hero-card, .panel, .state-card {
  position: relative;
  z-index: 1;
  background: rgba(255,255,255,0.72);
  border-radius: 36rpx;
  padding: 20rpx;
  border: 2rpx solid rgba(255,255,255,0.55);
  box-shadow: 0 20rpx 48rpx rgba(26,28,36,0.06);
  backdrop-filter: blur(18rpx);
  -webkit-backdrop-filter: blur(18rpx);
}
.schedule-dialog__sheet {
  background: rgba(255,255,255,0.94);
  border-radius: 36rpx;
  padding: 28rpx;
  border: 2rpx solid rgba(236, 236, 241, 0.95);
  box-shadow: 0 28rpx 60rpx rgba(26,28,36,0.08);
}
.hero-card__top, .section-heading, .match-card__meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.hero-card__top, .section-heading { align-items: flex-start; gap: 12rpx; }
.eyebrow, .section-kicker {
  margin: 0;
  color: #8f9198;
  font-size: 22rpx;
  font-weight: 700;
  letter-spacing: 3rpx;
}
.hero-card__title, .section-title {
  display: block;
  margin-top: 10rpx;
  color: #2a2c31;
  font-size: 48rpx;
  line-height: 1.08;
  font-weight: 800;
}
.section-title { font-size: 44rpx; }
.hero-card__summary {
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
.meta-pill--button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  white-space: nowrap;
  line-height: 1;
}
.season-progress {
  display: flex;
  flex-direction: column;
  gap: 32rpx;
  margin-top: 22rpx;
  padding: 8rpx 0 28rpx;
}
.season-progress__row {
  position: relative;
  display: flex;
  justify-content: space-between;
  gap: 8rpx;
  padding: 8rpx 0;
}
.season-progress__track {
  position: absolute;
  top: 18rpx;
  left: 0;
  right: 0;
  height: 8rpx;
  border-radius: 999rpx;
  background: #e6eaf1;
  z-index: 0;
}
.season-progress__fill {
  position: absolute;
  top: 18rpx;
  left: 0;
  height: 8rpx;
  border-radius: 999rpx;
  background: linear-gradient(90deg, #15161b, #3a3d47);
  z-index: 1;
  transition: width 600ms cubic-bezier(0.22, 1, 0.36, 1);
}
.season-progress__item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8rpx;
  z-index: 2;
  flex: 1;
  min-width: 0;
}
.season-progress__dot {
  width: 28rpx;
  height: 28rpx;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
}
.season-progress__dot-inner {
  width: 22rpx;
  height: 22rpx;
  border-radius: 50%;
  border: none;
  background: transparent;
  background-image: url("data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCA0MCA0MCI+PGNpcmNsZSBjeD0iMjAiIGN5PSIyMCIgcj0iMTguNSIgZmlsbD0iI2ZmZmZmZiIgc3Ryb2tlPSIjZDBkNGRjIiBzdHJva2Utd2lkdGg9IjEuNSIvPjxwYXRoIGQ9Ik0yMCwxMS41IEwyOC4xLDE3LjQgTDI1LDI2LjkgTDE1LDI2LjkgTDExLjksMTcuNCBaIiBmaWxsPSIjZDBkNGRjIi8+PHBhdGggZD0iTTIwLDExLjUgTDIwLDEuNSBNMjguMSwxNy40IEwzNy42LDE0LjMgTTI1LDI2LjkgTDMwLjksMzUgTTE1LDI2LjkgTDkuMSwzNSBNMTEuOSwxNy40IEwyLjQsMTQuMyIgc3Ryb2tlPSIjZDBkNGRjIiBzdHJva2Utd2lkdGg9IjIiIGZpbGw9Im5vbmUiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIvPjxwYXRoIGQ9Ik0yMCwxLjUgUTMwLDAgMzcuNiwxNC4zIE0zNy42LDE0LjMgUTQwLDI1IDMwLjksMzUgTTMwLjksMzUgUTIwLDQwIDkuMSwzNSBNOS4xLDM1IFEwLDI1IDIuNCwxNC4zIE0yLjQsMTQuMyBRMTAsMCAyMCwxLjUiIHN0cm9rZT0iI2QwZDRkYyIgc3Ryb2tlLXdpZHRoPSIxLjUiIGZpbGw9Im5vbmUiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgb3BhY2l0eT0iMC40Ii8+PC9zdmc+Cg==");
  background-size: cover;
  background-position: center;
  transition: all 200ms ease;
}
.season-progress__dot--completed .season-progress__dot-inner {
  background-image: url("data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCA0MCA0MCI+PGNpcmNsZSBjeD0iMjAiIGN5PSIyMCIgcj0iMTkiIGZpbGw9IiMxNTE2MWIiLz48cGF0aCBkPSJNMjAsMTEuNSBMMjguMSwxNy40IEwyNSwyNi45IEwxNSwyNi45IEwxMS45LDE3LjQgWiIgZmlsbD0iI2ZmZmZmZiIvPjxwYXRoIGQ9Ik0yMCwxMS41IEwyMCwxIE0yOC4xLDE3LjQgTDM4LjEsMTQuMSBNMjUsMjYuOSBMMzEuMiwzNS40IE0xNSwyNi45IEw4LjgsMzUuNCBNMTEuOSwxNy40IEwxLjksMTQuMSIgc3Ryb2tlPSIjZmZmZmZmIiBzdHJva2Utd2lkdGg9IjIuNSIgZmlsbD0ibm9uZSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIi8+PHBhdGggZD0iTTIwLDEgUTMwLDAgMzguMSwxNC4xIE0zOC4xLDE0LjEgUTQwLDI1IDMxLjIsMzUuNCBNMzEuMiwzNS40IFEyMCw0MCA4LjgsMzUuNCBNOC44LDM1LjQgUTAsMjUgMS45LDE0LjEgTTEuOSwxNC4xIFExMCwwIDIwLDEiIHN0cm9rZT0iI2ZmZmZmZiIgc3Ryb2tlLXdpZHRoPSIxLjUiIGZpbGw9Im5vbmUiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgb3BhY2l0eT0iMC40NSIvPjwvc3ZnPgo=");
}
.season-progress__dot--current .season-progress__dot-inner {
  background-image: url("data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCA0MCA0MCI+PGNpcmNsZSBjeD0iMjAiIGN5PSIyMCIgcj0iMTkiIGZpbGw9IiNjOTk2NDIiLz48cGF0aCBkPSJNMjAsMTEuNSBMMjguMSwxNy40IEwyNSwyNi45IEwxNSwyNi45IEwxMS45LDE3LjQgWiIgZmlsbD0iI2ZmZmZmZiIvPjxwYXRoIGQ9Ik0yMCwxMS41IEwyMCwxIE0yOC4xLDE3LjQgTDM4LjEsMTQuMSBNMjUsMjYuOSBMMzEuMiwzNS40IE0xNSwyNi45IEw4LjgsMzUuNCBNMTEuOSwxNy40IEwxLjksMTQuMSIgc3Ryb2tlPSIjZmZmZmZmIiBzdHJva2Utd2lkdGg9IjIuNSIgZmlsbD0ibm9uZSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIi8+PHBhdGggZD0iTTIwLDEgUTMwLDAgMzguMSwxNC4xIE0zOC4xLDE0LjEgUTQwLDI1IDMxLjIsMzUuNCBNMzEuMiwzNS40IFEyMCw0MCA4LjgsMzUuNCBNOC44LDM1LjQgUTAsMjUgMS45LDE0LjEgTTEuOSwxNC4xIFExMCwwIDIwLDEiIHN0cm9rZT0iI2ZmZmZmZiIgc3Ryb2tlLXdpZHRoPSIxLjUiIGZpbGw9Im5vbmUiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgb3BhY2l0eT0iMC40NSIvPjwvc3ZnPgo=");
  box-shadow: 0 2rpx 10rpx rgba(201, 150, 66, 0.35);
  animation: dot-blink 1.2s ease-in-out infinite;
}
.season-progress__dot--selected .season-progress__dot-inner {
  transform: scale(1.3);
}
.season-progress__dot--current.season-progress__dot--selected .season-progress__dot-inner {
  box-shadow: 0 4rpx 18rpx rgba(201, 150, 66, 0.55);
  animation: dot-blink-strong 1.2s ease-in-out infinite;
}
.season-progress__dot-number {
  color: #8f9198;
  font-size: 18rpx;
  font-weight: 700;
  line-height: 1.1;
  text-align: center;
}
.season-progress__item--completed .season-progress__dot-number {
  color: #15161b;
}
.season-progress__item--current .season-progress__dot-number {
  color: #c99642;
}
.season-progress__summary {
  display: block;
  margin-top: 18rpx;
  color: #767a84;
  font-size: 26rpx;
  line-height: 1.65;
}
.upcoming-sections, .match-stack {
  margin-top: 14rpx;
  display: grid;
  gap: 12rpx;
}
.match-card {
  position: relative;
  padding: 18rpx 18rpx 18rpx 22rpx;
  border-radius: 28rpx;
  border: 2rpx solid rgba(255,255,255,0.55);
  background: rgba(255,255,255,0.72);
  backdrop-filter: blur(14rpx);
  -webkit-backdrop-filter: blur(14rpx);
  box-shadow: 0 14rpx 36rpx rgba(26,28,36,0.05);
  overflow: hidden;
}
.match-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  width: 5rpx;
  height: 100%;
  background: #c4c8d0;
}
.match-card--finished::before {
  background: linear-gradient(180deg, #3d9c72, #2d8c63);
}
.match-card--live::before {
  background: linear-gradient(180deg, #ff9129, #ef7d16);
}
.match-card--live {
  box-shadow: 0 16rpx 40rpx rgba(255, 145, 41, 0.12);
}
.match-card--scheduled::before {
  background: linear-gradient(180deg, #c4c8d0, #a8adb8);
}
.match-card__meta text { color: #8f9198; font-size: 22rpx; }
.match-card__scoreboard {
  margin-top: 12rpx;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
  gap: 18rpx;
  align-items: center;
}
.match-card__team {
  display: flex;
  align-items: center;
  gap: 10rpx;
}
.match-card__team-logo {
  width: 56rpx;
  height: 56rpx;
  flex-shrink: 0;
}
.match-card__team-name {
  color: #151515;
  font-size: 28rpx;
  font-weight: 700;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.match-card__team--away {
  justify-content: flex-end;
}
.match-card__score-stack {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8rpx;
}
.match-card__score {
  color: #121212;
  font-size: 56rpx;
  font-weight: 800;
  line-height: 0.92;
}
.match-card__score--upcoming {
  font-size: 44rpx;
}
.match-card__status-tag {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 8rpx 16rpx;
  border-radius: 999rpx;
  background: rgba(255, 145, 41, 0.14);
  color: #df7616;
  font-size: 22rpx;
  line-height: 1;
}
.match-card__footer {
  margin-top: 14rpx;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16rpx;
}
.match-card__tech-link {
  padding: 0;
  border: none;
  background: transparent;
  color: #ef7d16;
  font-size: 24rpx;
  line-height: 1.2;
}
.schedule-dialog {
  position: fixed;
  inset: 0;
  z-index: 20;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  padding: 28rpx;
  background: rgba(21, 22, 27, 0.32);
}
.schedule-dialog__sheet {
  width: 100%;
  max-height: 76vh;
  overflow-y: auto;
}
.schedule-dialog__close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  white-space: nowrap;
  line-height: 1;
  padding: 10rpx 18rpx;
  border-radius: 999rpx;
  background: #f6f7fb;
  color: #6d7280;
  font-size: 24rpx;
}
.sheet-mask {
  position: fixed;
  inset: 0;
  z-index: 30;
  background: rgba(18, 20, 28, 0.36);
  backdrop-filter: blur(8rpx);
  display: flex;
  align-items: flex-end;
}
.sheet-mask--tech-stats {
  animation: tech-stats-mask-fade 220ms ease-out both;
}
.sheet-card {
  width: 100%;
  max-height: 78vh;
  border-radius: 36rpx 36rpx 0 0;
  background: rgba(255,255,255,0.98);
  padding: 28rpx 24rpx 40rpx;
  box-shadow: 0 -24rpx 56rpx rgba(12,14,20,0.12);
}
.tech-stats-sheet {
  max-height: 72vh;
  transform-origin: center bottom;
  animation: tech-stats-sheet-enter 280ms cubic-bezier(0.2, 0.9, 0.22, 1) both;
}
.tech-stats-sheet__close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  white-space: nowrap;
  line-height: 1;
  padding: 12rpx 18rpx;
  border-radius: 999rpx;
  background: #f6f7fb;
  color: #6d7280;
  font-size: 24rpx;
}
.tech-stats-sheet__summary {
  margin-top: 18rpx;
  padding: 24rpx;
  border-radius: 28rpx;
  border: 2rpx solid rgba(255, 145, 41, 0.16);
  background:
    radial-gradient(circle at top right, rgba(255, 145, 41, 0.12), transparent 36%),
    linear-gradient(180deg, rgba(255,255,255,0.98), rgba(250,246,239,0.92));
}
.tech-stats-sheet__teams {
  display: block;
  color: #121212;
  font-size: 32rpx;
  line-height: 1.4;
  font-weight: 800;
}
.tech-stats-sheet__meta {
  display: block;
  margin-top: 10rpx;
  color: #8f9198;
  font-size: 24rpx;
}
.tech-stats-sheet__list {
  margin-top: 24rpx;
  display: grid;
  gap: 18rpx;
}
.tech-stat-row {
  display: grid;
  grid-template-columns: 44rpx minmax(0, 1fr) auto minmax(0, 1fr) 44rpx;
  align-items: center;
  gap: 16rpx;
  padding: 18rpx 0;
  border-top: 2rpx solid #f0f1f5;
  opacity: 0;
  transform: translateY(14rpx);
  animation: tech-stat-row-enter 320ms cubic-bezier(0.24, 0.88, 0.28, 1) both;
  animation-delay: var(--tech-stat-delay, 120ms);
}
.tech-stat-row:first-child {
  border-top: none;
}
.tech-stat-row__value {
  color: #121212;
  font-size: 28rpx;
  font-weight: 800;
  text-align: left;
}
.tech-stat-row__value--away {
  text-align: right;
}
.tech-stat-row__label {
  min-width: 84rpx;
  color: #2a2c31;
  font-size: 30rpx;
  font-weight: 800;
  text-align: center;
}
.tech-stat-row__track {
  height: 16rpx;
  border-radius: 999rpx;
  background: #16171c;
  overflow: hidden;
  display: flex;
  align-items: center;
}
.tech-stat-row__track--home {
  justify-content: flex-end;
}
.tech-stat-row__fill {
  height: 100%;
  border-radius: 999rpx;
  background: linear-gradient(90deg, #ff8b2b, #f59e0b);
  transform: scaleX(0);
  animation: tech-stat-fill-grow 480ms cubic-bezier(0.22, 1, 0.36, 1) forwards;
  animation-delay: calc(var(--tech-stat-delay, 120ms) + 70ms);
}
.tech-stat-row__fill--home {
  background: linear-gradient(90deg, #f6b14b, #f08a12);
  transform-origin: right center;
}
.tech-stat-row__fill--away {
  background: linear-gradient(90deg, #f08a12, #f6b14b);
  transform-origin: left center;
}
.tech-stats-sheet__footnote {
  display: block;
  margin-top: 24rpx;
  color: #8f9198;
  font-size: 24rpx;
  line-height: 1.6;
}
.state-card--error text { font-size: 28rpx; color: #c03a2b; }
.state-card--empty {
  padding: 22rpx;
  margin-top: 14rpx;
}
.state-card--empty text { color: #767a84; font-size: 26rpx; }

@keyframes tech-stats-mask-fade {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes tech-stats-sheet-enter {
  from {
    opacity: 0;
    transform: translateY(32rpx) scale(0.98);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

@keyframes tech-stat-row-enter {
  from {
    opacity: 0;
    transform: translateY(14rpx);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes tech-stat-fill-grow {
  from {
    transform: scaleX(0);
  }
  to {
    transform: scaleX(1);
  }
}
</style>
