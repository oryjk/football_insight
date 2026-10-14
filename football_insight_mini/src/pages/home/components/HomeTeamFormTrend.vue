<template>
  <view v-if="form.points.length" class="team-form" role="img" :aria-label="form.description">
    <view class="team-form__plot" aria-hidden="true">
      <view class="team-form__baseline" />
      <view
        v-for="(segment, index) in form.segments"
        :key="`line-${index}`"
        class="team-form__segment"
        :class="[`team-form__segment--${segment.direction}`, { 'team-form__segment--latest': segment.latest }]"
        :style="{
          left: `${segment.left}%`,
          top: `${segment.top}%`,
          width: `${segment.width}%`,
          ...(segment.height ? { height: `${segment.height}%` } : {}),
        }"
      />
      <view
        v-for="(point, index) in form.points"
        :key="`point-${index}`"
        class="team-form__point"
        :class="{ 'team-form__point--latest': point.latest }"
        :style="{ left: `${point.left}%`, top: `${point.top}%` }"
      />
    </view>
    <text class="team-form__latest" aria-hidden="true">{{ form.latestLabel }}</text>
  </view>
  <text v-else class="team-form__empty">{{ outcomes ? '暂无已完赛战绩' : '近期战绩待更新' }}</text>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { MatchOutcome } from '../../../types/insight'
import { buildHomeTeamForm } from '../teamForm'

const props = defineProps<{ outcomes?: MatchOutcome[] }>()
const form = computed(() => buildHomeTeamForm(props.outcomes))
</script>

<style scoped>
.team-form {
  display: flex;
  align-items: center;
  gap: var(--fi-space-16);
  width: 100%;
  min-width: 0;
}
.team-form__plot {
  position: relative;
  height: var(--fi-space-56);
  margin: 0 var(--fi-space-8);
  flex: 1;
  min-width: 0;
}
.team-form__baseline {
  position: absolute;
  left: 0;
  right: 0;
  top: 50%;
  height: var(--fi-space-1);
  background: var(--fi-color-divider-soft);
}
.team-form__segment {
  --form-stroke: var(--fi-color-team-form-line);
  position: absolute;
  pointer-events: none;
}
.team-form__segment--latest {
  --form-stroke: var(--fi-color-primary);
}
.team-form__segment--flat {
  height: var(--fi-space-2);
  transform: translateY(-50%);
  background: var(--form-stroke);
}
.team-form__segment--up {
  background: linear-gradient(to bottom right, transparent calc(50% - var(--fi-space-1)), var(--form-stroke) calc(50% - var(--fi-space-1)), var(--form-stroke) calc(50% + var(--fi-space-1)), transparent calc(50% + var(--fi-space-1)));
}
.team-form__segment--down {
  background: linear-gradient(to top right, transparent calc(50% - var(--fi-space-1)), var(--form-stroke) calc(50% - var(--fi-space-1)), var(--form-stroke) calc(50% + var(--fi-space-1)), transparent calc(50% + var(--fi-space-1)));
}
.team-form__point {
  position: absolute;
  width: var(--fi-space-8);
  height: var(--fi-space-8);
  box-sizing: border-box;
  transform: translate(-50%, -50%);
  border: var(--fi-border-width) solid var(--fi-color-team-form-line);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-surface);
}
.team-form__point--latest {
  width: var(--fi-space-12);
  height: var(--fi-space-12);
  background: var(--fi-color-primary);
  border-color: var(--fi-color-primary);
  box-shadow: var(--fi-shadow-team-form-latest);
}
.team-form__latest {
  min-width: var(--fi-space-24);
  text-align: right;
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-22);
}
.team-form__empty {
  display: block;
  padding: var(--fi-space-14) 0;
  font-size: var(--fi-font-22);
  color: var(--fi-color-text-caption);
}
</style>
