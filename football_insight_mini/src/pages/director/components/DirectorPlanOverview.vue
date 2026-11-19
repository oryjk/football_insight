<template>
  <view class="overview-card">
    <view class="author-row">
      <view class="author-avatar">
        <FiAvatar :src="plan.author.avatar_url" :size="96" :name="plan.author.nickname" />
      </view>
      <view class="author-meta">
        <text class="author-name">{{ plan.author.nickname }}</text>
        <text class="author-sub">{{ plan.window_name }} · {{ plan.team_name }}</text>
      </view>
      <text class="style-badge">{{ plan.style_tag }}</text>
    </view>

    <view v-if="plan.manifesto" class="manifesto-box">
      <text class="manifesto-quote">「{{ plan.manifesto }}」</text>
    </view>

    <view class="roster-block">
      <text class="roster-title">
        最终外援阵容（{{ finalForeignList.length }} 人{{ totalForeignValueLabel ? ` · 总身价 ${totalForeignValueLabel}` : '' }}）
      </text>
      <view class="roster-grid">
        <view v-for="item in finalForeignList" :key="item.key" class="roster-chip">
          <view class="roster-chip__top">
            <view class="roster-chip__title">
              <view
                class="pes-pos"
                :class="`pes-pos--${resolvePesPositionBadge(item.position).zone}`"
              >
                {{ resolvePesPositionBadge(item.position).code }}
              </view>
              <text class="roster-chip__name">{{ item.player_name }}</text>
            </view>
            <text
              class="roster-chip__badge"
              :class="item.isNew ? 'roster-chip__badge--new' : 'roster-chip__badge--stay'"
            >
              {{ item.isNew ? '新签' : '留队' }}
            </text>
          </view>
          <text class="roster-chip__meta">
            {{ formatOverviewMeta(item) }}
          </text>
        </view>
      </view>
    </view>

    <view v-if="plan.foreign_leaves.length" class="roster-block">
      <text class="roster-title">送走的外援（{{ plan.foreign_leaves.length }}）</text>
      <view class="tag-row">
        <text v-for="item in plan.foreign_leaves" :key="item.key" class="leave-tag">
          {{ item.player_name }}（{{ directorPositionLabel(item.position) }}）
        </text>
      </view>
    </view>

    <view v-if="plan.domestic_ins.length || plan.domestic_outs.length" class="roster-block">
      <text class="roster-title">国内阵容微调</text>
      <view v-if="plan.domestic_ins.length" class="tag-row">
        <text class="tag-prefix">补强：</text>
        <text v-for="item in plan.domestic_ins" :key="item.key" class="in-tag">
          {{ item.player_name }}
        </text>
      </view>
      <view v-if="plan.domestic_outs.length" class="tag-row">
        <text class="tag-prefix">离队：</text>
        <text v-for="item in plan.domestic_outs" :key="item.key" class="leave-tag">
          {{ item.player_name }}
        </text>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import FiAvatar from '../../../components/FiAvatar.vue'
import type { DirectorPlanDto, PlanPlayerItem } from '../../../types/director'
import { directorLeagueLabel, directorPositionLabel } from '../../../utils/directorLinks'
import { resolvePesPositionBadge } from '../badges'
import {
  formatContractUntil,
  formatMarketValueEur,
  formatPlayerAge,
  sumMarketValueEur,
} from '../helpers'

const props = defineProps<{
  plan: DirectorPlanDto
}>()

const finalForeignList = computed(() => [
  ...props.plan.foreign_stays.map(item => ({ ...item, isNew: false })),
  ...props.plan.foreign_ins.map(item => ({ ...item, isNew: true })),
])

const totalForeignValueLabel = computed(() =>
  formatMarketValueEur(sumMarketValueEur(finalForeignList.value)),
)

function formatOverviewMeta(item: PlanPlayerItem): string {
  return [
    directorLeagueLabel(item.league_tag),
    item.current_club,
    formatPlayerAge(item.birth_date),
    formatMarketValueEur(item.market_value_eur),
    formatContractUntil(item.contract_until),
  ].filter(Boolean).join(' · ')
}
</script>

<style scoped lang="css">
.overview-card {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-18);
  padding: var(--fi-space-24);
  border-radius: var(--fi-radius-sm);
  background: var(--fi-color-surface);
  border: var(--fi-border-card);
  box-shadow: var(--fi-shadow-card);
}

.author-row {
  display: flex;
  align-items: center;
  gap: var(--fi-space-14);
}

.author-avatar {
  flex-shrink: 0;
  width: var(--fi-space-72);
  height: var(--fi-space-72);
  border-radius: var(--fi-radius-round);
  overflow: hidden;
  background: var(--fi-color-page-soft);
  font-size: var(--fi-font-26);
}

.author-meta {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-6);
}

.author-name {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-30);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-tight);
}

.author-sub {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-none);
}

.style-badge {
  flex-shrink: 0;
  padding: var(--fi-space-8) var(--fi-space-14);
  border-radius: var(--fi-radius-xs);
  background: var(--fi-color-primary);
  color: var(--fi-color-surface);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-none);
}

.manifesto-box {
  padding: var(--fi-space-16) var(--fi-space-20);
  border-radius: var(--fi-radius-xs);
  background: var(--fi-color-primary-soft);
}

.manifesto-quote {
  color: var(--fi-color-primary-deep);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-normal);
}

.roster-block {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-12);
}

.roster-title {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-extrabold);
}

.roster-grid {
  display: flex;
  flex-direction: column;
  border-top: 1rpx solid var(--fi-color-divider-soft);
}

.roster-chip {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-6);
  padding: var(--fi-space-14) 0;
  border-bottom: 1rpx solid var(--fi-color-divider-soft);
}

.roster-chip:last-child {
  border-bottom: none;
  padding-bottom: var(--fi-space-4);
}

.roster-chip__top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-10);
}

.roster-chip__title {
  display: flex;
  align-items: center;
  gap: var(--fi-space-8);
  min-width: 0;
}

.pes-pos {
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  min-width: 66rpx;
  height: var(--fi-space-32);
  padding: 0 var(--fi-space-8);
  border-radius: var(--fi-radius-2xs);
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

.roster-chip__name {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
}

.roster-chip__badge {
  display: inline-flex;
  align-items: center;
  padding: var(--fi-space-4) var(--fi-space-10);
  border-radius: var(--fi-radius-2xs);
  font-size: var(--fi-font-20);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

.roster-chip__badge--new {
  background: var(--fi-color-primary-soft);
  color: var(--fi-color-primary);
}

.roster-chip__badge--stay {
  background: var(--fi-color-seat-desired-bg);
  color: var(--fi-color-seat-desired);
}

.roster-chip__meta {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}

.tag-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--fi-space-10);
}

.tag-prefix {
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
}

.leave-tag {
  padding: var(--fi-space-6) var(--fi-space-12);
  border-radius: var(--fi-radius-2xs);
  background: var(--fi-color-page-soft);
  color: var(--fi-color-text-secondary);
  border: var(--fi-border-width) solid var(--fi-color-divider);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-none);
}

.in-tag {
  padding: var(--fi-space-6) var(--fi-space-12);
  border-radius: var(--fi-radius-2xs);
  background: var(--fi-color-seat-desired-bg);
  color: var(--fi-color-seat-desired);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}
</style>
