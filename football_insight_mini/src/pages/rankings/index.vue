<template>
  <page-meta :page-style="navigationPageStyle + pageLockStyle" />
  <view class="page-root" :class="{ 'page-root--locked': pageScrollLocked }">
    <FiBrandNav />

    <view>
      <RankingsContent
        ref="rankingsContentRef"
        @page-scroll-lock-change="handlePageScrollLockChange"
      />
    </view>
    <view>
      <MatchesContent embedded />
    </view>

  </view>
</template>

<script setup lang="ts">
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import { computed, nextTick, ref } from 'vue'
import { onLoad, onShareAppMessage } from '@dcloudio/uni-app'
import FiBrandNav from '../../components/FiBrandNav.vue'
import RankingsContent from './RankingsContent.vue'
import MatchesContent from '../matches/MatchesContent.vue'

const { navigationPageStyle } = useNavigationLayout()

const rankingsContentRef = ref<InstanceType<typeof RankingsContent> | null>(null)
const pendingSharedPosterSlug = ref<string | null>(null)
const pageScrollLocked = ref(false)
const pageLockStyle = computed(() => pageScrollLocked.value ? 'overflow: hidden;' : '')

async function openPendingSharedPoster(): Promise<void> {
  if (!pendingSharedPosterSlug.value) {
    return
  }

  await nextTick()
  await rankingsContentRef.value?.openSharedStandingsPoster(pendingSharedPosterSlug.value)
}

function handlePageScrollLockChange(locked: boolean): void {
  pageScrollLocked.value = locked
}

onLoad((query) => {
  if (query?.openPoster === '1' && typeof query.table === 'string' && query.table.trim()) {
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
    title: '中超榜单、赛程和最近赛果，看看现在谁在前面',
    path: '/pages/rankings/index',
  }
})
</script>

<style scoped lang="css">
.page-root {
  position: relative;
  min-height: 100vh;
  background: #f7f8fa;
}
.page-root--locked {
  height: 100vh;
  overflow: hidden;
}
</style>
