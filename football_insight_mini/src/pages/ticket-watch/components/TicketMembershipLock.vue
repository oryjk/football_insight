<template>
  <view class="panel membership-lock-panel">
    <view class="section-heading section-heading--compact">
      <view>
        <text class="section-kicker">会员权益已暂停</text>
        <text class="section-title">当前账号已取关公众号</text>
      </view>
      <text class="meta-pill">重新关注后恢复</text>
    </view>

    <text class="membership-lock-panel__copy">
      回流看板属于会员权益。你当前的会员等级不会变化，也不会影响你已推荐的人；重新关注公众号后，返回小程序刷新即可恢复。
    </text>

    <button class="membership-lock-panel__action" @tap="emit('login')">
      去我的页查看
    </button>
  </view>
</template>

<script setup lang="ts">
const emit = defineEmits<{ (e: 'login'): void }>()
</script>

<style scoped lang="css">
@import '../ticket-watch-shared.css';

.membership-lock-panel {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-18);
}

.membership-lock-panel__copy {
  color: var(--fi-color-ticket-membership-lock-panel-copy-color);
  font-size: var(--fi-font-26);
  line-height: var(--fi-leading-relaxed);
}

.membership-lock-panel__action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  align-self: flex-start;
  min-width: 220rpx;
  min-height: 76rpx;
  padding: 0 var(--fi-space-28);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-text-primary);
  color: var(--fi-color-bg);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
  text-align: center;
}

.membership-lock-panel__action::after {
  border: none;
}
</style>
