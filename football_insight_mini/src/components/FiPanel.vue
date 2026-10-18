<template>
  <view class="fi-panel" :class="{ 'fi-panel--flush': flush, 'fi-panel--clip': clip, 'fi-panel--dark': tone === 'dark' }">
    <slot />
  </view>
</template>

<script setup lang="ts">
/**
 * 通用内容卡片：只负责卡片外观（背景/圆角/边框/阴影/内边距）。
 * 标题区用 FiSectionHeading 放进默认插槽；父组件需要单独定制的内容（遮罩、模糊等）
 * 也放进插槽——插槽内容由父组件编译，父级 scoped 样式才能生效（小程序组件样式隔离）。
 */
defineProps<{
  /** 去掉内边距，内容自行控制留白（如整卡预览图）。 */
  flush?: boolean
  /** 裁剪溢出内容，用于卡内绝对定位的遮罩层。 */
  clip?: boolean
  /**
   * 深色主卡。外观必须由本组件自己的根节点承担：小程序里写在 <FiPanel class="..."> 上的 class
   * 落在组件宿主节点而不是 .fi-panel 上，背景不会生效（H5 会合并，所以只在 H5 看不出问题）。
   */
  tone?: 'dark'
}>()
</script>

<style scoped lang="css">
.fi-panel {
  position: relative;
  padding: var(--fi-component-panel-padding);
  border-radius: var(--fi-component-panel-radius);
  border: var(--fi-component-panel-border);
  background: var(--fi-component-panel-bg);
  box-shadow: var(--fi-component-panel-shadow);
}

.fi-panel--dark {
  background: var(--fi-gradient-topic-hero);
  border-color: rgba(255, 255, 255, 0.06);
  box-shadow: 0 10rpx 28rpx rgba(18, 20, 28, 0.18);
}

.fi-panel--flush {
  padding: 0;
}

.fi-panel--clip {
  overflow: hidden;
}
</style>
