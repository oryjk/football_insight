<template>
  <view class="fi-panel" :class="{ 'fi-panel--flush': flush, 'fi-panel--clip': clip }">
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

.fi-panel--flush {
  padding: 0;
}

.fi-panel--clip {
  overflow: hidden;
}
</style>
