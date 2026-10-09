<template>
<view class="panel">
          <view class="section-heading">
            <view>
              <text class="section-kicker">Season Progress</text>
              <text class="section-title">赛季进度</text>
            </view>
            <text class="meta-note">{{ seasonProgressSummary.completedRounds }} / {{ seasonProgressSummary.totalRounds }} 轮已完赛</text>
          </view>

          <view class="season-progress">
            <view
              v-for="(row, rowIndex) in seasonProgressRows"
              :key="rowIndex"
              class="season-progress__row"
            >
              <view class="season-progress__track" />
              <view
                class="season-progress__fill"
                :style="{ width: row.fillWidth }"
              />
              <view
                v-for="round in row.rounds"
                :key="round.round_number"
                class="season-progress__item"
                :class="`season-progress__item--${round.status}`"
                @click="emit('open-round', round.round_number)"
              >
                <button
                  class="season-progress__dot"
                  :class="[
                    `season-progress__dot--${round.status}`,
                    { 'season-progress__dot--selected': selectedRoundNumber === round.round_number },
                  ]"
                >
                  <view class="season-progress__dot-inner" />
                </button>
                <text class="season-progress__dot-number">{{ round.round_number }}</text>
              </view>
            </view>
          </view>

        </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { RoundReference } from '../../../types/insight'
import { buildSeasonProgressRows } from '../presentation'

const props = defineProps<{
  rounds: RoundReference[]
  selectedRoundNumber: number | null
}>()
const emit = defineEmits<{ (event: 'open-round', roundNumber: number): void }>()
const seasonProgressRows = computed(() => buildSeasonProgressRows(props.rounds))
const seasonProgressSummary = computed(() => ({
  completedRounds: props.rounds.filter((round) => round.status === 'completed').length,
  totalRounds: props.rounds.length,
}))
</script>

<style scoped lang="css">
@import '../matches-shared.css';

@keyframes dot-blink {
  0%, 100% { opacity: 1; transform: scale(1.3); }
  50% { opacity: 0.35; transform: scale(1.05); }
}

