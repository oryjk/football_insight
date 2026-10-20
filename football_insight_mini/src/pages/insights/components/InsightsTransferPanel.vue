<template>
  <view class="transfer-panel">
    <view class="transfer-panel__header">
      <text class="transfer-panel__title">转会投票</text>
      <text v-if="summary?.window" class="transfer-panel__status">{{ summary.window.name }} · {{ summary.window.open ? '投票中' : '已结束' }}</text>
    </view>
    <text v-if="hint" class="transfer-panel__hint">{{ hint }}</text>
    <view v-if="loading && !summary" class="transfer-panel__skeletons">
      <view v-for="index in 3" :key="index" class="fi-skeleton transfer-panel__skeleton" />
    </view>
    <view v-else-if="error && !summary" class="transfer-panel__state">
      <text>{{ error }}</text>
      <button class="transfer-panel__action" @tap="emit('retry')">重试</button>
    </view>
    <text v-else-if="summary && !summary.window" class="transfer-panel__state">暂无转会投票</text>
    <template v-else-if="summary">
      <view v-for="board in summary.boards" :key="board.intent" class="transfer-panel__board">
        <!-- 标题行只展示，不可点：原先整行可点且"查看全部"压在第一张卡的赞踩胶囊正上方，容易误点。 -->
        <view class="transfer-panel__board-head">
          <text class="transfer-panel__board-title">{{ TRANSFER_BOARD_LABELS[board.intent] }}<text class="transfer-panel__count">{{ board.total }}</text></text>
        </view>
        <text v-if="!board.items.length" class="transfer-panel__empty">还没有人提名，来做第一个</text>
        <view v-for="card in board.items" :key="card.share_code" class="transfer-panel__item">
          <FiTransferCard :card="card" :pending="pending.includes(card.share_code)" @open="emit('open', card.share_code)" @vote="emit('vote', card, $event)" />
        </view>
        <!-- "查看全部"放在列表下方居中，与右侧赞踩胶囊在位置上错开。 -->
        <view v-if="board.total > 0" class="transfer-panel__more-row" hover-class="transfer-panel__more-row--pressed" @tap="emit('more', board.intent)">
          <text class="transfer-panel__more">查看全部 {{ board.total }} 张 ›</text>
        </view>
      </view>
      <button v-if="summary.window?.open" class="transfer-panel__action transfer-panel__action--primary" @tap="emit('nominate')">提名球员</button>
    </template>
  </view>
</template>
<script setup lang="ts">
import FiTransferCard from '../../../components/FiTransferCard.vue'
import type { TransferCard, TransferIntent, TransferSummary } from '../../../types/transfer'
import { TRANSFER_BOARD_LABELS } from '../../../utils/transferLinks'
defineProps<{ summary: TransferSummary | null; loading: boolean; error: string; hint: string; pending: string[] }>()
const emit = defineEmits<{
  (e: 'retry' | 'nominate'): void
  (e: 'vote', card: TransferCard, value: 1 | -1): void
  (e: 'open', code: string): void
  (e: 'more', intent: TransferIntent): void
}>()
</script>
<style scoped>
@import '../../../styles/fi-skeleton.css';
.transfer-panel { display: flex; flex-direction: column; gap: var(--fi-space-20); }
.transfer-panel__header { display: flex; align-items: baseline; justify-content: space-between; gap: var(--fi-space-12); }
.transfer-panel__title { color: var(--fi-color-text-primary); font-size: var(--fi-font-28); font-weight: var(--fi-weight-extrabold); }
.transfer-panel__status { color: var(--fi-color-text-caption); font-size: var(--fi-font-22); }
.transfer-panel__hint { padding: var(--fi-space-12) var(--fi-space-16); border-radius: var(--fi-radius-md); background: var(--fi-color-page-soft); color: var(--fi-color-text-secondary); font-size: var(--fi-font-22); }
.transfer-panel__board { padding-top: var(--fi-space-16); border-top: var(--fi-border-width) solid var(--fi-color-divider); }
.transfer-panel__board-head { display: flex; align-items: center; padding-bottom: var(--fi-space-4); }
.transfer-panel__more-row { display: flex; align-items: center; justify-content: center; margin-top: var(--fi-space-4); padding: var(--fi-space-16) 0 var(--fi-space-8); border-top: var(--fi-border-width) solid var(--fi-color-divider-soft); }
.transfer-panel__more-row--pressed { opacity: var(--fi-opacity-pressed); }
.transfer-panel__board-title { color: var(--fi-color-text-strong); font-size: var(--fi-font-26); font-weight: var(--fi-weight-bold); }
.transfer-panel__count { margin-left: var(--fi-space-8); color: var(--fi-color-text-caption); font-size: var(--fi-font-22); font-weight: normal; }
.transfer-panel__more { color: var(--fi-color-text-secondary); font-size: var(--fi-font-22); }
.transfer-panel__empty, .transfer-panel__state { padding: var(--fi-space-20) 0; color: var(--fi-color-text-caption); font-size: var(--fi-font-24); text-align: center; }
.transfer-panel__item + .transfer-panel__item { border-top: var(--fi-border-width) solid var(--fi-color-divider-soft); }
.transfer-panel__action { margin: var(--fi-space-12) 0 0; padding: var(--fi-space-16); border-radius: var(--fi-radius-round); background: var(--fi-color-surface-panel); color: var(--fi-color-primary); font-size: var(--fi-font-24); line-height: var(--fi-leading-normal); }
.transfer-panel__action::after { border: 0; }
.transfer-panel__action--primary { background: var(--fi-color-primary); color: var(--fi-color-surface); }
.transfer-panel__skeletons { display: flex; flex-direction: column; gap: var(--fi-space-12); }
.transfer-panel__skeleton { height: calc(var(--fi-space-64) * 1.5); border-radius: var(--fi-radius-md); }
</style>
