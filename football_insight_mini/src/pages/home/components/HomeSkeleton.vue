<template>
  <!-- 骨架与真实首屏逐块对齐（今日话题 / 我的主队 / 最近赛果 / 积分榜 / 射手榜），
       高度取自 390px 宽真机视口的实测值，数据落定后内容不再整体跳动。 -->
  <view class="skeleton-stack">
    <view class="skeleton-panel-card skeleton-panel skeleton-block--topics">
      <view class="skeleton-heading">
        <view class="skeleton-line skeleton-line--section" />
        <view class="skeleton-line skeleton-line--kicker" />
      </view>
      <view class="skeleton-topic skeleton-panel">
        <view class="skeleton-line skeleton-line--kicker" />
        <view class="skeleton-line skeleton-line--title" />
        <view class="skeleton-line skeleton-line--body" />
        <view class="skeleton-topic__footer">
          <view class="skeleton-line skeleton-line--topic-heat" />
          <view class="skeleton-line skeleton-line--topic-action" />
        </view>
      </view>
    </view>

    <view class="skeleton-panel-card skeleton-panel skeleton-block--support">
      <view class="skeleton-line skeleton-line--section" />
      <view class="skeleton-line skeleton-line--body" />
      <view class="skeleton-line skeleton-line--body skeleton-line--body-short" />
    </view>

    <view class="skeleton-panel-card skeleton-panel skeleton-block--story">
      <view class="skeleton-line skeleton-line--kicker" />
      <view class="skeleton-line skeleton-line--title" />
      <view class="skeleton-line skeleton-line--title skeleton-line--title-short" />
      <view class="skeleton-line skeleton-line--body" />
      <view class="skeleton-line skeleton-line--body skeleton-line--body-short" />
      <view v-for="i in 3" :key="`skeleton-match-${i}`" class="skeleton-score-strip">
        <view class="skeleton-line skeleton-line--meta skeleton-line--meta-wide" />
        <view class="skeleton-score-body">
          <view class="skeleton-line skeleton-line--score-team" />
          <view class="skeleton-line skeleton-line--score-value" />
          <view class="skeleton-line skeleton-line--score-team" />
        </view>
      </view>
      <view class="skeleton-line skeleton-line--body" />
      <view class="skeleton-line skeleton-line--body skeleton-line--body-short" />
    </view>

    <view
      v-for="list in 2"
      :key="`skeleton-ranking-panel-${list}`"
      class="skeleton-panel-card skeleton-panel"
      :class="list === 1 ? 'skeleton-standings skeleton-block--standings' : 'skeleton-block--scorers'"
    >
      <view class="skeleton-heading">
        <view class="skeleton-copy-group">
          <view class="skeleton-line skeleton-line--kicker" />
          <view class="skeleton-line skeleton-line--section" />
          <view class="skeleton-line skeleton-line--meta" />
        </view>
      </view>
      <view class="skeleton-ranking-list">
        <view v-for="i in (list === 1 ? 4 : 5)" :key="`skeleton-standing-${list}-${i}`" class="skeleton-ranking-row">
          <view class="skeleton-line skeleton-line--rank" />
          <view class="skeleton-line skeleton-line--avatar" />
          <view class="skeleton-ranking-body">
            <view class="skeleton-line skeleton-line--name" />
            <view v-if="list !== 1" class="skeleton-line skeleton-line--note" />
          </view>
          <view class="skeleton-line skeleton-line--form" />
          <view class="skeleton-line skeleton-line--points" />
        </view>
      </view>
    </view>
  </view>
</template>

<style scoped lang="css">
.skeleton-stack {
  display: grid;
  gap: var(--fi-space-card-gap);
}

.skeleton-topic {
  display: grid;
  gap: var(--fi-space-12);
  padding: var(--fi-space-32) var(--fi-space-28, var(--fi-space-28));
  border-radius: var(--fi-radius-topic-hero);
  background: var(--fi-gradient-topic-hero);
}

.skeleton-topic .skeleton-line {
  margin-top: 0;
  background: var(--fi-color-topic-hero-control);
  border-color: transparent;
}

.skeleton-topic__footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: var(--fi-space-16);
}

