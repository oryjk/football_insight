<template>
  <HomePanelCard>
    <template #heading>
      <view>
        <text class="section-kicker">实时脉冲</text>
        <view class="story-headline">
          <text>{{ headlineParts.leading }}</text>
          <text v-if="headlineParts.highlighted" class="inline-highlight">{{ headlineParts.highlighted }}</text>
          <text>{{ headlineParts.trailing }}</text>
        </view>
      </view>
    </template>

    <template #meta>
      <text class="meta-note">更新于 {{ updatedAtLabel }}</text>
    </template>

    <text class="story-copy">{{ body }}</text>

    <view v-if="pulseMatches.length" class="score-strip-stack">
      <view
        v-for="match in pulseMatches"
        :key="`pulse-${match.status}-${match.match_id}`"
        class="score-strip"
        :class="{ 'score-strip--interactive': hasTechStats(match) }"
        hover-class="score-strip--pressed"
        hover-stay-time="100"
        @click="openTechStats(match)"
      >
        <image class="score-strip__corner" :src="pulseMatchCornerImage" mode="aspectFill" />
        <view class="score-strip__meta">
          <text>第 {{ match.round_number }} 轮</text>
          <view class="score-strip__meta-trailing">
            <text>{{ match.match_date }} {{ match.match_time }}</text>
            <text v-if="hasTechStats(match)" class="score-strip__meta-pill">技术统计</text>
          </view>
        </view>
        <view class="score-strip__body">
          <view class="score-strip__team">
            <text class="score-strip__team-name inline-highlight">{{ match.home_team_name }}</text>
          </view>
          <text class="score-strip__score inline-highlight">{{ match.home_score }} : {{ match.away_score }}</text>
          <view class="score-strip__team score-strip__team--away">
            <text class="score-strip__team-name inline-highlight">{{ match.away_team_name }}</text>
          </view>
        </view>
        <view v-if="hasTechStats(match)" class="score-strip__hint">
          <text class="score-strip__hint-text">点击查看技术统计</text>
        </view>
      </view>
    </view>

    <view class="watch-list">
      <view v-for="item in watchPoints" :key="item" class="watch-list__item">
        <text>{{ item }}</text>
      </view>
    </view>
  </HomePanelCard>
</template>

<script setup lang="ts">
import HomePanelCard from './HomePanelCard.vue'
import pulseMatchCornerImage from '../../../static/home/pulse-match-corner.png'
import { resolveHomePulseTechStats, type HomePulseLeadMatch } from '../helpers'

defineProps<{
  headlineParts: { leading: string; highlighted: string; trailing: string }
  body: string
  updatedAtLabel: string
  pulseMatches: HomePulseLeadMatch[]
  watchPoints: string[]
}>()

const emit = defineEmits<{
  (e: 'open-tech-stats', match: HomePulseLeadMatch): void
}>()

function hasTechStats(match: HomePulseLeadMatch): boolean {
  return resolveHomePulseTechStats(match).length > 0
}

function openTechStats(match: HomePulseLeadMatch): void {
  emit('open-tech-stats', match)
}
</script>

<style scoped lang="css">
.section-kicker {
  display: block;
  margin: 0;
  color: var(--fi-color-text-muted);
  font-size: 22rpx;
  font-weight: 700;
  letter-spacing: 3rpx;
}

.story-headline {
  display: block;
  color: var(--fi-color-text-strong);
  font-size: 44rpx;
  line-height: 1.16;
  font-weight: 800;
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
  color: var(--fi-color-text-muted);
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
  background: #9aa0aa;
  box-shadow: 0 0 0 6rpx rgba(154, 160, 170, 0.12);
}

.story-copy {
  display: block;
  margin-top: 18rpx;
  color: #555863;
  font-size: 28rpx;
  line-height: 1.65;
}

.inline-highlight {
  display: inline;
  color: #17181d;
  font-weight: 700;
  padding: 0 0.08em;
  background-image: linear-gradient(90deg, rgba(var(--fi-primitive-red-rgb), 0.22), rgba(var(--fi-primitive-red-rgb), 0.22));
  background-repeat: no-repeat;
  background-size: 100% 0.36em;
  background-position: left bottom;
}

.score-strip-stack {
  margin-top: 16rpx;
  display: grid;
  gap: 12rpx;
}

.score-strip {
  position: relative;
  overflow: hidden;
  padding: 18rpx 16rpx;
  display: grid;
  gap: 10rpx;
  border-radius: 28rpx;
  border: 2rpx solid #ececf1;
  background-color: var(--fi-primitive-white);
}

.score-strip--interactive {
  border-color: rgba(var(--fi-primitive-red-rgb), 0.18);
}

.score-strip__corner {
  position: absolute;
  z-index: 0;
  top: -58rpx;
  left: -74rpx;
  width: 214rpx;
  height: 132rpx;
  opacity: 0.52;
  pointer-events: none;
  transform: scaleX(-1);
}

.score-strip > view,
.score-strip > text {
  position: relative;
  z-index: 1;
}

.score-strip--pressed {
  transform: scale(0.992);
}

.score-strip__meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 18rpx;
}

.score-strip__meta-trailing {
  display: inline-flex;
  align-items: center;
  justify-content: flex-end;
  gap: 12rpx;
  flex-wrap: wrap;
}

.score-strip__meta text {
  color: var(--fi-color-text-muted);
  font-size: 22rpx;
}

.score-strip__meta-pill {
  padding: 10rpx 16rpx;
  border-radius: 999rpx;
  background: rgba(var(--fi-primitive-red-rgb), 0.1);
  color: var(--fi-color-primary) !important;
  font-weight: 700;
}

.score-strip__body {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
  align-items: center;
  gap: 18rpx;
}

.score-strip__team {
  min-width: 0;
}

.score-strip__team--away {
  text-align: right;
}

.score-strip__team-name {
  display: block;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-size: 30rpx;
  font-weight: 700;
}

.score-strip__score {
  color: var(--fi-color-text-strong);
  font-size: 60rpx;
  line-height: 0.92;
  font-weight: 800;
}

.score-strip__hint {
  display: flex;
  justify-content: flex-end;
}

.score-strip__hint-text {
  color: var(--fi-color-primary);
  font-size: 22rpx;
  line-height: 1.2;
  font-weight: 700;
}

.watch-list {
  margin-top: 16rpx;
  display: grid;
  gap: 12rpx;
}

.watch-list__item {
  position: relative;
  padding-left: 26rpx;
  color: #151515;
  font-size: 26rpx;
  line-height: 1.55;
}

.watch-list__item::before {
  content: '';
  position: absolute;
  top: 14rpx;
  left: 0;
  width: 10rpx;
  height: 10rpx;
  border-radius: 999rpx;
  background: #131313;
}
</style>
