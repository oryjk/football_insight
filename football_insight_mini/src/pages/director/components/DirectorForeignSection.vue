<template>
  <view class="foreign-card">
    <view class="foreign-card__head">
      <view class="foreign-card__title-wrap">
        <text class="foreign-card__step">STEP 1</text>
        <text class="foreign-card__title">外援名额与去留</text>
      </view>
      <text
        class="foreign-card__counter"
        :class="{ 'foreign-card__counter--over': quotaCheck.overLimit }"
      >
        {{ quotaCheck.total }} / {{ quotaCheck.quota }} 名额
      </text>
    </view>

    <view class="quota-track">
      <view
        class="quota-fill"
        :class="{ 'quota-fill--over': quotaCheck.overLimit }"
        :style="{ width: `${quotaCheck.progressPercent}%` }"
      />
    </view>
    <text v-if="quotaCheck.message" class="quota-warn">{{ quotaCheck.message }}</text>
    <text v-else class="quota-hint">
      已留 {{ foreignStayIds.length }} 名现役 + 新签 {{ foreignIns.length }} 名，还可补强 {{ quotaCheck.remaining }} 个空位
    </text>

    <view v-if="squadForeign.length" class="squad-list">
      <view v-for="player in squadForeign" :key="player.player_id" class="player-row">
        <view class="player-row__avatar">
          <FiAvatar :src="player.avatar_url" :size="96" :name="player.player_name" />
        </view>
        <view class="player-row__info">
          <view class="player-row__top">
            <text
              class="pes-pos"
              :class="`pes-pos--${resolvePesPositionBadge(player.position).zone}`"
            >
              {{ resolvePesPositionBadge(player.position).code }}
            </text>
            <text class="player-row__name">{{ player.player_name }}</text>
          </view>
          <text class="player-row__meta">
            {{ player.country }}{{ player.shirt_number ? ` · ${player.shirt_number}号` : '' }}
          </text>
        </view>
        <view class="player-row__toggle">
          <button
            class="toggle-pill"
            :class="{ 'toggle-pill--stay': isStay(player.player_id) }"
            hover-class="toggle-pill--pressed"
            @tap="emit('set-action', player.player_id, 'stay')"
          >
            留队
          </button>
          <button
            class="toggle-pill"
            :class="{ 'toggle-pill--leave': isLeave(player.player_id) }"
            hover-class="toggle-pill--pressed"
            @tap="emit('set-action', player.player_id, 'leave')"
          >
            离队
          </button>
        </view>
      </view>
    </view>
    <text v-else class="empty-hint">该队暂无登记在册的现役外援，可直接从下方签入新外援</text>

    <view class="signings-block">
      <view class="signings-block__head">
        <text class="signings-block__title">STEP 2 · 新签外援（{{ foreignIns.length }}）</text>
        <text class="signings-block__sub">重点含 K 联赛 / J 联赛 / 中超热门</text>
      </view>

      <view v-if="foreignIns.length" class="signing-list">
        <view v-for="item in foreignIns" :key="item.key" class="signing-chip">
          <view class="signing-chip__main">
            <view class="signing-chip__top">
              <text
                class="pes-pos"
                :class="`pes-pos--${resolvePesPositionBadge(item.position).zone}`"
              >
                {{ resolvePesPositionBadge(item.position).code }}
              </text>
              <text class="signing-chip__name">{{ item.player_name }}</text>
              <text class="signing-chip__tag">{{ directorLeagueLabel(item.league_tag) }}</text>
            </view>
            <text class="signing-chip__club">{{ item.current_club }}</text>
          </view>
          <button
            class="signing-chip__remove"
            hover-class="toggle-pill--pressed"
            @tap="emit('remove-in', item.key)"
          >
            移除
          </button>
        </view>
      </view>

      <button
        class="add-foreign-btn"
        :class="{ 'add-foreign-btn--disabled': quotaCheck.remaining <= 0 }"
        hover-class="toggle-pill--pressed"
        @tap="emit('open-picker')"
      >
        {{ quotaCheck.remaining > 0 ? `+ 补齐外援空位（还差 ${quotaCheck.remaining} 个）` : '5 外援名额已满（点上方离队或移除可换人）' }}
      </button>
    </view>
  </view>
</template>

<script setup lang="ts">
import FiAvatar from '../../../components/FiAvatar.vue'
import type { DirectorSquadPlayerDto, PlanPlayerItem } from '../../../types/director'
import { directorLeagueLabel } from '../../../utils/directorLinks'
import { resolvePesPositionBadge } from '../badges'
import type { ForeignQuotaCheck } from '../helpers'

const props = defineProps<{
  squadForeign: DirectorSquadPlayerDto[]
  foreignStayIds: number[]
  foreignLeaveIds: number[]
  foreignIns: PlanPlayerItem[]
  quotaCheck: ForeignQuotaCheck
}>()

const emit = defineEmits<{
  (e: 'set-action', playerId: number, action: 'stay' | 'leave'): void
  (e: 'remove-in', key: string): void
  (e: 'open-picker'): void
}>()

function isStay(playerId: number): boolean {
  return props.foreignStayIds.includes(playerId)
}

function isLeave(playerId: number): boolean {
  return props.foreignLeaveIds.includes(playerId)
}
</script>

