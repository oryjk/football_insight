<template>
  <view v-if="matches.length" class="goal-bars" role="group" :aria-label="`${playerName}最近${matches.length}次出场进球，由旧到新`">
    <view
      v-for="(match, index) in matches"
      :key="match.match_id"
      class="goal-bars__cell"
      :class="{ 'goal-bars__cell--latest': index === matches.length - 1 }"
      role="button"
      :aria-label="`${match.match_date}，${match.goals}球${match.goals === 3 ? '，帽子戏法' : ''}`"
      hover-class="goal-bars__cell--pressed"
      @click.stop="emit('select', match)"
    >
      <view class="goal-bars__column">
        <text v-if="match.goals === 3" class="goal-bars__hat">帽</text>
        <text v-else-if="match.goals >= 2" class="goal-bars__value">{{ match.goals }}</text>
        <view
          class="goal-bars__bar"
          :class="{ 'goal-bars__bar--zero': match.goals === 0 }"
          :style="match.goals ? { height: `calc(var(--fi-space-40) * ${match.goals / scaleMax})` } : undefined"
        />
      </view>
    </view>
  </view>
  <text v-else class="goal-bars__empty">{{ appearances ? '暂无逐场数据' : '逐场数据待更新' }}</text>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { PlayerMatchGoals } from '../types/insight'
import { normalizeRecentGoals } from '../utils/playerGoals'

const props = defineProps<{ playerName: string; appearances?: PlayerMatchGoals[]; scaleMax: number }>()
const matches = computed(() => normalizeRecentGoals(props.appearances))
const emit = defineEmits<{ (event: 'select', match: PlayerMatchGoals): void }>()
</script>

<style scoped>
.goal-bars {
  display: grid;
  grid-template-columns: repeat(10, minmax(0, 1fr));
  min-width: 0;
  width: 100%;
  gap: var(--fi-space-4);
  height: var(--fi-space-72);
  align-items: stretch;
}
.goal-bars__cell {
  display: flex;
  align-items: flex-end;
  justify-content: center;
  min-width: 0;
  padding: var(--fi-space-4) 0;
}
.goal-bars__cell--pressed {
  background: var(--fi-color-team-form-pressed);
  border-radius: var(--fi-radius-sm);
}
.goal-bars__column {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--fi-space-2);
  width: 100%;
  min-width: 0;
}
.goal-bars__bar {
  width: var(--fi-space-12);
  max-width: 100%;
  background: var(--fi-color-goal-bar);
  border-radius: var(--fi-space-2) var(--fi-space-2) 0 0;
}
.goal-bars__cell--latest .goal-bars__bar {
  background: var(--fi-color-primary);
}
.goal-bars__bar--zero,
.goal-bars__cell--latest .goal-bars__bar--zero {
  height: var(--fi-space-2);
  background: var(--fi-color-team-form-line);
  border-radius: var(--fi-space-2);
}
.goal-bars__value,
.goal-bars__hat {
  font-size: var(--fi-font-18);
  line-height: var(--fi-leading-tight);
  color: var(--fi-color-text-secondary);
}
.goal-bars__hat {
  color: var(--fi-color-hat-trick-text);
  background: var(--fi-color-hat-trick-bg);
  border-radius: var(--fi-space-4);
  padding: var(--fi-space-2);
  font-weight: var(--fi-weight-bold);
}
.goal-bars__empty {
  font-size: var(--fi-font-20);
  color: var(--fi-color-text-caption);
}
</style>
