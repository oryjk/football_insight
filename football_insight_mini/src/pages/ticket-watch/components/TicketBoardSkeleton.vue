<template>
  <view class="fi-skeleton-stack" role="status" aria-label="回流看板加载中">
    <template v-if="mode === 'current'">
      <view v-if="showTabs" class="ticket-skeleton__tabs" aria-hidden="true">
        <view v-for="tab in 3" :key="tab" class="fi-skeleton-block ticket-skeleton__tab" />
      </view>
      <view class="fi-skeleton-panel ticket-skeleton" aria-hidden="true">
        <view class="fi-skeleton-heading">
          <view class="ticket-skeleton__identity">
            <view class="fi-skeleton-block ticket-skeleton__ball" />
            <view class="fi-skeleton-block ticket-skeleton__match" />
          </view>
          <view class="fi-skeleton-block ticket-skeleton__meta" />
        </view>
        <view class="fi-skeleton-block ticket-skeleton__status" />
        <view class="ticket-skeleton__pair">
          <view v-for="metric in 2" :key="metric" class="ticket-skeleton__metric">
            <view class="fi-skeleton-block ticket-skeleton__label" />
            <view class="fi-skeleton-block ticket-skeleton__value" />
          </view>
        </view>
        <view class="ticket-skeleton__actions">
          <view class="fi-skeleton-block ticket-skeleton__button" />
          <view class="ticket-skeleton__pair">
            <view v-for="action in 2" :key="action" class="fi-skeleton-block ticket-skeleton__button" />
          </view>
        </view>
        <view class="fi-skeleton-block ticket-skeleton__body" />
      </view>
    </template>

    <view v-if="mode === 'history'" class="fi-skeleton-panel ticket-skeleton" aria-hidden="true">
      <view class="fi-skeleton-heading">
        <view class="fi-skeleton-copy">
          <view class="fi-skeleton-block fi-skeleton-kicker" />
          <view class="fi-skeleton-block fi-skeleton-title" />
        </view>
        <view class="fi-skeleton-block ticket-skeleton__pill" />
      </view>
      <view class="ticket-skeleton__history-list">
        <view v-for="match in 4" :key="match" class="ticket-skeleton__history-chip">
          <view class="fi-skeleton-block ticket-skeleton__label" />
          <view class="fi-skeleton-block ticket-skeleton__body" />
        </view>
      </view>
    </view>

    <template v-if="mode !== 'history-stats'">
      <view class="fi-skeleton-panel ticket-skeleton" aria-hidden="true">
        <view class="fi-skeleton-heading">
          <view class="fi-skeleton-copy">
            <view class="fi-skeleton-block fi-skeleton-kicker" />
            <view class="fi-skeleton-block fi-skeleton-title" />
          </view>
          <view class="fi-skeleton-block ticket-skeleton__pill" />
        </view>
        <view class="ticket-skeleton__triple">
          <view v-for="focus in 3" :key="focus" class="ticket-skeleton__focus">
            <view class="fi-skeleton-block ticket-skeleton__label" />
            <view class="fi-skeleton-block ticket-skeleton__body" />
            <view class="fi-skeleton-block ticket-skeleton__label" />
          </view>
        </view>
      </view>
      <view class="fi-skeleton-panel ticket-skeleton" aria-hidden="true">
        <view class="fi-skeleton-heading">
          <view class="fi-skeleton-copy">
            <view class="fi-skeleton-block fi-skeleton-kicker" />
            <view class="fi-skeleton-block fi-skeleton-title" />
          </view>
          <view class="fi-skeleton-block ticket-skeleton__pill" />
        </view>
        <view class="ticket-skeleton__inventory">
          <view v-for="cell in 8" :key="cell" class="ticket-skeleton__cell">
            <view class="fi-skeleton-block ticket-skeleton__body" />
            <view class="fi-skeleton-block ticket-skeleton__label" />
            <view class="fi-skeleton-block ticket-skeleton__body" />
          </view>
        </view>
      </view>
    </template>

    <view v-else class="fi-skeleton-panel ticket-skeleton" aria-hidden="true">
      <view class="fi-skeleton-heading">
        <view class="fi-skeleton-copy">
          <view class="fi-skeleton-block fi-skeleton-kicker" />
          <view class="fi-skeleton-block fi-skeleton-title" />
        </view>
        <view class="fi-skeleton-block ticket-skeleton__pill" />
      </view>
      <view class="ticket-skeleton__triple">
        <view v-for="metric in 3" :key="metric" class="ticket-skeleton__metric">
          <view class="fi-skeleton-block ticket-skeleton__label" />
          <view class="fi-skeleton-block ticket-skeleton__value" />
        </view>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
