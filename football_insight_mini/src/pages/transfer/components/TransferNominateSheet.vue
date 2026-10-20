<template>
  <FiBottomSheet :visible="visible" eyebrow="提名球员" title="你希望谁…" height="80vh" @update:visible="emit('update:visible', $event)">
    <view class="nominate__intents">
      <FiSegmented :items="NOMINATE_INTENT_ITEMS" :model-value="intent" aria-label="提名意向" @update:model-value="emit('update:intent', $event as TransferIntent)" />
    </view>
    <!-- 搜索只在本地过滤候选，不提交到后端。 -->
    <input v-model="keyword" class="nominate__search" placeholder="按名字查找" confirm-type="search" />
    <view v-if="loading" class="nominate__state"><text>候选球员加载中…</text></view>
    <view v-else-if="error" class="nominate__state">
      <text>{{ error }}</text>
      <button class="nominate__retry" @tap="emit('retry')">重试</button>
    </view>
    <view v-else-if="!visibleGroups.length" class="nominate__state"><text>没有符合的球员</text></view>
    <view v-for="group in visibleGroups" :key="group.key" class="nominate__group">
      <text class="nominate__group-title">{{ group.title }}</text>
      <view v-for="player in group.players" :key="player.player_id" class="nominate__row" hover-class="nominate__row--pressed" @tap="!submitting && emit('pick', player.player_id)">
        <image v-if="player.avatar_url" class="nominate__avatar" :src="player.avatar_url" mode="aspectFill" lazy-load />
        <view v-else class="nominate__avatar nominate__avatar--fallback"><text>{{ player.name.slice(0, 1) }}</text></view>
        <view class="nominate__body">
          <text class="nominate__name">{{ player.name }}</text>
          <text class="nominate__meta">{{ [positionLabel(player.position), player.current_team?.team_name ?? ''].filter(Boolean).join(' · ') }}</text>
        </view>
      </view>
    </view>
  </FiBottomSheet>
</template>
<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import FiBottomSheet from '../../../components/FiBottomSheet.vue'
import FiSegmented from '../../../components/FiSegmented.vue'
import type { TransferCandidateGroup, TransferIntent } from '../../../types/transfer'
import { positionLabel } from '../../../utils/transferLinks'
import { NOMINATE_INTENT_ITEMS, filterCandidateGroups } from '../helpers'
const props = defineProps<{ visible: boolean; intent: TransferIntent; groups: TransferCandidateGroup[]; loading: boolean; error: string; submitting: boolean }>()
const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'update:intent', value: TransferIntent): void
  (e: 'pick', playerId: number): void
  (e: 'retry'): void
}>()
const keyword = ref('')
const visibleGroups = computed(() => filterCandidateGroups(props.groups, keyword.value))
watch(() => props.visible, open => { if (open) keyword.value = '' })
</script>
<style scoped>
.nominate__intents { margin-bottom: var(--fi-space-16); }
.nominate__search { box-sizing: border-box; width: 100%; height: var(--fi-space-72); margin-bottom: var(--fi-space-16); padding: 0 var(--fi-space-24); border: var(--fi-border-card); border-radius: var(--fi-radius-round); background: var(--fi-color-page-soft); color: var(--fi-color-text-strong); font-size: var(--fi-font-26); }
.nominate__state { padding: var(--fi-space-32) 0; color: var(--fi-color-text-caption); font-size: var(--fi-font-24); text-align: center; }
.nominate__retry { margin: var(--fi-space-12) auto 0; padding: var(--fi-space-12) var(--fi-space-32); border-radius: var(--fi-radius-round); background: var(--fi-color-surface-panel); color: var(--fi-color-primary); font-size: var(--fi-font-24); }
.nominate__retry::after { border: 0; }
.nominate__group { margin-bottom: var(--fi-space-16); }
.nominate__group-title { display: block; padding: var(--fi-space-12) var(--fi-space-4); color: var(--fi-color-text-secondary); font-size: var(--fi-font-22); font-weight: var(--fi-weight-bold); }
.nominate__row { display: flex; align-items: center; gap: var(--fi-space-18); padding: var(--fi-space-14) var(--fi-space-4); border-top: var(--fi-border-width) solid var(--fi-color-divider-soft); }
.nominate__row--pressed { opacity: var(--fi-opacity-pressed); }
.nominate__avatar { flex-shrink: 0; width: var(--fi-space-64); height: var(--fi-space-64); border-radius: var(--fi-radius-round); background: var(--fi-color-page); }
.nominate__avatar--fallback { display: flex; align-items: center; justify-content: center; color: var(--fi-color-text-secondary); font-size: var(--fi-font-26); font-weight: var(--fi-weight-bold); }
.nominate__body { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: var(--fi-space-4); }
.nominate__name { color: var(--fi-color-text-strong); font-size: var(--fi-font-28); font-weight: var(--fi-weight-bold); }
.nominate__meta { color: var(--fi-color-text-muted); font-size: var(--fi-font-22); }
</style>
