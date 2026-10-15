<template>
  <image
    v-if="isRongchengTopic(item)"
    class="topic-crest"
    :class="{ 'topic-crest--loaded': loaded, 'topic-crest--history': variant === 'history' }"
    :src="rongchengCrest"
    mode="aspectFit"
    aria-hidden="true"
    @load="loaded = true"
    @error="loaded = false"
  />
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { CHENGDU_RONGCHENG_CREST_IMAGE_URL as rongchengCrest } from '../../config/assets'
import type { DailyTopicItem } from '../../types/dailyTopic'
import { isRongchengTopic } from '../../utils/dailyTopics'
withDefaults(defineProps<{ item: Pick<DailyTopicItem, 'teamTags'>; variant?: 'hero' | 'history' }>(), { variant: 'hero' })
const loaded = ref(false)
</script>

<style scoped>
.topic-crest { position: absolute; z-index: 0; width: var(--fi-component-topic-crest-size); height: var(--fi-component-topic-crest-size); right: calc(-1 * var(--fi-space-32)); top: 50%; transform: translateY(-50%) rotate(12deg); opacity: 0; pointer-events: none; }
.topic-crest--loaded { opacity: var(--fi-component-topic-crest-opacity); }
.topic-crest--history.topic-crest--loaded { opacity: var(--fi-component-topic-crest-history-opacity); }
</style>
