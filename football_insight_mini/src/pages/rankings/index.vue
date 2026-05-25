<template>
  <view class="page-root">
    <FiBrandNav open-on-current-page @open-ai="openAiFromBrandNav" />

    <view>
      <RankingsContent ref="rankingsContentRef" />
    </view>
    <view>
      <MatchesContent embedded />
    </view>

    <FiAiChatSheet
      :visible="aiChatVisible"
      :current-user="currentAiUser"
      :ai-chat-mode="aiPublicConfig?.ai_chat_mode"
      @close="closeAiChat"
    />
  </view>
</template>

<script setup lang="ts">
import { nextTick, ref } from 'vue'
import { onLoad, onShareAppMessage } from '@dcloudio/uni-app'
import FiBrandNav from '../../components/FiBrandNav.vue'
import FiAiChatSheet from '../../components/FiAiChatSheet.vue'
import RankingsContent from './RankingsContent.vue'
import MatchesContent from '../matches/MatchesContent.vue'
import { useAiChatSheet } from '../../composables/useAiChatSheet'

const rankingsContentRef = ref<InstanceType<typeof RankingsContent> | null>(null)
const pendingSharedPosterSlug = ref<string | null>(null)
const {
  aiChatVisible,
  currentAiUser,
  aiPublicConfig,
  openAiChat,
  closeAiChat,
} = useAiChatSheet()

async function openPendingSharedPoster(): Promise<void> {
  if (!pendingSharedPosterSlug.value) {
    return
  }

  await nextTick()
  await rankingsContentRef.value?.openSharedStandingsPoster(pendingSharedPosterSlug.value)
}

function openAiFromBrandNav(): void {
  void openAiChat()
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
</style>
