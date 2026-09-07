<template>
  <view class="standings-sheet-mask" @click.self="emit('close')" @touchmove.stop.prevent>
    <view class="standings-sheet standings-sheet--team-season" @click.stop @touchmove.stop>
      <view class="section-heading">
        <view>
          <text class="section-kicker">球队赛季战绩</text>
          <view class="team-season-sheet__title">
            <image :src="avatar" mode="aspectFit" class="team-season-sheet__title-avatar" />
            <text class="section-title team-season-sheet__title-name">{{ team.team_name }}</text>
          </view>
        </view>
        <button class="standings-sheet__close" @click="emit('close')">关闭</button>
      </view>

      <view class="team-season-sheet__summary">
        <view class="team-season-sheet__summary-main">
          <view class="team-season-sheet__summary-meta-grid">
            <view class="team-season-sheet__summary-metric">
              <text class="team-season-sheet__summary-value">{{ standingRankText }}</text>
              <text class="team-season-sheet__summary-label">积分榜排名</text>
            </view>
            <view class="team-season-sheet__summary-metric">
              <text class="team-season-sheet__summary-value">{{ categoryScoreText }}</text>
              <text class="team-season-sheet__summary-label">{{ categoryLabelText }}</text>
            </view>
            <view class="team-season-sheet__summary-metric">
              <text class="team-season-sheet__summary-value">{{ recordText }}</text>
              <text class="team-season-sheet__summary-label">赛季战绩</text>
            </view>
          </view>
        </view>
      </view>

      <view class="team-season-ability">
        <view class="team-season-ability__heading">
          <text class="team-season-ability__title">六维能力</text>
          <text class="team-season-ability__caption">按全联盟最大值折算，满分 100</text>
        </view>
        <view class="team-season-ability__body">
          <view class="team-season-ability__chart">
            <TeamAbilityHexagon :values="abilityValues" :size="200" show-labels />
          </view>
          <view class="team-season-ability__legend">
            <view
              v-for="(axis, index) in abilityAxes"
              :key="axis"
              class="team-season-ability__legend-item"
            >
              <text class="team-season-ability__legend-dot" :class="`team-season-ability__legend-dot--${index + 1}`"></text>
              <text class="team-season-ability__legend-label">{{ axis }}</text>
              <text class="team-season-ability__legend-value">{{ formatAbilityScore(abilityValues[index]) }}</text>
            </view>
          </view>
        </view>
      </view>

      <view v-if="loading" class="team-season-sheet__loading">
        <FiLoading
          title="赛季战绩加载中"
          caption="正在整理这支球队本赛季的每场比赛。"
        />
      </view>

      <view v-else-if="errorMessage" class="state-card state-card--error team-season-sheet__state">
        <text>{{ errorMessage }}</text>
      </view>

      <scroll-view
        v-else-if="matches.length"
        scroll-y
        class="team-season-sheet__list"
        :scroll-top="scrollTop"
        :scroll-with-animation="scrollAnimated"
      >
        <view
          v-for="(match, index) in matches"
          :key="match.matchId"
          :id="buildTeamSeasonMatchRowId(match.matchId)"
          class="team-season-match-row"
          :class="[
            match.isHomeTeam ? 'team-season-match-row--home' : 'team-season-match-row--away',
            match.focusKind ? `team-season-match-row--${match.focusKind}` : '',
          ]"
          :style="{ '--team-match-delay': `${100 + index * 55}ms` }"
        >
          <text
            v-if="match.focusKind"
            class="team-season-match-row__focus"
            :class="`team-season-match-row__focus--${match.focusKind}`"
          >
            {{ match.focusKind === 'latest-finished' ? '刚赛完' : '下一场' }}
          </text>
          <view class="team-season-match-row__meta">
            <view class="team-season-match-row__meta-left">
              <text>第 {{ match.roundNumber }} 轮 · {{ match.matchDate }} {{ match.matchTime }}</text>
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
</template>

<script setup lang="ts">
import { computed, getCurrentInstance, nextTick, ref, watch } from 'vue'
import FiLoading from '../../../components/FiLoading.vue'
import TeamAbilityHexagon from './TeamAbilityHexagon.vue'
import type { TeamRankingEntry } from '../../../types/insight'
import { TEAM_ABILITY_AXES } from '../ability'
import { buildTeamSeasonMatchRowId, formatTeamSeasonRecord, type TeamSeasonMatch } from '../../../utils/teamSeasonMatches'

const props = defineProps<{
  team: TeamRankingEntry
  avatar: string
  standingRankText: string
  categoryScoreText: string
  categoryLabelText: string
  matches: TeamSeasonMatch[]
  abilityValues: number[]
  loading: boolean
  errorMessage: string
}>()

const abilityAxes = TEAM_ABILITY_AXES

function formatAbilityScore(value: number | undefined): string {
  return `${Math.round((value ?? 0) * 100)}`
}

const emit = defineEmits<{
  (e: 'close'): void
}>()

const instance = getCurrentInstance()
const scrollTop = ref(0)
// 首次自动定位"下一场"时不做滚动动画，避免弹框打开瞬间画面跳动；之后再开启动画。
const scrollAnimated = ref(false)

