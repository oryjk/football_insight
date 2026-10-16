<template>
  <view class="fi-skeleton-stack" role="status" aria-label="榜单加载中">
    <view class="fi-skeleton-panel standings-skeleton" aria-hidden="true">
      <view v-for="card in 2" :key="card" class="standings-skeleton__card">
        <view class="fi-skeleton-heading">
          <view class="fi-skeleton-block standings-skeleton__icon" />
          <view class="fi-skeleton-block standings-skeleton__title" />
          <view class="fi-skeleton-block standings-skeleton__chevron" />
        </view>
        <view class="standings-skeleton__podium">
          <view v-for="place in 3" :key="place" class="standings-skeleton__place">
            <view class="fi-skeleton-block standings-skeleton__rank" />
            <view class="fi-skeleton-block standings-skeleton__avatar" />
            <view class="fi-skeleton-block standings-skeleton__name" />
          </view>
        </view>
      </view>
    </view>

    <view class="fi-skeleton-panel rankings-skeleton" aria-hidden="true">
      <view class="rankings-skeleton__controls">
        <view class="rankings-skeleton__scope">
          <view v-for="scope in 2" :key="scope" class="fi-skeleton-block rankings-skeleton__scope-item" />
        </view>
        <view class="rankings-skeleton__categories">
          <view v-for="category in 5" :key="category" class="fi-skeleton-block rankings-skeleton__category" />
        </view>
      </view>
      <view>
        <view class="fi-skeleton-heading">
          <view class="fi-skeleton-copy">
            <view class="fi-skeleton-block fi-skeleton-kicker" />
            <view class="fi-skeleton-block fi-skeleton-title" />
          </view>
          <view class="fi-skeleton-block rankings-skeleton__mode" />
        </view>
        <view class="rankings-skeleton__caption fi-skeleton-heading">
          <view class="fi-skeleton-block rankings-skeleton__caption-text" />
          <view class="fi-skeleton-block rankings-skeleton__points-label" />
        </view>
        <view v-for="row in 8" :key="row" class="rankings-skeleton__row">
          <view class="fi-skeleton-block rankings-skeleton__rank" />
          <view class="fi-skeleton-block fi-skeleton-avatar" />
          <view class="fi-skeleton-block rankings-skeleton__name" />
          <view class="rankings-skeleton__results">
            <view v-for="dot in 30" :key="dot" class="fi-skeleton-block rankings-skeleton__dot" />
          </view>
          <view class="fi-skeleton-block rankings-skeleton__points" />
        </view>
      </view>
    </view>
  </view>
</template>

<style scoped lang="css">
@import '../../../styles/fi-skeleton.css';

.standings-skeleton { padding: var(--fi-space-8) var(--fi-space-20); }
.standings-skeleton__card { display: grid; gap: var(--fi-space-16); padding: var(--fi-space-20) 0; border-bottom: var(--fi-border-divider); }
.standings-skeleton__card:last-child { border-bottom: 0; }
.standings-skeleton__icon { width: var(--fi-space-64); height: var(--fi-space-64); border-radius: var(--fi-radius-md); flex-shrink: 0; }
.standings-skeleton__title { flex: 1; max-width: 52%; height: var(--fi-space-32); margin-right: auto; }
.standings-skeleton__chevron { width: var(--fi-space-24); height: var(--fi-space-24); }
.standings-skeleton__podium { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: var(--fi-space-12); }
.standings-skeleton__place { display: flex; align-items: center; gap: var(--fi-space-8); min-width: 0; padding: var(--fi-space-12); border-radius: var(--fi-radius-md); background: var(--fi-color-page); }
.standings-skeleton__rank { width: var(--fi-space-32); height: var(--fi-space-32); flex-shrink: 0; }
.standings-skeleton__avatar { width: var(--fi-space-40); height: var(--fi-space-40); flex-shrink: 0; }
.standings-skeleton__name { flex: 1; height: var(--fi-space-24); }
.rankings-skeleton { display: grid; gap: var(--fi-space-22); }
.rankings-skeleton__controls { display: grid; gap: var(--fi-space-14); padding-bottom: var(--fi-space-10); overflow: hidden; }
.rankings-skeleton__scope { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: var(--fi-space-6); padding: var(--fi-space-6); border: var(--fi-border-width) solid var(--fi-color-border-chip); border-radius: var(--fi-radius-round); background: var(--fi-color-page); }
.rankings-skeleton__scope-item { height: var(--fi-space-64); }
.rankings-skeleton__categories { display: flex; gap: var(--fi-space-10); overflow: hidden; padding-bottom: var(--fi-space-4); }
.rankings-skeleton__category { flex: 0 0 22%; height: calc(var(--fi-space-18) * 2 + var(--fi-font-24)); }
.rankings-skeleton__mode { width: 36%; height: calc(var(--fi-space-28) + var(--fi-space-24)); }
.rankings-skeleton__caption { margin-top: var(--fi-space-18); padding-bottom: var(--fi-space-12); border-bottom: var(--fi-border-width) solid var(--fi-color-divider-soft); }
.rankings-skeleton__caption-text { width: 52%; height: var(--fi-space-22); }
.rankings-skeleton__points-label { width: var(--fi-space-44); height: var(--fi-space-22); }
.rankings-skeleton__row { display: grid; grid-template-columns: var(--fi-space-32) var(--fi-space-44) minmax(0, calc(var(--fi-font-28) * 4.5)) minmax(0, 1fr) var(--fi-space-56); gap: var(--fi-space-8); align-items: center; padding: var(--fi-space-20) 0 var(--fi-space-16); border-bottom: var(--fi-border-width) solid var(--fi-color-divider-soft); }
.rankings-skeleton__rank { width: 100%; }
.rankings-skeleton__name { width: 85%; height: var(--fi-font-28); }
.rankings-skeleton__results { display: grid; grid-template-columns: repeat(15, minmax(0, 1fr)); grid-auto-rows: var(--fi-space-36); align-items: center; min-width: 0; }
.rankings-skeleton__dot { width: 100%; max-width: var(--fi-space-18); height: var(--fi-space-18); justify-self: center; }
.rankings-skeleton__points { width: 80%; height: var(--fi-font-36); justify-self: end; }
</style>
