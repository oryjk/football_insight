<template>
  <page-meta :page-style="navigationPageStyle + pageLockStyle" />
  <view class="page-root" :class="{ 'page-root--locked': pageScrollLocked }">
    <FiBrandNav />
    <FiPageBackdrop :image-url="pageBackgroundImageUrl" />

    <template v-for="section in visibleSections" :key="section">
      <view v-if="section === 'rankings'" class="page-section-slot">
        <RankingsContent
          ref="rankingsContentRef"
          :embedded="true"
          @page-scroll-lock-change="handlePageScrollLockChange"
        />
      </view>
      <view v-else class="page-section-slot">
        <MatchesContent embedded />
      </view>
    </template>
  </view>
</template>

<script setup lang="ts">
import { buildSharePath } from '../../utils/shareSource'
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import { computed, nextTick, ref } from 'vue'
import { onLoad, onShareAppMessage, onShow } from '@dcloudio/uni-app'
import FiBrandNav from '../../components/FiBrandNav.vue'
import FiPageBackdrop from '../../components/FiPageBackdrop.vue'
import { usePageLayout } from '../../composables/usePageLayout'
import RankingsContent from './RankingsContent.vue'
import MatchesContent from '../matches/MatchesContent.vue'

const { navigationPageStyle } = useNavigationLayout()
const { visibleBlocks, pageBackgroundImageUrl, refreshLayout } = usePageLayout('rankings')

const visibleSections = computed<Array<'rankings' | 'matches'>>(() => {
  const rankingsIndex = visibleBlocks.value.findIndex((b) => b.key.startsWith('rankings_'))
  const matchesIndex = visibleBlocks.value.findIndex((b) => b.key.startsWith('matches_'))
  const entries: Array<{ id: 'rankings' | 'matches'; index: number }> = []
  if (rankingsIndex >= 0) entries.push({ id: 'rankings', index: rankingsIndex })
  if (matchesIndex >= 0) entries.push({ id: 'matches', index: matchesIndex })
  return entries.sort((a, b) => a.index - b.index).map((e) => e.id)
})

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

onShow(() => {
  void refreshLayout()
})

onShareAppMessage(() => {
  const standingsPosterPayload = rankingsContentRef.value?.getStandingsPosterSharePayload()

  if (standingsPosterPayload) {
    return { ...standingsPosterPayload, path: buildSharePath(standingsPosterPayload.path, 'rankings') }
  }

  return {
    title: '中超榜单、赛程和最近赛果，看看现在谁在前面',
    path: buildSharePath('/pages/rankings/index', 'rankings'),
  }
})
</script>

<style scoped lang="css">
.page-root {
  position: relative;
  min-height: 100vh;
  background: var(--fi-color-page-soft);
  padding-top: var(--fi-brand-nav-height);
  box-sizing: border-box;
}
.page-section-slot {
  position: relative;
  z-index: 1;
}
.page-root--locked {
  height: 100vh;
  overflow: hidden;
}
</style>
