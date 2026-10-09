<template>
<view  class="recent-reflux-lock">
      <view class="recent-reflux-lock__preview">
        <view class="recent-reflux-lock__bar recent-reflux-lock__bar--red" />
        <view class="recent-reflux-lock__bar recent-reflux-lock__bar--middle" />
        <view class="recent-reflux-lock__bar recent-reflux-lock__bar--green" />
      </view>
      <view class="recent-reflux-lock__body">
        <text class="recent-reflux-lock__title">V6 及以上会员可以查看</text>
        <text class="recent-reflux-lock__copy"
          >V6 可查看 30 分钟内，V7 可查看 10 分钟内，V8/V9 可查看 3
          分钟内的具体座位区域。</text
        >
      </view>
      <button class="recent-reflux-lock__action" @tap="emit('upgrade')">
        升级查看
      </button>
    </view>
</template>
<script setup lang="ts">
const emit = defineEmits<{ (e: 'upgrade'): void }>()
</script>
<style scoped lang="css">
.recent-reflux-lock {
  position: relative;
  overflow: hidden;
  display: grid;
  grid-template-columns: 108rpx minmax(0, 1fr);
  gap: var(--fi-space-18);
  margin-top: var(--fi-space-18);
  padding: var(--fi-space-24);
  border-radius: var(--fi-radius-ticket-20);
  background:
    linear-gradient(
      135deg,
      var(--fi-color-ticket-recent-reflux-lock-background),
      var(--fi-color-ticket-recent-reflux-lock-background-2)
    ),
    linear-gradient(
      180deg,
      var(--fi-color-ticket-recent-reflux-lock-background-3),
      var(--fi-color-ticket-recent-reflux-lock-background-4)
    );
  border: var(--fi-border-width) solid
    var(--fi-color-ticket-recent-reflux-lock-border);
}

.recent-reflux-lock::after {
  content: '';
  position: absolute;
  right: -38rpx;
  top: -48rpx;
  width: 190rpx;
  height: 190rpx;
  border-radius: var(--fi-radius-round);
  background: radial-gradient(
    circle,
    var(--fi-color-ticket-recent-reflux-lock-background-5),
    var(--fi-color-ticket-skeleton-panel-background) 68%
  );
  pointer-events: none;
}
.recent-reflux-lock__preview {
  position: relative;
  z-index: 1;
  height: 112rpx;
  border-radius: var(--fi-radius-sm);
  background: var(--fi-color-ticket-recent-reflux-lock-preview-background);
  border: var(--fi-border-width) solid
    var(--fi-color-ticket-recent-reflux-lock-preview-border);
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--fi-space-8);
}

.recent-reflux-lock__bar {
  width: 14rpx;
  border-radius: var(--fi-radius-round);
  box-shadow: var(--fi-shadow-ticket-recent-reflux-lock-bar);
}

.recent-reflux-lock__bar--red {
  height: 74rpx;
  color: var(--fi-color-ticket-recent-reflux-lock-bar-red-color);
  background: var(--fi-color-ticket-recent-reflux-lock-bar-red-background);
}

.recent-reflux-lock__bar--middle {
  height: 56rpx;
  color: var(--fi-color-ticket-recent-reflux-lock-bar-middle-color);
  background: var(--fi-color-ticket-recent-reflux-lock-bar-middle-background);
}

.recent-reflux-lock__bar--green {
  height: 42rpx;
  color: var(--fi-color-ticket-recent-reflux-lock-bar-green-color);
  background: var(--fi-color-ticket-recent-reflux-lock-bar-green-background);
}

.recent-reflux-lock__body {
  position: relative;
  z-index: 1;
  min-width: 0;
}

.recent-reflux-lock__title {
  display: block;
  color: var(--fi-color-ticket-recent-reflux-lock-title-color);
  font-size: var(--fi-font-30);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-snug);
}

.recent-reflux-lock__copy {
  display: block;
  margin-top: var(--fi-space-10);
  color: var(--fi-color-ticket-recent-reflux-lock-copy-color);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-normal);
}

.recent-reflux-lock__action {
  position: relative;
  z-index: 1;
  grid-column: 1 / -1;
  height: 72rpx;
  line-height: 72rpx;
  border-radius: var(--fi-radius-round);
  background: linear-gradient(
    135deg,
    var(--fi-color-ticket-recent-reflux-lock-action-background),
    var(--fi-color-ticket-recent-reflux-lock-action-background-2)
  );
  color: var(--fi-color-bg);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-extrabold);
  text-align: center;
}

.recent-reflux-lock__action::after {
  border: none;
}
</style>
