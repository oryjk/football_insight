<template>
  <view class="hero-card">
    <image class="hero-dots hero-dots--top" :src="memberCardDotsImage" mode="aspectFill" />
    <image class="hero-dots hero-dots--bottom" :src="memberCardDotsImage" mode="aspectFill" />
    <view class="hero-card__top">
      <view class="hero-card__heading">
        <text class="eyebrow">Football Insight</text>
        <text class="hero-card__title">这一轮之后，谁在改变联赛格局</text>
      </view>
    </view>

    <view class="hero-card__guide">
      <view class="hero-card__guide-title">先看这三件事</view>
      <view class="hero-card__guide-copy">
        <template v-if="heroGuide.mode === 'team-and-scorer-with-match'">
          <text>先看 </text>
          <text class="inline-highlight">{{ heroGuide.topTeamName }}</text>
          <text> 能否守住榜首，再看 </text>
          <text class="inline-highlight">{{ heroGuide.topScorerName }}</text>
          <text> 是否继续领跑射手榜，最后看最新完赛如何继续改变联赛格局。</text>
        </template>
        <template v-else-if="heroGuide.mode === 'team-and-scorer-with-live-match'">
          <text>先看 </text>
          <text class="inline-highlight">{{ heroGuide.topTeamName }}</text>
          <text> 的榜首走势，再看 </text>
          <text class="inline-highlight">{{ heroGuide.topScorerName }}</text>
          <text> 是否继续领跑射手榜，最后盯住正在进行中的比分变化。</text>
        </template>
        <template v-else-if="heroGuide.mode === 'team-and-scorer'">
          <text>先看 </text>
          <text class="inline-highlight">{{ heroGuide.topTeamName }}</text>
          <text> 的榜首走势，再看 </text>
          <text class="inline-highlight">{{ heroGuide.topScorerName }}</text>
          <text> 领衔的射手竞争。</text>
        </template>
        <template v-else>
          <text>先看榜首走势，再看射手竞争，最后看最近哪场比赛最值得继续跟进。</text>
        </template>
      </view>
      <text v-if="heroGuideNote" class="hero-card__guide-note">{{ heroGuideNote }}</text>
    </view>

    <view v-if="isBriefingReady" class="briefing-grid">
      <view
        v-for="item in briefingItems"
        :key="item.label"
        class="briefing-card"
        :class="`briefing-card--${item.accent}`"
      >
        <text class="briefing-card__label">{{ item.label }}</text>

        <view class="briefing-card__body">
          <view class="briefing-card__main">
            <view v-if="item.accent === 'leader' && item.avatars[0]?.src" class="briefing-card__leader-logo">
              <image
                :src="item.avatars[0].src"
                :alt="item.avatars[0].name"
                mode="aspectFit"
                class="briefing-card__leader-logo-image"
              />
            </view>

            <view v-else-if="item.accent === 'leader' && item.avatars.length" class="briefing-card__entity-group">
              <image
                v-for="avatar in item.avatars.slice(0, 3)"
                :key="`${item.label}-${avatar.name}`"
                :src="avatar.src || ''"
                :alt="avatar.name"
                mode="aspectFill"
                class="briefing-card__entity-avatar"
              />
            </view>

            <view v-if="item.entities.length" class="briefing-card__entity-list">
              <view
                v-for="entity in item.entities"
                :key="`${item.label}-${entity.name}-${entity.caption}`"
                class="briefing-card__entity-row"
              >
                <image
                  :src="entity.avatar || ''"
                  :alt="entity.name"
                  mode="aspectFill"
                  class="briefing-card__entity-row-avatar"
                />
                <view class="briefing-card__entity-row-body">
                  <text class="briefing-card__entity-row-name">{{ entity.name }}</text>
                  <text class="briefing-card__entity-row-team">{{ entity.caption }}</text>
                </view>
              </view>
            </view>

            <view v-else class="briefing-card__title-block">
              <text class="briefing-card__value">{{ item.value }}</text>
              <text v-if="item.subValue" class="briefing-card__subvalue">{{ item.subValue }}</text>
            </view>
          </view>

          <view v-if="marqueeRows(item.accent).length" class="briefing-card__marquees">
            <view
              v-for="(row, rowIndex) in marqueeRows(item.accent)"
              :key="`${item.label}-marquee-${rowIndex}`"
              class="briefing-card__marquee"
            >
              <view class="briefing-card__marquee-track" :style="{ animationDuration: `${15 + rowIndex * 2}s` }">
                <text
                  v-for="(message, messageIndex) in [...row, ...row]"
                  :key="`${item.label}-marquee-${rowIndex}-${messageIndex}`"
                  class="briefing-card__marquee-item"
                >
                  {{ message }}
                </text>
              </view>
            </view>
          </view>

          <view class="briefing-card__metric">
            <text class="briefing-card__metric-value">{{ item.metricValue }}</text>
            <text class="briefing-card__metric-label">{{ item.metricLabel }}</text>
          </view>
        </view>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import memberCardDotsImage from '../../../static/user/member-card-dots.png'
