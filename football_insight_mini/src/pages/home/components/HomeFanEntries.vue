<template>
  <view class="fan-entries">
    <view
      class="fan-entry fan-entry--lineup"
      role="button"
      :aria-label="`${lineupModel.title}：${lineupModel.subtitle}`"
      hover-class="fan-entry--pressed"
      :hover-stay-time="80"
      @tap="emit('open-lineup')"
    >
      <view class="fan-entry__top">
        <view class="fan-entry__head">
          <text class="fan-entry__kicker fan-entry__kicker--lineup">{{ lineupModel.kicker }}</text>
          <text class="fan-entry__title">{{ lineupModel.title }}</text>
        </view>
        <view class="fan-entry__pitch" aria-hidden="true">
          <view class="fan-entry__pitch-half" />
          <view class="fan-entry__pitch-circle" />
          <view
            v-for="dot in lineupModel.pitchDots"
            :key="dot.slot"
            class="fan-entry__pitch-dot"
            :style="dot.style"
          />
        </view>
      </view>
      <text class="fan-entry__subtitle">{{ lineupModel.subtitle }}</text>
      <view class="fan-entry__cta fan-entry__cta--lineup">
        <text class="fan-entry__cta-text">{{ lineupModel.ctaLabel }}</text>
        <image class="fan-entry__cta-icon" :src="chevronRightWhiteIcon" mode="aspectFit" aria-hidden="true" />
      </view>
    </view>

    <view
      class="fan-entry fan-entry--like"
      role="button"
      :aria-label="`${transferModel.title}：${transferModel.subtitle}`"
      hover-class="fan-entry--pressed"
      :hover-stay-time="80"
      @tap="emit('open-transfer')"
    >
      <view class="fan-entry__top">
        <view class="fan-entry__head">
          <text class="fan-entry__kicker fan-entry__kicker--like">{{ transferModel.kicker }}</text>
          <text class="fan-entry__title">{{ transferModel.title }}</text>
        </view>
        <view class="fan-entry__visual" aria-hidden="true">
          <view v-if="transferModel.avatars.length" class="fan-entry__avatars">
            <view
              v-for="(player, index) in transferModel.avatars"
              :key="player.playerId"
              class="fan-entry__avatar"
              :class="{ 'fan-entry__avatar--stacked': index > 0 }"
            >
              <FiAvatar :src="player.avatarUrl" :size="96" :name="player.name" />
            </view>
          </view>
          <view class="fan-entry__like-badge">
            <image class="fan-entry__like-icon" :src="likeActiveIcon" mode="aspectFit" />
            <text class="fan-entry__like-count">{{ transferModel.likeBadgeText }}</text>
          </view>
        </view>
      </view>
      <text class="fan-entry__subtitle">{{ transferModel.subtitle }}</text>
      <view class="fan-entry__cta fan-entry__cta--like">
        <text class="fan-entry__cta-text">{{ transferModel.ctaLabel }}</text>
        <image class="fan-entry__cta-icon" :src="chevronRightWhiteIcon" mode="aspectFit" aria-hidden="true" />
      </view>
    </view>
    <view
      class="fan-entry fan-entry--director"
      role="button"
      aria-label="我来当总监：休赛期建队模拟，定 5 外援拉微信群 PK"
      hover-class="fan-entry--pressed"
      :hover-stay-time="80"
      @tap="emit('open-director')"
    >
      <view class="fan-entry__director-left">
        <text class="fan-entry__kicker fan-entry__kicker--director">休赛期建队</text>
        <text class="fan-entry__title">我来当总监</text>
        <text class="fan-entry__subtitle">给{{ teamName }}定 5 外援阵容，发微信群看 1v1 分歧报告</text>
      </view>
      <view class="fan-entry__cta fan-entry__cta--director">
        <text class="fan-entry__cta-text">去建队</text>
        <image class="fan-entry__cta-icon" :src="chevronRightWhiteIcon" mode="aspectFit" aria-hidden="true" />
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import FiAvatar from '../../../components/FiAvatar.vue'
import chevronRightWhiteIcon from '../../../static/home/chevron-right-white.svg'
import likeActiveIcon from '../../../static/lineup/reaction-like-active.png'
import type { LineupConsensus } from '../../../types/lineup'
import type { TransferSummary } from '../../../types/transfer'
import {
  buildHomeLineupEntryViewModel,
  buildHomeTransferEntryViewModel,
} from '../homeFanEntriesHelpers'

const props = defineProps<{
  teamName: string
  consensus: LineupConsensus | null
  transferSummary: TransferSummary | null
}>()

const emit = defineEmits<{
  (e: 'open-lineup'): void
  (e: 'open-transfer'): void
  (e: 'open-director'): void
}>()

const lineupModel = computed(() =>
  buildHomeLineupEntryViewModel(props.consensus, props.teamName),
)
const transferModel = computed(() =>
  buildHomeTransferEntryViewModel(props.transferSummary, props.teamName),
)
</script>

<style scoped lang="css">
.fan-entries {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--fi-space-card-gap);
  align-items: stretch;
}