const recordText = computed(() => formatTeamSeasonRecord(props.matches))
const nextScheduledMatch = computed(() =>
  props.matches.find((match) => match.focusKind === 'next-scheduled') ?? null,
)

watch(
  nextScheduledMatch,
  async (match) => {
    if (!match) {
      return
    }

    await nextTick()
    centerMatch(match.matchId)
  },
  { flush: 'post' },
)

function hasVerticalRectShape(value: unknown): value is { top: number; height: number } {
  return !!value && typeof value === 'object'
    && typeof (value as { top?: unknown }).top === 'number'
    && typeof (value as { height?: unknown }).height === 'number'
}

function hasScrollTop(value: unknown): value is { scrollTop: number } {
  return !!value && typeof value === 'object'
    && typeof (value as { scrollTop?: unknown }).scrollTop === 'number'
}

async function centerMatch(matchId: number): Promise<void> {
  if (!instance) {
    return
  }

  await nextTick()

  const query = uni.createSelectorQuery().in(instance)
  query.select('.team-season-sheet__list').boundingClientRect()
  query.select('.team-season-sheet__list').scrollOffset(() => {})
  query.select(`#${buildTeamSeasonMatchRowId(matchId)}`).boundingClientRect()
  query.exec((result) => {
    const [rawScrollRect, rawScrollOffset, rawMatchRect] = (result ?? []) as unknown[]

    if (!hasVerticalRectShape(rawScrollRect) || !hasScrollTop(rawScrollOffset) || !hasVerticalRectShape(rawMatchRect)) {
      return
    }

    const verticalDelta = (rawMatchRect.top + rawMatchRect.height / 2)
      - (rawScrollRect.top + rawScrollRect.height / 2)
    const nextScrollTop = Math.max(0, Math.round(rawScrollOffset.scrollTop + verticalDelta))

    if (nextScrollTop !== scrollTop.value) {
      scrollTop.value = nextScrollTop
    }

    // 首次定位完成后再允许滚动动画
    scrollAnimated.value = true
  })
}
</script>

<style scoped lang="css">
.state-card--error text {
  font-size: var(--fi-font-28);
  color: #c03a2b;
}

.standings-sheet-mask {
  position: fixed;
  inset: 0;
  z-index: 20;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  padding: var(--fi-space-28);
  background: rgba(21, 22, 27, 0.32);
  animation: fi-overlay-fade-in 180ms ease both;
}

.standings-sheet {
  width: 100%;
  max-height: 76vh;
  overflow-y: auto;
  padding: var(--fi-space-28);
  border-radius: 32rpx;
  background: var(--fi-primitive-white);
  animation: fi-sheet-up 240ms cubic-bezier(0.22, 1, 0.36, 1) both;
}

.standings-sheet--team-season {
  max-height: 82vh;
}

.standings-sheet__close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  white-space: nowrap;
  line-height: var(--fi-leading-none);
  padding: var(--fi-space-10) var(--fi-space-18);
  border-radius: var(--fi-radius-round);
  background: var(--fi-component-close-bg);
  font-size: var(--fi-component-close-size);
  color: var(--fi-component-close-text);
  margin: 0 0 0 auto;
}

.section-heading {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--fi-space-12);
}

.section-kicker {
  margin: 0;
  color: var(--fi-component-kicker-text);
  font-size: var(--fi-component-kicker-size);
  font-weight: var(--fi-weight-bold);
  letter-spacing: 3rpx;
}

.section-title {
  display: block;
  color: var(--fi-color-text-strong);
  font-size: 44rpx;
  line-height: 1.08;
  font-weight: var(--fi-weight-extrabold);
}

.team-season-sheet__title {
  display: flex;
  align-items: center;
  gap: var(--fi-space-14);
  min-width: 0;
  margin-top: var(--fi-space-8);
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
  margin-top: var(--fi-space-18);
  padding: var(--fi-space-22) var(--fi-space-24);
  border-radius: var(--fi-radius-md);
  border: var(--fi-primitive-border-width) solid var(--fi-color-border-chip);
  background: var(--fi-primitive-white);
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
  gap: var(--fi-space-12);
  min-width: 0;
}

.team-season-sheet__summary-metric {
  min-width: 0;
  display: grid;
  gap: var(--fi-space-6);
  padding-left: var(--fi-space-14);
  border-left: var(--fi-primitive-border-width) solid rgba(235, 236, 241, 0.95);
}

.team-season-sheet__summary-metric:first-child {
  border-left: 0;
  padding-left: 0;
}

.team-season-sheet__summary-value {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-28);
  line-height: var(--fi-leading-tight);
  font-weight: var(--fi-weight-extrabold);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.team-season-sheet__summary-label {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-20);
  line-height: 1.15;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.team-season-ability {
  margin-top: var(--fi-space-18);
  padding: var(--fi-space-22) var(--fi-space-24);
  border-radius: var(--fi-radius-md);
  border: var(--fi-primitive-border-width) solid var(--fi-color-border-chip);
  background: linear-gradient(160deg, rgba(250, 247, 240, 0.9), rgba(255, 255, 255, 0.98));
  display: grid;
  gap: var(--fi-space-18);
}