import type { HomeBriefingMarqueeAccent } from '../../../utils/homeBriefingMarquees'
import type { HomeBriefingItem, HomeHeroGuide } from '../helpers'

const props = defineProps<{
  heroGuide: HomeHeroGuide
  heroGuideNote: string
  isBriefingReady: boolean
  briefingItems: HomeBriefingItem[]
  briefingMarqueeRows: Partial<Record<HomeBriefingMarqueeAccent, string[][]>>
}>()

function marqueeRows(accent: HomeBriefingMarqueeAccent): string[][] {
  return props.briefingMarqueeRows[accent] ?? []
}
</script>

<style scoped lang="css">
.hero-card {
  position: relative;
  overflow: hidden;
  padding: 26rpx 24rpx 20rpx;
  border-radius: 36rpx;
  background: rgba(255, 255, 255, 0.96);
  border: 2rpx solid rgba(229, 231, 236, 0.98);
  box-shadow: 0 18rpx 46rpx rgba(26, 28, 36, 0.07);
}

.hero-dots {
  position: absolute;
  z-index: 0;
  width: 292rpx;
  height: 180rpx;
  opacity: 0.34;
  pointer-events: none;
}

.hero-dots--top {
  top: -54rpx;
  right: -54rpx;
}

.hero-dots--bottom {
  left: -86rpx;
  bottom: 136rpx;
  opacity: 0.18;
  transform: scaleX(-1);
}

.hero-card__top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16rpx;
  position: relative;
  z-index: 1;
}

.hero-card__heading {
  display: flex;
  flex-direction: column;
  gap: 10rpx;
  flex: 1;
  min-width: 0;
}

.eyebrow {
  display: block;
  margin: 0;
  color: var(--fi-color-text-muted);
  font-size: 22rpx;
  font-weight: 700;
  letter-spacing: 3rpx;
}

.hero-card__title {
  color: #17181d;
  max-width: none;
  font-size: 46rpx;
  line-height: 1.12;
  letter-spacing: 0;
  font-weight: 800;
}

.hero-card__guide {
  position: relative;
  z-index: 1;
  margin-top: 24rpx;
  padding: 22rpx 0 0;
  border-top: 2rpx solid rgba(231, 232, 238, 0.94);
}

.hero-card__guide-title {
  color: #191b20;
  font-size: 30rpx;
  font-weight: 800;
  display: flex;
  align-items: center;
  gap: 12rpx;
}

.hero-card__guide-title::before {
  content: '';
  width: 8rpx;
  height: 8rpx;
  background: var(--fi-color-primary);
  border-radius: 999rpx;
  box-shadow: 0 0 0 8rpx rgba(var(--fi-primitive-red-rgb), 0.12);
}

.hero-card__guide-copy {
  margin-top: 12rpx;
  color: #606672;
  font-size: 27rpx;
  line-height: 1.62;
}