.fan-entry {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-14);
  box-sizing: border-box;
  padding: var(--fi-space-20);
  border-radius: var(--fi-radius-xl);
  box-shadow: var(--fi-shadow-card);
  transition: opacity var(--fi-motion-duration-fast) var(--fi-motion-ease-out);
}

.fan-entry--lineup {
  background: var(--fi-gradient-fan-entry-lineup);
  border: var(--fi-border-width) solid var(--fi-color-fan-entry-lineup-border);
}

.fan-entry--like {
  background: var(--fi-gradient-fan-entry-like);
  border: var(--fi-border-width) solid var(--fi-color-fan-entry-like-border);
}

.fan-entry--director {
  grid-column: 1 / -1;
  flex-direction: row;
  align-items: center;
  justify-content: space-between;
  background: linear-gradient(135deg, #eef3ff 0%, #ffffff 72%);
  border: var(--fi-border-card);
}

.fan-entry__director-left {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: var(--fi-space-8);
}

.fan-entry__kicker--director {
  background: var(--fi-color-outsider-soft);
  color: var(--fi-color-outsider);
}

.fan-entry__cta--director {
  flex-shrink: 0;
  margin-top: 0;
  background: var(--fi-color-text-primary);
}

.fan-entry--pressed {
  opacity: var(--fi-opacity-pressed);
}

.fan-entry__top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--fi-space-10);
}

.fan-entry__head {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: var(--fi-space-10);
}

.fan-entry__kicker {
  display: inline-flex;
  align-items: center;
  padding: var(--fi-space-4) var(--fi-space-10);
  border-radius: var(--fi-radius-round);
  font-size: var(--fi-font-20);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-snug);
  white-space: nowrap;
}

.fan-entry__kicker--lineup {
  background: var(--fi-color-seat-desired-bg);
  color: var(--fi-color-seat-desired);
}

.fan-entry__kicker--like {
  background: var(--fi-color-primary-soft);
  color: var(--fi-color-primary);
}

.fan-entry__title {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-32);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-tight);
}

.fan-entry__pitch {
  position: relative;
  flex-shrink: 0;
  width: var(--fi-space-fan-entry-pitch-width);
  height: var(--fi-space-fan-entry-pitch-height);
  overflow: hidden;
  border-radius: var(--fi-radius-xs);
  border: var(--fi-border-width) solid var(--fi-color-lineup-markings);
  background: var(--fi-color-lineup-grass);
  box-shadow: var(--fi-shadow-soft);
}

.fan-entry__pitch-half {
  position: absolute;
  top: 50%;
  left: 0;
  width: 100%;
  border-top: var(--fi-border-width) solid var(--fi-color-lineup-markings);
}

.fan-entry__pitch-circle {
  position: absolute;
  top: 50%;
  left: 50%;
  width: var(--fi-space-20);
  height: var(--fi-space-20);
  transform: translate(-50%, -50%);
  border: var(--fi-border-width) solid var(--fi-color-lineup-markings);
  border-radius: var(--fi-radius-round);
}

.fan-entry__pitch-dot {
  position: absolute;
  width: var(--fi-space-6);
  height: var(--fi-space-6);
  transform: translate(-50%, -50%);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-surface);
}

.fan-entry__visual {
  flex-shrink: 0;
  height: var(--fi-space-fan-entry-pitch-height);
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  justify-content: space-between;
}

.fan-entry__avatars {
  display: flex;
  align-items: center;
}

.fan-entry__avatar {
  flex-shrink: 0;
  width: var(--fi-space-44);
  height: var(--fi-space-44);
  overflow: hidden;
  border-radius: var(--fi-radius-round);
  border: var(--fi-border-width) solid var(--fi-color-surface);
  background: var(--fi-color-page-soft);
  box-shadow: var(--fi-shadow-soft);
  font-size: var(--fi-font-18);
}

.fan-entry__avatar--stacked {
  margin-left: var(--fi-space-fan-entry-avatar-overlap);
}

.fan-entry__like-badge {
  display: inline-flex;
  align-items: center;
  gap: var(--fi-space-6);
  padding: var(--fi-space-4) var(--fi-space-12);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-surface);
  border: var(--fi-border-width) solid var(--fi-color-fan-entry-like-border);
  box-shadow: var(--fi-shadow-soft);
}

.fan-entry__like-icon {
  width: var(--fi-space-24);
  height: var(--fi-space-24);
}

.fan-entry__like-count {
  color: var(--fi-color-primary);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-extrabold);
  font-variant-numeric: tabular-nums;
  line-height: var(--fi-leading-none);
}

.fan-entry__subtitle {
  display: block;
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-normal);
}

.fan-entry__cta {
  margin-top: auto;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--fi-space-10) var(--fi-space-16) var(--fi-space-10) var(--fi-space-18);
  border-radius: var(--fi-radius-round);
  color: var(--fi-color-surface);
}

.fan-entry__cta--lineup {
  background: var(--fi-color-lineup-grass);
}

.fan-entry__cta--like {
  background: var(--fi-color-primary);
}

.fan-entry__cta-text {
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

.fan-entry__cta-icon {
  flex-shrink: 0;
  width: var(--fi-space-36);
  height: var(--fi-space-36);
}

@media (prefers-reduced-motion: reduce) {
  .fan-entry {
    transition: none;
  }
}
</style>
