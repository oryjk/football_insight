<template>
  <view class="fi-skeleton-stack" role="status" aria-label="赛程加载中">
    <view class="fi-skeleton-panel" aria-hidden="true">
      <view class="fi-skeleton-heading">
        <view class="fi-skeleton-copy">
          <view class="fi-skeleton-block fi-skeleton-kicker" />
          <view class="fi-skeleton-block fi-skeleton-title" />
        </view>
        <view class="fi-skeleton-block matches-skeleton__summary" />
      </view>
      <view class="matches-skeleton__progress">
        <view v-for="row in 2" :key="row" class="matches-skeleton__rounds">
          <view v-for="round in 15" :key="round" class="matches-skeleton__round">
            <view class="fi-skeleton-block matches-skeleton__dot" />
            <view class="fi-skeleton-block matches-skeleton__number" />
          </view>
        </view>
      </view>
    </view>
    <view class="fi-skeleton-panel" aria-hidden="true">
      <view class="fi-skeleton-copy">
        <view class="fi-skeleton-block fi-skeleton-kicker" />
        <view class="fi-skeleton-block fi-skeleton-title" />
      </view>
      <view v-for="match in 2" :key="match" class="matches-skeleton__match">
        <view class="fi-skeleton-block matches-skeleton__meta" />
        <view class="matches-skeleton__teams">
          <view class="fi-skeleton-block fi-skeleton-avatar" />
          <view class="fi-skeleton-block matches-skeleton__name" />
          <view class="fi-skeleton-block matches-skeleton__score" />
          <view class="fi-skeleton-block matches-skeleton__name" />
          <view class="fi-skeleton-block fi-skeleton-avatar" />
        </view>
      </view>
    </view>
  </view>
</template>

<style scoped lang="css">
@import '../../../styles/fi-skeleton.css';

.matches-skeleton__summary { width: 30%; }
.matches-skeleton__progress { display: grid; gap: var(--fi-space-32); margin-top: var(--fi-space-22); padding: var(--fi-space-16) 0 var(--fi-space-28); }
.matches-skeleton__rounds { display: grid; grid-template-columns: repeat(15, minmax(0, 1fr)); gap: var(--fi-space-8); }
.matches-skeleton__round { display: grid; justify-items: center; gap: var(--fi-space-8); }
.matches-skeleton__dot { width: var(--fi-space-28); height: var(--fi-space-28); }
.matches-skeleton__number { width: var(--fi-space-16); height: var(--fi-space-18); }
.matches-skeleton__match { display: grid; gap: var(--fi-space-20); margin-top: var(--fi-space-card-gap); padding: var(--fi-space-24) var(--fi-space-20); border: var(--fi-border-divider); border-radius: var(--fi-radius-lg); background: var(--fi-color-page); }
.matches-skeleton__meta { width: 48%; }
.matches-skeleton__teams { display: grid; grid-template-columns: var(--fi-space-44) minmax(0, 1fr) var(--fi-space-64) minmax(0, 1fr) var(--fi-space-44); gap: var(--fi-space-8); align-items: center; }
.matches-skeleton__name { height: var(--fi-space-28); }
.matches-skeleton__score { height: var(--fi-space-40); }
</style>