.hero-card__guide-note {
  margin-top: 10rpx;
  color: var(--fi-color-text-muted);
  font-size: 22rpx;
  line-height: 1.55;
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

.briefing-grid {
  position: relative;
  z-index: 1;
  margin-top: 22rpx;
  display: grid;
  gap: 0;
  border-top: 2rpx solid rgba(231, 232, 238, 0.88);
}

.briefing-card {
  position: relative;
  overflow: visible;
  min-height: 150rpx;
  padding: 20rpx 0 20rpx 20rpx;
  border-radius: 0;
  border: 0;
  border-bottom: 2rpx solid rgba(231, 232, 238, 0.88);
  background: transparent;
}

.briefing-card::before {
  content: '';
  position: absolute;
  left: 0;
  top: 26rpx;
  bottom: 26rpx;
  width: 4rpx;
  border-radius: 999rpx;
}

.briefing-card--leader::before {
  background: var(--fi-color-primary-deep);
}

.briefing-card--scorer::before {
  background: var(--fi-color-primary);
}

.briefing-card--assist::before {
  background: #ef4444;
}

.briefing-card__label {
  color: #8a8f9a;
  font-size: 22rpx;
  font-weight: 700;
  line-height: 1;
}

.briefing-card__body {
  margin-top: 14rpx;
  display: grid;
  grid-template-columns: minmax(210rpx, 0.95fr) minmax(0, 1.35fr) 120rpx;
  column-gap: 16rpx;
  align-items: center;
}

.briefing-card__main {
  display: flex;
  align-items: center;
  gap: 14rpx;
  min-width: 0;
}

.briefing-card__leader-logo,
.briefing-card__entity-group {
  flex: 0 0 auto;
}

.briefing-card__leader-logo {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 60rpx;
  height: 60rpx;
  border-radius: 18rpx;
  background: var(--fi-color-page-soft);
  border: 2rpx solid rgba(231, 232, 238, 0.92);
}

.briefing-card__leader-logo-image {
  width: 48rpx;
  height: 48rpx;
}

.briefing-card__entity-group {
  display: flex;
  align-items: center;
  min-width: 78rpx;
}

.briefing-card__entity-avatar {
  width: 54rpx;
  height: 54rpx;
  border-radius: 999rpx;
  margin-right: -14rpx;
  border: 4rpx solid rgba(255, 255, 255, 0.96);
  background: var(--fi-color-page-soft);
  box-shadow: 0 8rpx 18rpx rgba(24, 27, 33, 0.07);
}

.briefing-card__entity-avatar:last-child {
  margin-right: 0;
}

.briefing-card__title-block {
  display: grid;
  gap: 5rpx;
  min-width: 0;
}

.briefing-card__entity-list {
  display: grid;
  gap: 12rpx;
  min-width: 0;
  width: 100%;
}

.briefing-card__entity-row {
  display: flex;
  align-items: center;
  min-width: 0;
  gap: 12rpx;
}

.briefing-card__entity-row-avatar {
  flex: 0 0 auto;
  width: 48rpx;
  height: 48rpx;
  border-radius: 999rpx;
  background: var(--fi-color-page-soft);
}

.briefing-card__entity-row-body {
  display: grid;
  gap: 4rpx;
  min-width: 0;
}

.briefing-card__entity-row-name {
  color: #17181d;
  font-size: 28rpx;
  line-height: 1.12;
  font-weight: 800;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.briefing-card__entity-row-team {
  color: #8a8f9a;
  font-size: 20rpx;
  line-height: 1.16;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.briefing-card__value {
  color: #17181d;
  font-size: 30rpx;
  line-height: 1.12;
  font-weight: 800;
}

.briefing-card__subvalue {
  color: #8a8f9a;
  font-size: 21rpx;
  line-height: 1.22;
}

.briefing-card__marquees {
  display: grid;
  align-self: stretch;
  align-content: center;
  gap: 6rpx;
  overflow: hidden;
  min-width: 0;
  padding-right: 0;
}

.briefing-card__marquee {
  overflow: hidden;
  white-space: nowrap;
}

.briefing-card__marquee-track {
  display: inline-flex;
  align-items: center;
  gap: 20rpx;
  min-width: max-content;
  animation-name: briefing-marquee-scroll;
  animation-timing-function: linear;
  animation-iteration-count: infinite;
}

.briefing-card__marquee-item {
  position: relative;
  padding-left: 16rpx;
  color: rgba(86, 91, 101, 0.88);
  font-size: 21rpx;
  line-height: 1.3;
}

.briefing-card__marquee-item::before {
  content: '•';
  position: absolute;
  left: 0;
  color: rgba(var(--fi-primitive-red-rgb), 0.9);
  font-size: 24rpx;
}

.briefing-card__metric {
  display: grid;
  width: 120rpx;
  align-content: center;
  justify-items: end;
  justify-self: end;
  align-self: stretch;
  text-align: right;
  gap: 8rpx;
}

.briefing-card__metric-value {
  color: var(--fi-color-primary);
  font-size: 50rpx;
  line-height: 0.95;
  font-weight: 800;
}

.briefing-card__metric-label {
  color: #31343b;
  font-size: 22rpx;
  font-weight: 700;
  line-height: 1.18;
}

@keyframes briefing-marquee-scroll {
  from {
    transform: translateX(0);
  }

  to {
    transform: translateX(-50%);
  }
}
</style>