@keyframes dot-blink-strong {
  0%, 100% { opacity: 1; transform: scale(1.5); }
  50% { opacity: 0.35; transform: scale(1.15); }
}
.season-progress {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-32);
  margin-top: var(--fi-space-22);
  padding: var(--fi-space-8) 0 var(--fi-space-28);
}
.season-progress__row {
  position: relative;
  display: flex;
  justify-content: space-between;
  gap: var(--fi-space-8);
  padding: var(--fi-space-8) 0;
}
.season-progress__track {
  position: absolute;
  top: var(--fi-space-18);
  left: 0;
  right: 0;
  height: var(--fi-space-8);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-season-progress-track-background);
  z-index: 0;
}
.season-progress__fill {
  position: absolute;
  top: var(--fi-space-18);
  left: 0;
  height: var(--fi-space-8);
  border-radius: var(--fi-radius-round);
  background: var(--fi-component-matches-season-progress-fill-background);
  z-index: 1;
  transition: width 600ms cubic-bezier(0.22, 1, 0.36, 1);
}
.season-progress__item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--fi-space-8);
  z-index: 2;
  flex: 1;
  min-width: 0;
}
.season-progress__dot {
  width: var(--fi-space-28);
  height: var(--fi-space-28);
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
}
.season-progress__dot-inner {
  width: var(--fi-space-22);
  height: var(--fi-space-22);
  border-radius: var(--fi-radius-season-progress-dot-inner-border-radius);
  border: none;
  background: transparent;
  background-image: url("data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCA0MCA0MCI+PGNpcmNsZSBjeD0iMjAiIGN5PSIyMCIgcj0iMTguNSIgZmlsbD0iI2ZmZmZmZiIgc3Ryb2tlPSIjZDBkNGRjIiBzdHJva2Utd2lkdGg9IjEuNSIvPjxwYXRoIGQ9Ik0yMCwxMS41IEwyOC4xLDE3LjQgTDI1LDI2LjkgTDE1LDI2LjkgTDExLjksMTcuNCBaIiBmaWxsPSIjZDBkNGRjIi8+PHBhdGggZD0iTTIwLDExLjUgTDIwLDEuNSBNMjguMSwxNy40IEwzNy42LDE0LjMgTTI1LDI2LjkgTDMwLjksMzUgTTE1LDI2LjkgTDkuMSwzNSBNMTEuOSwxNy40IEwyLjQsMTQuMyIgc3Ryb2tlPSIjZDBkNGRjIiBzdHJva2Utd2lkdGg9IjIiIGZpbGw9Im5vbmUiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIvPjxwYXRoIGQ9Ik0yMCwxLjUgUTMwLDAgMzcuNiwxNC4zIE0zNy42LDE0LjMgUTQwLDI1IDMwLjksMzUgTTMwLjksMzUgUTIwLDQwIDkuMSwzNSBNOS4xLDM1IFEwLDI1IDIuNCwxNC4zIE0yLjQsMTQuMyBRMTAsMCAyMCwxLjUiIHN0cm9rZT0iI2QwZDRkYyIgc3Ryb2tlLXdpZHRoPSIxLjUiIGZpbGw9Im5vbmUiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgb3BhY2l0eT0iMC40Ii8+PC9zdmc+Cg==");
  background-size: cover;
  background-position: center;
  transition: all 200ms ease;
}
.season-progress__dot--completed .season-progress__dot-inner {
  background-image: url("data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCA0MCA0MCI+PGNpcmNsZSBjeD0iMjAiIGN5PSIyMCIgcj0iMTkiIGZpbGw9IiMxNTE2MWIiLz48cGF0aCBkPSJNMjAsMTEuNSBMMjguMSwxNy40IEwyNSwyNi45IEwxNSwyNi45IEwxMS45LDE3LjQgWiIgZmlsbD0iI2ZmZmZmZiIvPjxwYXRoIGQ9Ik0yMCwxMS41IEwyMCwxIE0yOC4xLDE3LjQgTDM4LjEsMTQuMSBNMjUsMjYuOSBMMzEuMiwzNS40IE0xNSwyNi45IEw4LjgsMzUuNCBNMTEuOSwxNy40IEwxLjksMTQuMSIgc3Ryb2tlPSIjZmZmZmZmIiBzdHJva2Utd2lkdGg9IjIuNSIgZmlsbD0ibm9uZSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIi8+PHBhdGggZD0iTTIwLDEgUTMwLDAgMzguMSwxNC4xIE0zOC4xLDE0LjEgUTQwLDI1IDMxLjIsMzUuNCBNMzEuMiwzNS40IFEyMCw0MCA4LjgsMzUuNCBNOC44LDM1LjQgUTAsMjUgMS45LDE0LjEgTTEuOSwxNC4xIFExMCwwIDIwLDEiIHN0cm9rZT0iI2ZmZmZmZiIgc3Ryb2tlLXdpZHRoPSIxLjUiIGZpbGw9Im5vbmUiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgb3BhY2l0eT0iMC40NSIvPjwvc3ZnPgo=");
}
.season-progress__dot--current .season-progress__dot-inner {
  background-image: url("data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCA0MCA0MCI+PGNpcmNsZSBjeD0iMjAiIGN5PSIyMCIgcj0iMTkiIGZpbGw9IiNkYzI2MjYiLz48cGF0aCBkPSJNMjAsMTEuNSBMMjguMSwxNy40IEwyNSwyNi45IEwxNSwyNi45IEwxMS45LDE3LjQgWiIgZmlsbD0iI2ZmZmZmZiIvPjxwYXRoIGQ9Ik0yMCwxMS41IEwyMCwxIE0yOC4xLDE3LjQgTDM4LjEsMTQuMSBNMjUsMjYuOSBMMzEuMiwzNS40IE0xNSwyNi45IEw4LjgsMzUuNCBNMTEuOSwxNy40IEwxLjksMTQuMSIgc3Ryb2tlPSIjZmZmZmZmIiBzdHJva2Utd2lkdGg9IjIuNSIgZmlsbD0ibm9uZSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIi8+PHBhdGggZD0iTTIwLDEgUTMwLDAgMzguMSwxNC4xIE0zOC4xLDE0LjEgUTQwLDI1IDMxLjIsMzUuNCBNMzEuMiwzNS40IFEyMCw0MCA4LjgsMzUuNCBNOC44LDM1LjQgUTAsMjUgMS45LDE0LjEgTTEuOSwxNC4xIFExMCwwIDIwLDEiIHN0cm9rZT0iI2ZmZmZmZiIgc3Ryb2tlLXdpZHRoPSIxLjUiIGZpbGw9Im5vbmUiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgb3BhY2l0eT0iMC40NSIvPjwvc3ZnPgo=");
  box-shadow: var(--fi-shadow-season-progress-dot-current-season-progress-dot-inner-box-shadow);
  animation: dot-blink 1.2s ease-in-out infinite;
}
.season-progress__dot--selected .season-progress__dot-inner {
  transform: scale(1.3);
}
.season-progress__dot--current.season-progress__dot--selected .season-progress__dot-inner {
  box-shadow: var(--fi-shadow-season-progress-dot-current-season-progress-dot-selected-season-p-box-shadow);
  animation: dot-blink-strong 1.2s ease-in-out infinite;
}
.season-progress__dot-number {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-18);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-tight);
  text-align: center;
}
.season-progress__item--completed .season-progress__dot-number {
  color: var(--fi-color-text-primary);
}
.season-progress__item--current .season-progress__dot-number {
  color: var(--fi-color-primary);
}
</style>