.skeleton-line--topic-heat {
  width: 260rpx;
  height: 16rpx;
}

.skeleton-line--topic-action {
  width: var(--fi-space-72);
  height: 52rpx;
}

.skeleton-panel {
  position: relative;
  overflow: hidden;
}

.skeleton-panel::before {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(
    90deg,
    rgba(255, 255, 255, 0) 0%,
    rgba(255, 255, 255, 0.62) 48%,
    rgba(255, 255, 255, 0) 100%
  );
  transform: translateX(-100%);
  animation: skeleton-shimmer 1.35s ease-in-out infinite;
  pointer-events: none;
}

.skeleton-hero-card {
  padding: 26rpx var(--fi-space-24) var(--fi-space-20);
  border-radius: var(--fi-radius-xl);
  background: rgba(255, 255, 255, 0.96);
  border: 2rpx solid rgba(229, 231, 236, 0.98);
  box-shadow: 0 18rpx 46rpx rgba(26, 28, 36, 0.07);
}

.skeleton-hero-guide {
  margin-top: var(--fi-space-24);
  padding-top: var(--fi-space-24);
  border-top: 2rpx solid #ececf1;
}

.skeleton-line--guide-title {
  width: 220rpx;
  height: 36rpx;
  margin-bottom: var(--fi-space-12);
}

.skeleton-briefing-card {
  padding: var(--fi-space-20);
  border-radius: var(--fi-radius-xl);
  background: rgba(255, 255, 255, 0.96);
  border: 2rpx solid rgba(229, 231, 236, 0.98);
  box-shadow: 0 18rpx 46rpx rgba(26, 28, 36, 0.07);
}

.skeleton-briefing-row {
  margin-top: var(--fi-space-18);
  padding: var(--fi-space-20) 0 var(--fi-space-20) var(--fi-space-20);
  border-bottom: 2rpx solid rgba(231, 232, 238, 0.88);
  display: grid;
  gap: var(--fi-space-12);
}

.skeleton-briefing-row:last-child {
  border-bottom: none;
}

.skeleton-panel-card {
  padding: var(--fi-space-20);
  border-radius: var(--fi-radius-xl);
  background: rgba(255, 255, 255, 0.94);
  border: 2rpx solid rgba(236, 236, 241, 0.95);
  box-shadow: 0 20rpx 48rpx rgba(26, 28, 36, 0.06);
}

.skeleton-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-12);
}

.skeleton-copy-group {
  display: grid;
  gap: var(--fi-space-12);
  width: 100%;
}

.skeleton-line,
.skeleton-pill,
.skeleton-button {
  background: linear-gradient(180deg, rgba(242, 244, 248, 0.98), rgba(233, 237, 243, 0.92));
  border: 2rpx solid rgba(230, 233, 239, 0.95);
}

.skeleton-line {
  border-radius: var(--fi-radius-round);
}

.skeleton-line--kicker {
  width: 168rpx;
  height: 24rpx;
}

.skeleton-line--title {
  width: 430rpx;
  max-width: 100%;
  height: 56rpx;
  border-radius: var(--fi-radius-md);
}

.skeleton-line--section {
  width: 320rpx;
  max-width: 100%;
  height: 40rpx;
  border-radius: var(--fi-radius-md);
}

.skeleton-line--hero-title {
  width: 560rpx;
  max-width: 100%;
  height: 64rpx;
  border-radius: var(--fi-radius-md);
}

.skeleton-line--meta {
  margin-top: var(--fi-space-18);
  height: 24rpx;
}

.skeleton-line--meta-wide {
  width: 72%;
}

.skeleton-line--body {
  width: 88%;
  height: 26rpx;
  margin-top: var(--fi-space-18);
}

.skeleton-line--body-short {
  width: 56%;
}

.skeleton-line--briefing-value {
  width: 260rpx;
  height: 32rpx;
}

.skeleton-line--briefing-metric {
  width: 180rpx;
  height: 24rpx;
}

.skeleton-pill {
  width: 150rpx;
  height: 52rpx;
  border-radius: var(--fi-radius-round);
  flex-shrink: 0;
}

.skeleton-pill--short {
  width: 120rpx;
}

