<template>
  <view class="page-root">
    <view class="mode-shell">
      <view class="mode-toggle">
        <button
          class="mode-toggle__item"
          :class="{ active: activeMode === 'rankings' }"
          @tap="activeMode = 'rankings'"
        >
          榜单
        </button>
        <button
          class="mode-toggle__item"
          :class="{ active: activeMode === 'matches' }"
          @tap="activeMode = 'matches'"
        >
          赛程
        </button>
      </view>
    </view>

    <RankingsContent
      v-if="activeMode === 'rankings'"
      ref="rankingsContentRef"
    />
    <MatchesContent v-else />
  </view>
</template>

<script setup lang="ts">
import { nextTick, ref } from 'vue'
import { onLoad, onShareAppMessage } from '@dcloudio/uni-app'
import RankingsContent from './RankingsContent.vue'
import MatchesContent from '../matches/MatchesContent.vue'

const activeMode = ref<'rankings' | 'matches'>('rankings')
const rankingsContentRef = ref<InstanceType<typeof RankingsContent> | null>(null)
const pendingSharedPosterSlug = ref<string | null>(null)

function normalizeRankingsMode(value: unknown): 'rankings' | 'matches' {
  return value === 'matches' ? 'matches' : 'rankings'
}

async function openPendingSharedPoster(): Promise<void> {
  if (!pendingSharedPosterSlug.value) {
    return
  }

  await nextTick()
  await rankingsContentRef.value?.openSharedStandingsPoster(pendingSharedPosterSlug.value)
}

onLoad((query) => {
  activeMode.value = normalizeRankingsMode(query?.mode)

  if (query?.openPoster === '1' && typeof query.table === 'string' && query.table.trim()) {
    activeMode.value = 'rankings'
    pendingSharedPosterSlug.value = decodeURIComponent(query.table)
    void openPendingSharedPoster()
  }
})

onShareAppMessage(() => {
  const standingsPosterPayload = rankingsContentRef.value?.getStandingsPosterSharePayload()

  if (standingsPosterPayload) {
    return standingsPosterPayload
  }

  return {
    title: activeMode.value === 'matches'
      ? '中超赛程和最近赛果，看看下一场对阵'
      : '中超榜单和积分走势，看看现在谁在前面',
    path: `/pages/rankings/index?mode=${activeMode.value}`,
  }
})
</script>

<style scoped lang="css">
.page-root {
  position: relative;
  min-height: 100vh;
  background: #f7f8fa;
}

.mode-shell {
  position: fixed;
  left: 0;
  right: 0;
  top: 0;
  z-index: 30;
  padding: 18rpx 26rpx 10rpx;
  background: rgba(247, 248, 250, 0.86);
  backdrop-filter: blur(16rpx);
  -webkit-backdrop-filter: blur(16rpx);
}

.mode-toggle {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 4rpx;
  border: 2rpx solid rgba(21, 22, 27, 0.06);
  border-radius: 999rpx;
  padding: 6rpx;
  background: rgba(255, 255, 255, 0.72);
  box-shadow: 0 10rpx 24rpx rgba(26, 28, 36, 0.05);
}

.mode-toggle__item {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 58rpx;
  border-radius: 999rpx;
  background: transparent;
  color: #727782;
  font-size: 26rpx;
  font-weight: 900;
  line-height: 1;
}

.mode-toggle__item::after {
  border: 0;
}

.mode-toggle__item.active {
  background: #15161b;
  color: #ffffff;
  box-shadow: 0 8rpx 18rpx rgba(21, 22, 27, 0.12);
}
</style>
