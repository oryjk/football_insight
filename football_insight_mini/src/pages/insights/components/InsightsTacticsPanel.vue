<template>
  <view class="tactics-panel">
    <InsightsMyTacticsCard :authenticated="authenticated" :loading="mineLoading" :error="mineError" :draft="draft" :published="published" @edit="emit('edit')" @retry="emit('retry-mine')" />
    <view class="community">
      <view class="community__header"><text class="community__title">球队战术<text v-if="total" class="community__count">{{ total }}</text></text>
        <view class="community__sort"><FiSegmented :items="sortItems" :model-value="sort" compact aria-label="战术排序" @update:model-value="changeSort" /></view>
      </view>
      <view v-if="loading && !items.length" class="community__skeletons"><view v-for="n in 3" :key="n" class="community__skeleton fi-skeleton-block" /></view>
      <view v-if="error" class="community__state"><text>{{ error }}</text><button class="community__retry" @tap="emit('retry-list')">重试</button></view>
      <view v-else-if="!loading && !items.length" class="community__state"><text>还没有人发布战术，来做第一个排阵的人。</text></view>
      <view v-for="(item, index) in items" :key="item.share_code" class="community__item">
        <FiLineupCommunityCard :item="item" :rank="sort === 'likes' ? index + 1 : undefined" :pending="pendingLikes.includes(item.share_code)" compact @open="emit('open', item.share_code)" @like="emit('like', item)" @dislike="emit('dislike', item)" />
      </view>
      <button v-if="items.length && items.length < total" class="community__more" :disabled="loading" @tap="emit('more')">{{ loading ? '加载中…' : '查看更多战术' }}</button>
    </view>
  </view>
</template>
<script setup lang="ts">
import FiSegmented from '../../../components/FiSegmented.vue'
import FiLineupCommunityCard from '../../../components/FiLineupCommunityCard.vue'
import InsightsMyTacticsCard from './InsightsMyTacticsCard.vue'
import type { LineupSort, PublicLineupSummary, SavedLineup, SavedLineupDraft } from '../../../types/lineup'
defineProps<{ authenticated: boolean; mineLoading: boolean; mineError: string; draft: SavedLineupDraft | null; published: SavedLineup | null; items: PublicLineupSummary[]; total: number; sort: LineupSort; loading: boolean; error: string; pendingLikes: string[] }>()
const emit = defineEmits<{ (e: 'edit' | 'retry-mine' | 'retry-list' | 'more'): void; (e: 'open', code: string): void; (e: 'like' | 'dislike', item: PublicLineupSummary): void; (e: 'sort', order: LineupSort): void }>()
const sortItems = [{ key: 'latest', label: '最新' }, { key: 'likes', label: '点赞榜' }]
function changeSort(value: string) { if (value === 'latest' || value === 'likes') emit('sort', value) }
</script>
<style scoped>
@import '../../../styles/fi-skeleton.css';
.tactics-panel { display:flex; flex-direction:column; gap:var(--fi-space-28); }
.community { padding-top:var(--fi-space-24); border-top:var(--fi-border-width) solid var(--fi-color-divider); }
.community__header { display:flex; justify-content:space-between; align-items:center; gap:var(--fi-space-12); margin-bottom:var(--fi-space-16); }
.community__title { color:var(--fi-color-text-primary); font-size:var(--fi-font-28); font-weight:var(--fi-weight-extrabold); }
.community__count { margin-left:var(--fi-space-10); color:var(--fi-color-text-caption); font-size:var(--fi-font-22); font-weight:normal; }
.community__sort { width:calc(var(--fi-space-64) * 3.5); flex-shrink:0; }
.community__item { border-bottom:var(--fi-border-width) solid var(--fi-color-divider); }
.community__state { padding:var(--fi-space-28) var(--fi-space-16); text-align:center; color:var(--fi-color-text-caption); font-size:var(--fi-font-24); line-height:var(--fi-leading-normal); }
.community__retry,.community__more { margin:var(--fi-space-12) 0 0; padding:var(--fi-space-16); background:var(--fi-color-surface-panel); border-radius:var(--fi-radius-round); color:var(--fi-color-primary); font-size:var(--fi-font-24); line-height:var(--fi-leading-normal); }
.community__retry::after,.community__more::after { border:0; }
.community__skeletons { display:flex; flex-direction:column; gap:var(--fi-space-12); }
.community__skeleton { height:calc(var(--fi-space-64) * 1.5); border-radius:var(--fi-radius-md); }
</style>
