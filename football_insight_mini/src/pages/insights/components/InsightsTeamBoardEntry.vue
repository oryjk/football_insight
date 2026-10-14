<template>
  <FiPanel>
    <FiSectionHeading kicker="球队战术板" title="把这张洞察卡变成观点">
      <template #meta>
        <button class="meta-pill meta-pill--button" @click="emit('open')">进入战术板</button>
      </template>
    </FiSectionHeading>

    <text class="board-entry__copy">
      不发普通灌水帖，只能围绕当前球队的进球贡献、助攻贡献和失球贡献发布观点。先用数据站住，再去讨论判断。
    </text>
  </FiPanel>
</template>

<script setup lang="ts">
import FiPanel from '../../../components/FiPanel.vue'
import FiSectionHeading from '../../../components/FiSectionHeading.vue'
const emit = defineEmits<{
  (e: 'open'): void
}>()
</script>
<style scoped lang="css">
.meta-pill--button {
  display: inline-flex;
  margin: 0 0 0 auto;
  align-items: center;
  justify-content: center;
  white-space: nowrap;
  line-height: var(--fi-leading-none);
  box-sizing: border-box;
  padding: var(--fi-space-14) var(--fi-space-24);
  border-radius: var(--fi-radius-round);
  border: var(--fi-primitive-border-width) solid var(--fi-color-border-chip);
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
}

.board-entry__copy {
  position: relative;
  z-index: 1;
  display: block;
  margin-top: var(--fi-space-18);
  color: #6b707b;
  font-size: var(--fi-font-28);
  line-height: var(--fi-leading-relaxed);
}
</style>