.skeleton-button {
  width: 100%;
  height: 72rpx;
  margin-top: var(--fi-space-18);
  border-radius: var(--fi-radius-round);
}

.skeleton-score-strip {
  padding: var(--fi-space-24) var(--fi-space-22);
  display: grid;
  gap: var(--fi-space-14);
  border-radius: var(--fi-radius-lg);
  border: 2rpx solid #ececf1;
  background: linear-gradient(180deg, rgba(249, 249, 252, 0.9), rgba(255, 255, 255, 1));
  margin-top: var(--fi-space-16);
}

.skeleton-score-body {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
  align-items: center;
  gap: var(--fi-space-18);
}

.skeleton-line--score-team {
  width: 100%;
  height: 30rpx;
}

.skeleton-line--score-value {
  width: 120rpx;
  height: 60rpx;
  border-radius: 20rpx;
}

.skeleton-ranking-list {
  margin-top: var(--fi-space-12);
  display: grid;
  gap: var(--fi-space-10);
}

.skeleton-ranking-row {
  display: grid;
  grid-template-columns: var(--fi-space-32) var(--fi-space-44) minmax(0, calc(var(--fi-font-28) * 4.5)) minmax(0, 1fr) var(--fi-space-72);
  gap: var(--fi-space-8);
  align-items: center;
  padding: var(--fi-space-8) 0;
}

.skeleton-standings .skeleton-ranking-row {
  display: grid;
  grid-template-columns: var(--fi-space-32) var(--fi-space-44) minmax(0, calc(var(--fi-font-28) * 4.5)) minmax(0, 1fr) var(--fi-space-56);
  gap: var(--fi-space-8);
  padding: var(--fi-space-20) 0 var(--fi-space-16);
}

.skeleton-standings .skeleton-line--rank {
  width: var(--fi-space-32);
}

.skeleton-standings .skeleton-line--avatar {
  width: var(--fi-space-44);
  height: var(--fi-space-44);
}

.skeleton-standings .skeleton-line--name,
.skeleton-standings .skeleton-line--points {
  width: 100%;
}

.skeleton-line--form {
  width: 100%;
  height: var(--fi-space-44);
}

.skeleton-line--rank {
  width: var(--fi-space-32);
  height: var(--fi-font-24);
}

.skeleton-line--avatar {
  width: var(--fi-space-44);
  height: var(--fi-space-44);
  border-radius: var(--fi-radius-round);
}

.skeleton-ranking-body {
  flex: 1;
  min-width: 0;
  display: grid;
  gap: var(--fi-space-8);
}

.skeleton-line--name {
  width: 240rpx;
  max-width: 100%;
  height: var(--fi-font-28);
}

.skeleton-line--note {
  width: 160rpx;
  max-width: 100%;
  height: var(--fi-font-22);
}

.skeleton-line--points {
  width: var(--fi-space-72);
  height: var(--fi-font-36);
  justify-self: end;
}

/* 高度取自 390px 视口实测（rpx = px × 750 / 390），内容多于骨架时仍可撑开。 */
.skeleton-block--topics { min-height: 464rpx; }
.skeleton-block--support { min-height: 262rpx; }
.skeleton-block--story { min-height: 1460rpx; }
.skeleton-block--standings { min-height: 685rpx; }
.skeleton-block--scorers { min-height: 729rpx; }

.skeleton-block--topics .skeleton-topic {
  margin-top: var(--fi-space-16);
  min-height: 320rpx;
}

.skeleton-block--story .skeleton-score-strip { min-height: 244rpx; align-content: center; }

.skeleton-block--standings .skeleton-ranking-row { min-height: 104rpx; }
.skeleton-block--scorers .skeleton-ranking-row { min-height: 100rpx; }

.skeleton-block--scorers .skeleton-line--name,
.skeleton-block--scorers .skeleton-line--note {
  width: 100%;
}

.skeleton-block--scorers .skeleton-ranking-body {
  align-content: center;
  align-self: stretch;
}

.skeleton-line--title-short {
  width: 300rpx;
  margin-top: var(--fi-space-12);
}

@keyframes skeleton-shimmer {
  0% {
    transform: translateX(-100%);
  }
  100% {
    transform: translateX(100%);
  }
}
</style>
