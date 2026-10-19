<template>
  <view class="fi-skeleton-stack" role="status" aria-label="换座池加载中">
    <view class="fi-skeleton-panel" aria-hidden="true">
      <view class="fi-skeleton-heading">
        <view class="fi-skeleton-block seat-skeleton__headline" />
        <view class="fi-skeleton-block seat-skeleton__pill" />
      </view>
      <view class="fi-skeleton-block seat-skeleton__map" />
    </view>
    <view class="fi-skeleton-panel" aria-hidden="true">
      <view class="fi-skeleton-copy">
        <view class="fi-skeleton-block fi-skeleton-kicker" />
        <view class="fi-skeleton-block fi-skeleton-title" />
      </view>
      <view class="seat-skeleton__list">
        <view v-for="row in 3" :key="row" class="seat-skeleton__row">
          <view class="fi-skeleton-block fi-skeleton-avatar" />
          <view class="fi-skeleton-copy">
            <view class="fi-skeleton-block fi-skeleton-kicker" />
            <view class="fi-skeleton-block seat-skeleton__line" />
          </view>
        </view>
      </view>
    </view>
  </view>
</template>

<style scoped>
@import '../../../styles/fi-skeleton.css';

.seat-skeleton__headline {
  width: 56%;
  height: var(--fi-space-32);
}

.seat-skeleton__pill {
  flex-shrink: 0;
  width: 22%;
  height: var(--fi-space-40);
}

/* 约 384rpx，与首屏地图高度接近 */
.seat-skeleton__map {
  width: 100%;
  margin-top: var(--fi-space-16);
  height: calc(var(--fi-space-64) * 6);
  border-radius: var(--fi-radius-md);
}

.seat-skeleton__list {
  display: grid;
  gap: var(--fi-space-20);
  margin-top: var(--fi-space-20);
}

.seat-skeleton__row {
  display: flex;
  align-items: center;
  gap: var(--fi-space-16);
}

.seat-skeleton__line {
  width: 72%;
}
</style>