withDefaults(defineProps<{
  mode: 'current' | 'history' | 'history-stats'
  showTabs?: boolean
}>(), { showTabs: true })
</script>

<style scoped lang="css">
@import '../../../styles/fi-skeleton.css';

/* Keep shared skeleton shapes, with a static solid fill for this board. */
.fi-skeleton-panel::before { display: none; }
.fi-skeleton-block { background: var(--fi-color-skeleton-end); }
.ticket-skeleton { display: grid; gap: var(--fi-space-20); }
.ticket-skeleton__tabs { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: var(--fi-space-6); padding: var(--fi-space-6); border: var(--fi-border-width) solid var(--fi-color-border-chip); border-radius: var(--fi-radius-round); background: var(--fi-color-page); }
.ticket-skeleton__tab { height: var(--fi-space-64); }
.ticket-skeleton__identity { display: flex; align-items: center; flex: 1; min-width: 0; gap: var(--fi-space-14); }
.ticket-skeleton__ball { width: var(--fi-icon-md); height: var(--fi-icon-md); flex-shrink: 0; }
.ticket-skeleton__match { width: 72%; height: var(--fi-space-32); border-radius: var(--fi-radius-sm); }
.ticket-skeleton__meta { width: 30%; height: var(--fi-space-20); flex-shrink: 0; }
.ticket-skeleton__status { height: var(--fi-space-56); border-radius: var(--fi-radius-md); }
.ticket-skeleton__pair { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: var(--fi-space-16); }
.ticket-skeleton__metric { display: grid; gap: var(--fi-space-10); padding: var(--fi-space-16) var(--fi-space-18); border-radius: var(--fi-radius-md); background: var(--fi-color-page); }
.ticket-skeleton__label { width: 48%; height: var(--fi-space-22); }
.ticket-skeleton__metric .ticket-skeleton__label { height: var(--fi-icon-md); }
.ticket-skeleton__value { width: 52%; height: var(--fi-space-44); border-radius: var(--fi-radius-sm); }
.ticket-skeleton__actions { display: grid; gap: var(--fi-space-12); }
.ticket-skeleton__actions .ticket-skeleton__pair { gap: var(--fi-space-12); }
.ticket-skeleton__button { height: var(--fi-space-78); border-radius: var(--fi-radius-md); }
.ticket-skeleton__body { width: 88%; }
.ticket-skeleton__pill { width: 20%; height: var(--fi-space-48); flex-shrink: 0; }
.ticket-skeleton__triple { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: var(--fi-space-10); }
.ticket-skeleton__focus { display: grid; gap: var(--fi-space-10); padding: var(--fi-space-18) var(--fi-space-14); border-radius: var(--fi-radius-md); background: var(--fi-color-page); }
.ticket-skeleton__inventory { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: var(--fi-space-10); }
.ticket-skeleton__cell { display: grid; justify-items: center; gap: var(--fi-space-10); padding: var(--fi-space-22) var(--fi-space-14) var(--fi-space-16); border-radius: var(--fi-radius-md); background: var(--fi-color-page); }
.ticket-skeleton__history-list { display: flex; gap: var(--fi-space-10); overflow: hidden; }
.ticket-skeleton__history-chip { display: grid; gap: var(--fi-space-12); flex: 0 0 38%; padding: var(--fi-space-20) var(--fi-space-22); border-radius: var(--fi-radius-md); background: var(--fi-color-page); }
</style>
