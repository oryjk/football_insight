<template>
  <view class="fi-section-heading" :class="`fi-section-heading--${align}`">
    <view class="fi-section-heading__main">
      <slot name="heading">
        <text v-if="kicker" class="fi-section-heading__kicker">{{ kicker }}</text>
        <text v-if="title" class="fi-section-heading__title" :class="`fi-section-heading__title--${size}`">{{ title }}</text>
      </slot>
    </view>
    <slot name="meta" />
  </view>
</template>

<script setup lang="ts">
/**
 * 区块标题：小标题（kicker）+ 主标题，右侧 meta 插槽放徽标/按钮。
 * 主标题字号三档：md（默认 44）、lg（50，榜单类）、xl（56，页面 hero）。
 * 标题需要富文本（高亮、拼接）时用 heading 插槽整体替换左侧内容。
 */
withDefaults(defineProps<{
  kicker?: string
  title?: string
  size?: 'md' | 'lg' | 'xl'
  align?: 'start' | 'center'
}>(), {
  kicker: '',
  title: '',
  size: 'md',
  align: 'start',
})
</script>

<style scoped lang="css">
.fi-section-heading {
  display: flex;
  justify-content: space-between;
  gap: var(--fi-component-heading-gap);
}

.fi-section-heading--start {
  align-items: flex-start;
}

.fi-section-heading--center {
  align-items: center;
}

.fi-section-heading__main {
  flex: 1;
  min-width: 0;
}

.fi-section-heading__kicker {
  display: block;
  color: var(--fi-component-kicker-text);
  font-size: var(--fi-component-kicker-size);
  font-weight: var(--fi-component-kicker-weight);
  letter-spacing: var(--fi-component-kicker-tracking);
}

.fi-section-heading__title {
  display: block;
  color: var(--fi-component-title-text);
  font-size: var(--fi-component-title-size);
  line-height: var(--fi-component-title-leading);
  font-weight: var(--fi-component-title-weight);
}

.fi-section-heading__kicker + .fi-section-heading__title {
  margin-top: var(--fi-component-title-gap);
}

.fi-section-heading__title--lg {
  font-size: var(--fi-component-title-size-lg);
}

.fi-section-heading__title--xl {
  font-size: var(--fi-component-title-size-xl);
}
</style>