.team-season-ability__heading {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--fi-space-12);
}

.team-season-ability__title {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-none);
}

.team-season-ability__caption {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-20);
  line-height: var(--fi-leading-none);
}

.team-season-ability__body {
  display: flex;
  align-items: center;
  gap: var(--fi-space-24);
}

/* 给顶点标注预留空间，避免左右两侧文字被裁切 */
.team-season-ability__chart {
  padding: 34rpx 62rpx;
  flex-shrink: 0;
}

.team-season-ability__legend {
  flex: 1;
  min-width: 0;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--fi-space-14) var(--fi-space-16);
}

.team-season-ability__legend-item {
  display: flex;
  align-items: center;
  gap: var(--fi-space-8);
  min-width: 0;
}

.team-season-ability__legend-dot {
  width: 12rpx;
  height: 12rpx;
  border-radius: 50%;
  flex-shrink: 0;
  background: rgba(var(--fi-primitive-red-rgb, 214, 48, 49), 0.85);
}

.team-season-ability__legend-dot--2 { background: #e89b0c; }
.team-season-ability__legend-dot--3 { background: #2563eb; }
.team-season-ability__legend-dot--4 { background: #16a34a; }
.team-season-ability__legend-dot--5 { background: #7c5cbf; }
.team-season-ability__legend-dot--6 { background: #0e9aa7; }

.team-season-ability__legend-label {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-none);
  white-space: nowrap;
}

.team-season-ability__legend-value {
  margin-left: auto;
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-none);
}

/* 加载态预留与赛程列表相同的高度，避免数据到达时弹框高度突变造成抖动 */
.team-season-sheet__loading {
  margin-top: var(--fi-space-22);
  min-height: 52vh;
  display: flex;
  align-items: center;
  justify-content: center;
}

.team-season-sheet__list {
  margin-top: var(--fi-space-22);
  max-height: 52vh;
}

.team-season-sheet__state,
.team-season-sheet__empty {
  margin-top: var(--fi-space-22);
}

.team-season-sheet__empty {
  padding: var(--fi-space-28) 12rpx;
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-26);
  text-align: center;
}

.team-season-match-row {
  position: relative;
  margin-top: var(--fi-space-10);
  padding: var(--fi-space-20) var(--fi-space-18);
  border-top: var(--fi-primitive-border-width) solid transparent;
  border-radius: var(--fi-radius-sm);
  display: grid;
  gap: var(--fi-space-14);
  opacity: 0;
  transform: translateY(14rpx);
  animation: team-season-row-enter 320ms cubic-bezier(0.24, 0.88, 0.28, 1) both;
  animation-delay: var(--team-match-delay, 100ms);
}

.team-season-match-row:first-child {
  border-top: none;
}

.team-season-match-row--home {
  background: linear-gradient(135deg, #7f1d1d, #b91c1c);
  box-shadow: 0 12rpx 26rpx rgba(127, 29, 29, 0.18);
}

.team-season-match-row--away {
  background: linear-gradient(135deg, #14532d, #16a34a);
  box-shadow: 0 12rpx 26rpx rgba(20, 83, 45, 0.18);
}

.team-season-match-row--latest-finished,
.team-season-match-row--next-scheduled {
  box-shadow: 0 14rpx 30rpx rgba(17, 24, 39, 0.22), inset 0 0 0 3rpx rgba(255, 255, 255, 0.28);
}

.team-season-match-row__meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-16);
  color: #fff;
  font-size: var(--fi-font-22);
}

.team-season-match-row__meta-left {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: var(--fi-space-10);
  padding-right: 128rpx;
  overflow: hidden;
  white-space: nowrap;
}

.team-season-match-row__focus {
  position: absolute;
  right: var(--fi-space-18);
  top: -2rpx;
  z-index: 1;
  min-width: 108rpx;
  padding: var(--fi-space-8) var(--fi-space-18) var(--fi-space-8) var(--fi-space-16);
  border-radius: 0 0 16rpx 16rpx;
  font-size: var(--fi-font-20);
  line-height: 1.1;
  font-weight: var(--fi-weight-black);
  text-align: center;
  box-shadow: 0 8rpx 18rpx rgba(17, 24, 39, 0.2);
}

.team-season-match-row__focus--latest-finished {
  background: #2563eb;
  color: #fff;
}

.team-season-match-row__focus--next-scheduled {
  background: #fff;
  color: #14532d;
}

.team-season-match-row__body {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: center;
  gap: var(--fi-space-18);
}

.team-season-match-row__side {
  display: flex;
  align-items: center;
  gap: var(--fi-space-12);
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
  color: #fff;
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-bold);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.team-season-match-row__team--active {
  color: #fff;
}

.team-season-match-row__score {
  color: #fff;
  font-size: 40rpx;
  line-height: var(--fi-leading-none);
  font-weight: var(--fi-weight-extrabold);
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
