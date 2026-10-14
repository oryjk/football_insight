<template>
  <button
    class="home-support-vote"
    :class="{ 'home-support-vote--supported': supported, 'home-support-vote--disabled': disabled && !supported }"
    :disabled="disabled"
    :aria-label="`${team.team_name}，${label}，${team.support_count} 次助力`"
    hover-class="home-support-vote--pressed"
    @click.stop="emit('vote', team.team_id)"
  >
    <image :src="thumbsUpIcon" mode="aspectFit" class="home-support-vote__icon" aria-hidden="true" />
    <text class="home-support-vote__label">{{ label }}</text>
    <text class="home-support-vote__count">{{ team.support_count }}</text>
  </button>
</template>

<script setup lang="ts">
import type { SupportMatchTeam } from '../../../types/support'
import thumbsUpIcon from '../../../static/home/support-thumbs-up.svg'

defineProps<{ team: SupportMatchTeam; label: string; disabled: boolean; supported: boolean }>()
const emit = defineEmits<{ (event: 'vote', teamId: number): void }>()
</script>

<style scoped>
.home-support-vote {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--fi-space-8);
  box-sizing: border-box;
  max-width: 100%;
  min-height: var(--fi-space-64);
  margin: var(--fi-space-8) 0 0;
  padding: var(--fi-space-12) var(--fi-space-16);
  border: var(--fi-border-width) solid var(--fi-color-primary);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-topic-audio-surface);
  color: var(--fi-color-primary);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-none);
}
.home-support-vote::after { border: 0; }
.home-support-vote--supported { font-weight: var(--fi-weight-bold); }
.home-support-vote--disabled { opacity: 0.45; }
.home-support-vote--pressed { background: var(--fi-color-divider-soft); }
.home-support-vote__icon {
  width: var(--fi-space-28);
  height: var(--fi-space-28);
  flex-shrink: 0;
}
.home-support-vote__label { white-space: nowrap; }
.home-support-vote__count {
  font-weight: var(--fi-weight-bold);
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
