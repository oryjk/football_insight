<template>
  <div class="hero">
    <div class="hero__main">
      <span class="hero__kicker">当前比赛 · 换座撮合</span>
      <span class="hero__title">{{ matchTitle }}</span>
      <span v-if="matchSummary" class="hero__summary">{{ matchSummary }}</span>
    </div>
    <div class="hero__side">
      <span class="hero__count">{{ candidatesCount }} 条意向</span>
      <button v-if="!isLoggedIn" type="button" class="hero__cta" @click="emit('login')">
        登录后发布
      </button>
      <button v-else-if="!hasMyRequest" type="button" class="hero__cta" @click="emit('publish')">
        + 发布我的换座
      </button>
      <div v-else class="hero__mine">
        <div class="hero__mine-status">
          <span class="hero__mine-dot"></span>
          <span class="hero__mine-label">{{ myStatusLabel }}</span>
        </div>
        <div class="hero__mine-seats">
          <span class="hero__seat hero__seat--current">{{ mySeatLabel }}</span>
          <span class="hero__arrow">→</span>
          <span class="hero__seat hero__seat--desired">{{ myDesiredSummary }}</span>
        </div>
        <button type="button" class="hero__manage" @click="emit('manage')">管理我的发布</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  matchTitle: string
  matchSummary: string
  candidatesCount: number
  isLoggedIn: boolean
  hasMyRequest: boolean
  mySeatLabel: string
  myDesiredSummary: string
  myStatusLabel: string
}>()

const emit = defineEmits<{
  (e: 'login'): void
  (e: 'publish'): void
  (e: 'manage'): void
}>()
</script>

<style scoped>
.hero {
  display: flex;
  align-items: center;
  gap: 24px;
  padding: 22px 26px;
  border-radius: 12px;
  border: 1px solid rgba(238, 233, 224, 0.95);
  background: rgba(255, 255, 255, 0.94);
  box-shadow: 0 6px 13px rgba(46, 38, 27, 0.06);
}

.hero__main {
  flex: 1;
  min-width: 0;
  display: grid;
  gap: 3px;
}

.hero__kicker {
  color: #8f7c5f;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 1.5px;
}

.hero__title {
  color: #17181c;
  font-size: 22px;
  font-weight: 800;
  line-height: 1.2;
}

.hero__summary {
  color: #988f84;
  font-size: 13px;
}

.hero__side {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 8px;
}

.hero__count {
  padding: 4px 10px;
  border-radius: 999px;
  border: 1px solid rgba(230, 220, 198, 0.92);
  background: linear-gradient(180deg, rgba(255, 251, 242, 0.98), rgba(248, 241, 227, 0.94));
  color: #9c855c;
  font-size: 11px;
  line-height: 1.6;
  white-space: nowrap;
}

.hero__cta {
  padding: 9px 18px;
  border: 0;
  border-radius: 999px;
  background: linear-gradient(180deg, #20242c, #191d26);
  color: #fff;
  font-size: 14px;
  font-weight: 700;
  box-shadow: 0 5px 12px rgba(21, 22, 27, 0.24);
  white-space: nowrap;
  cursor: pointer;
}

.hero__cta:active {
  transform: scale(0.98);
}

.hero__mine {
  display: grid;
  justify-items: end;
  gap: 5px;
}

.hero__mine-status {
  display: flex;
  align-items: center;
  gap: 5px;
}

.hero__mine-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: linear-gradient(180deg, #f6b44e, #d89b34);
  box-shadow: 0 0 0 3px rgba(216, 155, 52, 0.14);
}

.hero__mine-label {
  color: #17181c;
  font-size: 13px;
  font-weight: 700;
}

.hero__mine-seats {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
  justify-content: flex-end;
}

.hero__arrow {
  color: #988f84;
  font-size: 12px;
}

.hero__seat {
  padding: 3px 8px;
  border-radius: 7px;
  font-size: 12px;
  font-weight: 700;
  white-space: nowrap;
}

.hero__seat--current {
  background: linear-gradient(180deg, #f7efe1, #f1e3ca);
  border: 1px solid rgba(220, 201, 165, 0.6);
  color: #927445;
}

.hero__seat--desired {
  background: linear-gradient(180deg, #eaf8ef, #dff1e6);
  border: 1px solid rgba(29, 138, 85, 0.3);
  color: #167348;
}

.hero__manage {
  padding: 6px 14px;
  border-radius: 999px;
  border: 1px solid rgba(32, 36, 44, 0.16);
  background: #fff;
  color: #17181c;
  font-size: 12px;
  font-weight: 700;
  white-space: nowrap;
  cursor: pointer;
}

.hero__manage:active {
  transform: scale(0.98);
}

@media (max-width: 768px) {
  .hero {
    flex-direction: column;
    align-items: stretch;
    gap: 12px;
    padding: 14px 12px;
  }

  .hero__side {
    align-items: stretch;
  }

  .hero__cta {
    text-align: center;
  }

  .hero__mine {
    justify-items: start;
  }

  .hero__mine-seats {
    justify-content: flex-start;
  }
}
</style>
