<template>
  <view class="fi-skeleton-stack" role="status" aria-label="洞察加载中">
    <view class="fi-skeleton-panel insights-skeleton" aria-hidden="true">
      <view class="insights-skeleton__chips">
        <view v-for="chip in 3" :key="chip" class="fi-skeleton-block insights-skeleton__chip" />
      </view>
      <view class="fi-skeleton-heading">
        <view class="fi-skeleton-block fi-skeleton-title" />
        <view class="fi-skeleton-block insights-skeleton__link" />
      </view>
      <view class="insights-skeleton__metrics">
        <view v-for="metric in 2" :key="metric" class="insights-skeleton__metric">
          <view class="fi-skeleton-block insights-skeleton__label" />
          <view class="fi-skeleton-block insights-skeleton__value" />
        </view>
      </view>
    </view>
    <view class="fi-skeleton-panel insights-skeleton" aria-hidden="true">
      <view class="insights-skeleton__tabs">
        <view v-for="tab in 3" :key="tab" class="fi-skeleton-block insights-skeleton__tab" />
      </view>
      <view class="fi-skeleton-block insights-skeleton__caption" />
      <view v-for="row in 3" :key="row" class="insights-skeleton__row">
        <view class="fi-skeleton-heading">
          <view class="fi-skeleton-block insights-skeleton__avatar" />
          <view class="fi-skeleton-copy">
            <view class="fi-skeleton-block insights-skeleton__name" />
            <view class="fi-skeleton-block insights-skeleton__label" />
          </view>
          <view class="fi-skeleton-block insights-skeleton__share" />
        </view>
        <view class="fi-skeleton-block insights-skeleton__bar" />
      </view>
    </view>
  </view>
</template>

<style scoped lang="css">
@import '../../../styles/fi-skeleton.css';

.insights-skeleton { display: grid; gap: var(--fi-space-22); padding: 0; border: 0; border-radius: 0; background: transparent; box-shadow: none; }
.insights-skeleton__chips { display: flex; gap: var(--fi-space-10); overflow: hidden; }
.insights-skeleton__chip { flex: 0 0 38%; height: var(--fi-space-72); }
.insights-skeleton__link { width: 22%; height: var(--fi-space-32); }
.insights-skeleton__metrics { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: var(--fi-space-12); }
.insights-skeleton__metric { display: grid; gap: var(--fi-space-20); padding: var(--fi-space-18) var(--fi-space-20); border-radius: var(--fi-radius-md); background: var(--fi-color-page); }
.insights-skeleton__label { width: 40%; height: var(--fi-space-22); }
.insights-skeleton__value { width: 45%; height: var(--fi-space-56); }
.insights-skeleton__tabs { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: var(--fi-space-6); padding: var(--fi-space-6); border: var(--fi-border-width) solid var(--fi-color-border-chip); border-radius: var(--fi-radius-round); background: var(--fi-color-page); }
.insights-skeleton__tab { height: var(--fi-space-64); }
.insights-skeleton__caption { width: 72%; }
.insights-skeleton__row { display: grid; gap: var(--fi-space-10); }
.insights-skeleton__avatar { width: var(--fi-space-48); height: var(--fi-space-48); flex-shrink: 0; }
.insights-skeleton__name { width: 60%; height: var(--fi-space-28); }
.insights-skeleton__share { width: 14%; height: var(--fi-space-22); }
.insights-skeleton__bar { height: var(--fi-space-14); }
@media (prefers-reduced-motion: reduce) {
  .fi-skeleton-panel::before { animation: none; }
}
</style>
