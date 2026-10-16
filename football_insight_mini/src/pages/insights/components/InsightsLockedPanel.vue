<template>
  <FiPanel flush clip>
    <view class="insights-locked">
      <!-- mode=guest：未登录时的模糊预览 -->
      <view v-if="mode === 'guest'" class="insights-locked__preview">
        <view class="insights-locked__chips">
          <text v-for="name in previewTeams" :key="name">{{ name }}</text>
        </view>

        <view class="insights-locked__summary">
          <view class="insights-locked__metric">
            <text>总进球</text>
            <text class="insights-locked__metric-value">14</text>
            <text>赛季累计打进</text>
          </view>
          <view class="insights-locked__metric">
            <text>总失球</text>
            <text class="insights-locked__metric-value">5</text>
            <text>赛季累计丢失</text>
          </view>
        </view>

        <view class="insights-locked__rows">
          <view v-for="section in previewSections" :key="section.title" class="insights-locked__section">
            <text class="insights-locked__section-title">{{ section.title }}</text>
            <view v-for="row in section.rows" :key="row.name" class="insights-locked__row">
              <text>{{ row.name }}</text>
              <view><view :style="{ width: row.width }" /></view>
            </view>
          </view>
        </view>
      </view>

      <!-- mode=membership：会员权益暂停时的提示预览 -->
      <view v-else class="insights-locked__preview">
        <view class="insights-locked__summary">
          <view class="insights-locked__metric">
            <text>洞察权限</text>
            <text class="insights-locked__metric-value">暂停</text>
            <text>取关后自动冻结</text>
          </view>
          <view class="insights-locked__metric">
            <text>恢复方式</text>
            <text class="insights-locked__metric-value">重新关注</text>
            <text>返回这里刷新即可</text>
          </view>
        </view>
      </view>

      <view class="insights-lock-overlay">
        <view v-if="mode === 'membership'" class="insights-lock-overlay__card">
          <text class="section-kicker">会员权益已暂停</text>
          <text class="insights-lock-overlay__title">当前账号已取关公众号</text>
          <text class="insights-lock-overlay__copy">
            洞察页和回流看板属于会员权益。重新关注公众号后，回到小程序刷新即可恢复；会员等级和你已推荐的人不会受影响。
          </text>
          <button class="insights-lock-overlay__action" @click="emit('action')">去我的页查看</button>
        </view>
      </view>
    </view>
  </FiPanel>
</template>

<script setup lang="ts">
import FiPanel from '../../../components/FiPanel.vue'
import { computed } from 'vue'
import {
  INSIGHTS_PREVIEW_ASSIST_ROWS,
  INSIGHTS_PREVIEW_GOAL_AGAINST_ROWS,
  INSIGHTS_PREVIEW_GOAL_PLAYER_ROWS,
  INSIGHTS_PREVIEW_TEAMS,
} from '../helpers'

withDefaults(
  defineProps<{
    mode: 'guest' | 'membership'
  }>(),
  { mode: 'guest' },
)

const emit = defineEmits<{
  (e: 'action'): void
}>()

const previewTeams = INSIGHTS_PREVIEW_TEAMS
const previewSections = computed(() => [
  { title: '进球贡献 · 对手维度', rows: INSIGHTS_PREVIEW_GOAL_AGAINST_ROWS },
  { title: '进球贡献 · 球员维度', rows: INSIGHTS_PREVIEW_GOAL_PLAYER_ROWS },
  { title: '助攻贡献 · 球员维度', rows: INSIGHTS_PREVIEW_ASSIST_ROWS },
])
</script>

<style scoped lang="css">
.insights-locked {
  position: relative;
  min-height: var(--fi-space-insights-preview-height);
}

.insights-locked__preview {
  padding: var(--fi-space-20);
  filter: blur(var(--fi-space-4));
  opacity: 0.9;
}

.insights-locked__chips {
  display: flex;
  flex-wrap: wrap;
  gap: var(--fi-space-12);
}

.insights-locked__chips text {
  padding: var(--fi-space-14) var(--fi-space-22);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
}

.insights-locked__summary {
  margin-top: var(--fi-space-20);
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--fi-space-16);
}

.insights-locked__metric {
  display: grid;
  gap: var(--fi-space-10);
  padding: var(--fi-space-22);
  border-radius: var(--fi-radius-md);
  background: var(--fi-color-insights-preview-metric);
  color: var(--fi-color-text-caption);
  font-size: var(--fi-font-22);
}

.insights-locked__metric-value {
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-56);
  font-weight: var(--fi-weight-extrabold);
}

.insights-locked__rows {
  margin-top: var(--fi-space-20);
  display: grid;
  gap: var(--fi-space-16);
}

.insights-locked__section {
  display: grid;
  gap: var(--fi-space-12);
  padding: var(--fi-space-22);
  border-radius: var(--fi-radius-md);
  background: var(--fi-color-insights-preview-surface);
}

.insights-locked__section-title {
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
}

.insights-locked__row {
  display: grid;
  grid-template-columns: var(--fi-space-insights-preview-column) 1fr;
  gap: var(--fi-space-16);
  align-items: center;
}

.insights-locked__row text {
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-22);
}

.insights-locked__row > view {
  height: var(--fi-space-14);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-insights-track);
  overflow: hidden;
}

.insights-locked__row > view > view {
  height: 100%;
  border-radius: inherit;
  background: var(--fi-color-text-primary);
}

.insights-lock-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--fi-space-28);
  background: var(--fi-color-insights-preview-overlay);
}

.insights-lock-overlay__card {
  width: 100%;
  padding: var(--fi-space-30) var(--fi-space-28);
  border-radius: var(--fi-radius-insights-lock);
  background: var(--fi-color-insights-lock-surface);
  backdrop-filter: blur(var(--fi-space-18));
  box-shadow: var(--fi-shadow-insights-lock);
  animation: fi-sheet-up 240ms cubic-bezier(0.22, 1, 0.36, 1) both;
}

.section-kicker {
  margin: 0;
  color: var(--fi-component-kicker-text);
  font-size: var(--fi-component-kicker-size);
  font-weight: var(--fi-weight-bold);
  letter-spacing: var(--fi-space-insights-kicker-spacing);
}

.insights-lock-overlay__title {
  display: block;
  margin-top: var(--fi-space-10);
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-44);
  font-weight: var(--fi-weight-extrabold);
}

.insights-lock-overlay__copy {
  display: block;
  margin-top: var(--fi-space-18);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-28);
  line-height: var(--fi-leading-relaxed);
}

.insights-lock-overlay__action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin-top: var(--fi-space-20);
  align-self: flex-start;
  padding: var(--fi-space-20) var(--fi-space-28);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-text-primary);
  color: var(--fi-color-surface);
  font-size: var(--fi-font-28);
  white-space: nowrap;
  line-height: var(--fi-leading-none);
}
</style>