<style scoped lang="css">
.foreign-card {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-16);
  padding: var(--fi-space-24);
  border-radius: var(--fi-radius-lg);
  background: var(--fi-color-surface);
  border: var(--fi-border-card);
  box-shadow: var(--fi-shadow-card);
}

.foreign-card__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-12);
}

.foreign-card__title-wrap {
  display: flex;
  align-items: center;
  gap: var(--fi-space-10);
}

.foreign-card__step {
  padding: var(--fi-space-4) var(--fi-space-10);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-text-primary);
  color: var(--fi-color-surface);
  font-size: var(--fi-font-18);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-none);
}

.foreign-card__title {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-30);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-snug);
}

.foreign-card__counter {
  color: var(--fi-color-primary);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-extrabold);
  font-variant-numeric: tabular-nums;
}

.foreign-card__counter--over {
  color: var(--fi-color-error);
}

.quota-track {
  width: 100%;
  height: var(--fi-space-12);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-divider-soft);
  overflow: hidden;
}

.quota-fill {
  height: 100%;
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-primary);
  transition: width var(--fi-motion-duration-base) var(--fi-motion-ease-out);
}

.quota-fill--over {
  background: var(--fi-color-error);
}

.quota-hint {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-normal);
}

.quota-warn {
  color: var(--fi-color-primary-deep);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-normal);
}

.squad-list {
  display: flex;
  flex-direction: column;
  border-top: 1rpx solid var(--fi-color-divider-soft);
}

.player-row {
  display: flex;
  align-items: center;
  gap: var(--fi-space-14);
  padding: var(--fi-space-16) 0;
  border-bottom: 1rpx solid var(--fi-color-divider-soft);
}

.player-row:last-child {
  border-bottom: none;
  padding-bottom: var(--fi-space-4);
}

.player-row__avatar {
  flex-shrink: 0;
  width: var(--fi-space-64);
  height: var(--fi-space-64);
  border-radius: var(--fi-radius-round);
  overflow: hidden;
  background: var(--fi-color-page-soft);
  font-size: var(--fi-font-24);
}

.player-row__info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-6);
}

.player-row__top {
  display: flex;
  align-items: center;
  gap: var(--fi-space-8);
}

.pes-pos {
  flex-shrink: 0;
  min-width: 66rpx;
  padding: var(--fi-space-4) var(--fi-space-8);
  border-radius: var(--fi-radius-xs);
  background: #1c232d;
  font-size: var(--fi-font-20);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-none);
  text-align: center;
  letter-spacing: 1rpx;
}

.pes-pos--gk { color: #f2c94c; }
.pes-pos--df { color: #38b6ff; }
.pes-pos--mf { color: #4cd964; }
.pes-pos--fw { color: #ff477e; }

.player-row__name {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-tight);
}

.player-row__meta {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-none);
}

.player-row__toggle {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: var(--fi-space-8);
}

.toggle-pill {
  margin: 0;
  padding: var(--fi-space-10) var(--fi-space-18);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-surface);
  color: var(--fi-color-text-secondary);
  border: var(--fi-border-width) solid var(--fi-color-divider);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

.toggle-pill::after {
  border: 0;
}

.toggle-pill--stay {
  background: var(--fi-color-seat-desired);
  color: var(--fi-color-surface);
  border-color: var(--fi-color-seat-desired);
}

.toggle-pill--leave {
  background: var(--fi-color-primary);
  color: var(--fi-color-surface);
  border-color: var(--fi-color-primary);
}

.toggle-pill--pressed {
  opacity: var(--fi-opacity-pressed);
}

.empty-hint {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-normal);
}

.signings-block {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-12);
  padding-top: var(--fi-space-16);
  border-top: var(--fi-border-divider);
}

.signings-block__head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--fi-space-8);
}

.signings-block__title {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-extrabold);
}

.signings-block__sub {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-20);
}

.signing-list {
  display: flex;
  flex-direction: column;
  border-top: 1rpx solid var(--fi-color-divider-soft);
}

.signing-chip {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-12);
  padding: var(--fi-space-14) 0;
  border-bottom: 1rpx solid var(--fi-color-divider-soft);
}

.signing-chip:last-child {
  border-bottom: none;
}

.signing-chip__main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-6);
}

.signing-chip__top {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: var(--fi-space-8);
}

.signing-chip__name {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
}

.signing-chip__tag {
  padding: var(--fi-space-4) var(--fi-space-10);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-primary-soft);
  color: var(--fi-color-primary);
  font-size: var(--fi-font-20);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

.signing-chip__club {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}

.signing-chip__remove {
  flex-shrink: 0;
  margin: 0;
  padding: var(--fi-space-8) var(--fi-space-16);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-surface);
  color: var(--fi-color-text-secondary);
  border: var(--fi-border-width) solid var(--fi-color-divider);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-none);
}

.signing-chip__remove::after {
  border: 0;
}

.add-foreign-btn {
  margin: 0;
  padding: var(--fi-space-18) var(--fi-space-20);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-text-primary);
  color: var(--fi-color-surface);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

.add-foreign-btn::after {
  border: 0;
}

.add-foreign-btn--disabled {
  background: var(--fi-color-page-soft);
  color: var(--fi-color-text-muted);
  border: var(--fi-border-card);
}
</style>
