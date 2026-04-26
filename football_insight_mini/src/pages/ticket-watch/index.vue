<template>
  <view v-if="pageEntered" class="page">
      <view class="hero-card">
        <view class="hero-card__icon">
          <view class="hero-card__icon-box">
            <image class="hero-card__icon-image" :src="boardIcon" mode="aspectFit" />
          </view>
        </view>
        <view class="hero-card__body">
          <text class="eyebrow">回流看板</text>
          <text class="hero-card__title">实时监控比赛余票动态</text>
          <text class="hero-card__summary">
            先看热区，再看价位和复盘结论
          </text>
        </view>
        <view class="hero-card__action">
          <text class="hero-card__badge">
            <image class="inline-image-icon" :src="boardIcon" mode="aspectFit" />
            <text>回流看板</text>
          </text>
        </view>
      </view>

      <view class="tab-switch">
        <button
          class="tab-switch__item"
          :class="{ 'tab-switch__item--active': activeTab === 'current' }"
          @click="activeTab = 'current'"
        >
          当前比赛
        </button>
        <button
          class="tab-switch__item"
          :class="{ 'tab-switch__item--active': activeTab === 'history' }"
          @click="activeTab = 'history'"
        >
          历史比赛
        </button>
      </view>

      <view v-if="systemConfigUnderReview" class="state-card">
        <text>当前版本展示基础内容。</text>
      </view>

      <view v-else-if="membershipBenefitsLocked" class="panel membership-lock-panel">
        <view class="section-heading section-heading--compact">
          <view>
            <text class="section-kicker">会员权益已暂停</text>
            <text class="section-title">当前账号已取关公众号</text>
          </view>
          <text class="meta-pill">重新关注后恢复</text>
        </view>

        <text class="membership-lock-panel__copy">
          回流看板属于会员权益。你当前的会员等级不会变化，也不会影响你已推荐的人；重新关注公众号后，返回小程序刷新即可恢复。
        </text>

        <button class="membership-lock-panel__action" @tap="goToUserPage">去我的页查看</button>
      </view>

      <template v-else-if="activeTab === 'current'">
        <view v-if="currentLoading" class="skeleton-stack">
          <view class="panel skeleton-panel">
            <view class="section-heading section-heading--compact">
              <view class="skeleton-copy-group">
                <view class="skeleton-line skeleton-line--kicker" />
                <view class="skeleton-line skeleton-line--title" />
              </view>
              <view class="skeleton-pill" />
            </view>

            <view class="skeleton-line skeleton-line--meta skeleton-line--meta-wide" />

            <view class="watch-metrics skeleton-metrics">
              <view class="watch-metric skeleton-metric">
                <view class="skeleton-line skeleton-line--label" />
                <view class="skeleton-line skeleton-line--metric" />
              </view>
              <view class="watch-metric skeleton-metric">
                <view class="skeleton-line skeleton-line--label" />
                <view class="skeleton-line skeleton-line--metric" />
              </view>
            </view>

            <view class="watch-monitor-actions">
              <view class="skeleton-button" />
            </view>

            <view class="skeleton-line skeleton-line--body" />
          </view>

          <view class="panel skeleton-panel">
            <view class="section-heading section-heading--compact">
              <view class="skeleton-copy-group">
                <view class="skeleton-line skeleton-line--kicker" />
                <view class="skeleton-line skeleton-line--section" />
              </view>
              <view class="skeleton-pill skeleton-pill--short" />
            </view>

            <view class="focus-grid focus-grid--triple">
              <view v-for="index in 3" :key="`current-skeleton-focus-${index}`" class="skeleton-focus-chip">
                <view class="skeleton-line skeleton-line--chip-name" />
                <view class="skeleton-line skeleton-line--chip-meta" />
                <view class="skeleton-line skeleton-line--chip-count" />
              </view>
            </view>
          </view>

          <view class="panel skeleton-panel">
            <view class="section-heading section-heading--compact">
              <view class="skeleton-copy-group">
                <view class="skeleton-line skeleton-line--kicker" />
                <view class="skeleton-line skeleton-line--section" />
              </view>
              <view class="skeleton-pill skeleton-pill--short" />
            </view>

            <view class="inventory-grid">
              <view v-for="index in 8" :key="`current-skeleton-cell-${index}`" class="skeleton-inventory-cell">
                <view class="skeleton-line skeleton-line--cell-name" />
                <view class="skeleton-line skeleton-line--cell-count" />
                <view class="skeleton-line skeleton-line--cell-time" />
              </view>
            </view>
          </view>
        </view>

        <view v-else-if="currentErrorMessage" class="state-card state-card--error">
          <text>{{ currentErrorMessage }}</text>
        </view>

        <template v-else>
          <view v-if="currentMatch" class="panel watch-match-card watch-match-card--reference">
            <view class="watch-match-card__topline">
              <view class="watch-match-card__identity">
                <view class="watch-match-card__ball">
                  <image class="watch-match-card__ball-image" :src="soccerBallIcon" mode="aspectFit" />
                </view>
                <view class="watch-match-card__title-wrap">
                  <text class="section-title section-title--match">{{ currentMatch.home_team_name }} VS {{ currentMatch.away_team_name }}</text>
                </view>
              </view>

              <view class="watch-match-card__membership-pill">
                <image class="watch-match-card__membership-icon" :src="crownIcon" mode="aspectFit" />
                <text>{{ membershipExclusiveLabel }}</text>
              </view>
            </view>

            <view class="watch-match-card__meta watch-match-card__meta--reference">
              <text class="watch-match-card__round">第 {{ currentMatch.round_number }} 轮</text>
              <text class="watch-match-card__date">{{ currentMatch.match_date }} {{ currentMatch.match_time }}</text>
            </view>

            <view class="watch-metrics">
              <view class="watch-metric">
                <view class="watch-metric__head">
                  <text class="watch-metric__label">命中区域</text>
                  <image class="watch-metric__icon-image" :src="targetIcon" mode="aspectFit" />
                </view>
                <text class="watch-metric__value">{{ currentAvailableRegionCount }}</text>
              </view>
              <view class="watch-metric">
                <view class="watch-metric__head">
                  <text class="watch-metric__label">累计回流</text>
                  <image class="watch-metric__icon-image" :src="trendIcon" mode="aspectFit" />
                </view>
                <text class="watch-metric__value">{{ currentTotalOccurrences }}</text>
              </view>
            </view>

            <view class="watch-monitor-actions">
              <button
                class="watch-monitor-actions__button"
                :class="{ 'watch-monitor-actions__button--ghost': isMonitoringActive }"
                @tap="toggleMonitoring"
              >
                <image class="watch-monitor-actions__button-icon-image" :src="playCircleIcon" mode="aspectFit" />
                {{ isMonitoringActive ? '停止监控' : '开始监控' }}
              </button>
            </view>

            <text class="watch-match-card__note watch-match-card__note--reference">
              <image class="watch-match-card__note-icon-image" :src="shieldIcon" mode="aspectFit" />
              <text>{{ currentMessage }}</text>
            </text>
          </view>

          <view v-else class="state-card state-card--empty">
            <text>{{ currentMessage }}</text>
          </view>

          <view v-if="currentMatch && currentRecentRefluxPanelMode !== 'hidden'" class="panel recent-reflux-panel">
            <view class="section-heading section-heading--compact">
              <view class="recent-reflux-panel__heading">
                <view class="recent-reflux-panel__icon">
                  <image class="recent-reflux-panel__icon-image" :src="radarIcon" mode="aspectFit" />
                </view>
                <text class="section-title">最近回流速览</text>
              </view>
              <text class="meta-pill">{{ currentRecentRefluxPanelMode === 'unlocked' ? '分档可见' : 'V6 及以上' }}</text>
            </view>

            <view v-if="currentRecentRefluxPanelMode === 'unlocked'" class="recent-reflux-buckets">
              <view
                v-for="bucket in currentRecentRefluxBuckets"
                :key="bucket.key"
                class="recent-reflux-bucket"
                :class="[
                  `recent-reflux-bucket--${bucket.key}`,
                  { 'recent-reflux-bucket--locked': !isCurrentRecentRefluxBucketUnlocked(bucket.key) },
                ]"
              >
                <view class="recent-reflux-bucket__head">
                  <view>
                    <text class="recent-reflux-bucket__title">{{ bucket.title }}</text>
                    <text class="recent-reflux-bucket__subtitle">{{ bucket.subtitle }}</text>
                  </view>
                </view>

                <view v-if="isCurrentRecentRefluxBucketUnlocked(bucket.key) && bucket.items.length" class="recent-reflux-list">
                  <view
                    v-for="item in bucket.items.slice(0, 4)"
                    :key="`recent-reflux-${bucket.key}-${item.block_name}`"
                    class="recent-reflux-item"
                    :class="[resolvePriceToneClass(item.price), `recent-reflux-item--${bucket.key}`]"
                  >
                    <view class="recent-reflux-item__beam" />
                    <text class="recent-reflux-item__name">{{ item.block_name }}</text>
                    <text class="recent-reflux-item__price">¥{{ item.price }}</text>
                    <text class="recent-reflux-item__time">{{ formatRecentRefluxMinuteLabel(item.minutes_ago) }}</text>
                  </view>
                </view>

                <text v-else-if="isCurrentRecentRefluxBucketUnlocked(bucket.key)" class="recent-reflux-bucket__empty">暂无</text>
                <view v-else class="recent-reflux-bucket-lock" @tap="goToMembershipPurchase">
                  <view class="recent-reflux-bucket-lock__icon" aria-hidden="true">
                    <view class="recent-reflux-bucket-lock__shackle" />
                    <view class="recent-reflux-bucket-lock__body" />
                  </view>
                  <text class="recent-reflux-bucket-lock__tag">{{ formatRecentRefluxBucketRequiredTier(bucket.key) }} 解锁</text>
                  <text class="recent-reflux-bucket-lock__copy">{{ bucket.title }}明细</text>
                </view>
              </view>
            </view>

            <view v-else class="recent-reflux-lock">
              <view class="recent-reflux-lock__preview">
                <view class="recent-reflux-lock__bar recent-reflux-lock__bar--red" />
                <view class="recent-reflux-lock__bar recent-reflux-lock__bar--yellow" />
                <view class="recent-reflux-lock__bar recent-reflux-lock__bar--green" />
              </view>
              <view class="recent-reflux-lock__body">
                <text class="recent-reflux-lock__title">V6 及以上会员可以查看</text>
                <text class="recent-reflux-lock__copy">V6 可查看 30 分钟内，V7 可查看 10 分钟内，V8/V9 可查看 3 分钟内的具体座位区域。</text>
              </view>
              <button class="recent-reflux-lock__action" @tap="goToMembershipPurchase">升级查看</button>
            </view>
          </view>

          <view v-if="currentMatch && (currentFocusBlocks.length || currentInterestFocusBlocks.length)" class="panel focus-panel">
            <view class="section-heading section-heading--compact">
              <view class="focus-panel__heading">
                <view class="focus-panel__headline">
                  <image class="focus-panel__headline-icon" :src="fireIcon" mode="aspectFit" />
                  <text class="focus-panel__headline-title">热区速览</text>
                  <text class="focus-panel__headline-subtitle">先看双榜热区</text>
                </view>
              </view>
              <view class="section-heading__actions">
                <text class="meta-pill">
                  <image class="inline-image-icon inline-image-icon--small" :src="swapArrowsIcon" mode="aspectFit" />
                  <text>回流 + 钓友</text>
                </text>
                <button class="collapse-pill" @tap="toggleCurrentSectionCollapsed('focus')">
                  {{ isCurrentSectionCollapsed('focus') ? '展开' : '收起' }}
                </button>
              </view>
            </view>

            <template v-if="!isCurrentSectionCollapsed('focus')">
              <view v-if="currentFocusBlocks.length" class="focus-section">
                <view class="focus-section__head">
                  <view class="focus-section__title-row">
                    <image class="focus-section__title-icon-image" :src="refluxBarsIcon" mode="aspectFit" />
                    <text class="focus-section__title">回流热区</text>
                  </view>
                  <text class="focus-section__meta">先刷这 3 个区</text>
                </view>
                <view class="focus-grid focus-grid--triple">
                  <view
                    v-for="block in currentFocusBlocks"
                    :key="`current-focus-${block.block_name}`"
                    class="focus-chip focus-chip--compact"
                    :class="resolvePriceToneClass(block.price)"
                  >
                    <view class="focus-chip__name-row">
                      <text class="focus-chip__name">{{ block.block_name }}</text>
                      <image class="focus-chip__name-icon-image" :src="fireIcon" mode="aspectFit" />
                    </view>
                    <text class="focus-chip__meta">¥{{ block.price }}</text>
                    <text class="focus-chip__count">{{ block.occurrences }} 张</text>
                  </view>
                </view>
              </view>

              <view v-if="currentInterestFocusBlocks.length" class="focus-section">
                <view class="focus-section__head">
                  <view class="focus-section__title-row">
                    <image class="focus-section__title-icon-image" :src="friendUserIcon" mode="aspectFit" />
                    <text class="focus-section__title">钓友钓区</text>
                  </view>
                  <text class="focus-section__meta">大家更想蹲这 3 个区</text>
                </view>
                <view class="focus-grid focus-grid--triple">
                  <view
                    v-for="block in currentInterestFocusBlocks"
                    :key="`current-interest-focus-${block.block_name}`"
                    class="focus-chip focus-chip--compact focus-chip--interest"
                    :class="resolvePriceToneClass(block.price)"
                  >
                    <view class="focus-chip__name-row">
                      <text class="focus-chip__name">{{ block.block_name }}</text>
                      <image class="focus-chip__name-icon-image" :src="fireIcon" mode="aspectFit" />
                    </view>
                    <text class="focus-chip__meta">¥{{ block.price }}</text>
                    <text class="focus-chip__count focus-chip__count--interest">{{ block.interested_user_count }} 位钓友</text>
                  </view>
                </view>
              </view>

              <text class="focus-copy">先扫回流前三，再看钓友都蹲哪几个区，下面再看完整分区看板。</text>
            </template>
          </view>

          <view v-if="currentMatch && currentUser" class="panel tracked-interest-panel">
            <view class="section-heading section-heading--compact">
              <view>
                <text class="section-kicker">My Tracking</text>
                <text class="section-title">我的钓区追踪</text>
              </view>
              <view class="section-heading__actions">
                <text class="meta-pill">
                  {{ currentTrackedInterestSummary.hitCount }}/{{ currentTrackedInterestSummary.total }} 已等到
                </text>
                <button class="collapse-pill" @tap="toggleCurrentSectionCollapsed('tracked-interests')">
                  {{ isCurrentSectionCollapsed('tracked-interests') ? '展开' : '收起' }}
                </button>
              </view>
            </view>

            <template v-if="!isCurrentSectionCollapsed('tracked-interests')">
              <text v-if="currentTrackedInterestSummary.total" class="tracked-interest-summary">
                你当前钓了 {{ currentTrackedInterestSummary.total }} 个区，其中 {{ currentTrackedInterestSummary.hitCount }} 个区已经等到回流，{{ currentTrackedInterestSummary.pendingCount }} 个区还在继续等。
              </text>
              <text v-else class="tracked-interest-summary tracked-interest-summary--empty">
                你还没标记自己的钓区，点下面分区卡片就会开始记录等待时间。
              </text>

              <view v-if="currentTrackedInterests.length" class="tracked-interest-list">
                <view
                  v-for="interest in currentTrackedInterests"
                  :key="`tracked-interest-${interest.block_name}`"
                  class="tracked-interest-item"
                >
                  <view class="tracked-interest-item__head">
                    <text class="tracked-interest-item__name">{{ interest.block_name }}</text>
                    <text
                      class="tracked-interest-item__status"
                      :class="{ 'tracked-interest-item__status--hit': interest.first_inventory_at }"
                    >
                      {{ formatTrackedInterestWaitLabel(interest.started_at, interest.first_inventory_at) }}
                    </text>
                  </view>
                  <view class="tracked-interest-item__meta">
                    <text>开始钓：{{ formatTrackedInterestTime(interest.started_at) }}</text>
                    <text>
                      首次回流：{{ interest.first_inventory_at ? formatTrackedInterestTime(interest.first_inventory_at) : '暂未等到' }}
                    </text>
                  </view>
                </view>
              </view>
            </template>
          </view>

          <view v-if="currentMatch && prioritizedCurrentSections.length" class="inventory-stack">
            <view
              v-for="section in prioritizedCurrentSections"
              :key="`current-${section.price}`"
              class="panel inventory-panel"
              :class="resolvePriceToneClass(section.price)"
            >
              <view class="inventory-panel__header">
                <view>
                  <text class="inventory-panel__price">¥{{ section.price }}</text>
                  <text class="inventory-panel__meta">{{ section.region_count }} 个区域</text>
                </view>
                <view class="inventory-panel__actions">
                  <text class="inventory-panel__summary">
                    {{ section.available_region_count }} 区有回流 · 共 {{ section.total_occurrences }} 张
                  </text>
                  <button class="collapse-pill collapse-pill--inventory" @tap="toggleCurrentSectionCollapsed(`inventory:${section.price}`)">
                    {{ isCurrentSectionCollapsed(`inventory:${section.price}`) ? '展开' : '收起' }}
                  </button>
                </view>
              </view>

              <view v-if="!isCurrentSectionCollapsed(`inventory:${section.price}`)" class="inventory-grid">
                <button
                  v-for="item in section.items"
                  :key="`current-${section.price}-${item.block_name}`"
                  class="inventory-cell"
                  :disabled="isInterestToggleLoading(currentMatch.match_id, item.block_name)"
                  hover-class="inventory-cell--pressed"
                  :class="resolveInventoryCellClass(item, section, currentMatch.match_id)"
                  @tap="handleBlockInterestToggle(currentMatch, item, 'current')"
                >
                  <text class="inventory-cell__name">{{ item.block_name }}</text>
                  <text class="inventory-cell__count">{{ item.has_inventory ? `${item.occurrences}张` : '--' }}</text>
                  <text class="inventory-cell__time">{{ item.has_inventory ? formatLatestTime(item.latest_time) : '无回流' }}</text>
                  <view v-if="item.interested_user_count > 0" class="inventory-cell__interest-strip">
                    <view class="inventory-cell__interest-meter">
                      <text
                        v-for="barIndex in 4"
                        :key="`current-interest-bar-${item.block_name}-${barIndex}`"
                        class="inventory-cell__interest-bar"
                        :class="{ 'inventory-cell__interest-bar--active': barIndex <= resolveBlockInterestHeatLevel(item.interested_user_count) }"
                      />
                    </view>
                    <text class="inventory-cell__interest-label">{{ item.interested_user_count }}位钓友</text>
                  </view>
                </button>
              </view>
            </view>
          </view>

          <view v-if="currentMatch" class="panel">
            <view class="section-heading section-heading--compact">
              <view>
                <text class="section-kicker">Supply Signal</text>
                <text class="section-title">比赛供给洞察</text>
              </view>
              <view class="section-heading__actions">
                <text class="meta-pill">{{ currentBoardStats.priceBandCount }} 个价位</text>
                <button class="collapse-pill" @tap="toggleCurrentSectionCollapsed('insight')">
                  {{ isCurrentSectionCollapsed('insight') ? '展开' : '收起' }}
                </button>
              </view>
            </view>

            <template v-if="!isCurrentSectionCollapsed('insight')">
              <view class="insight-grid">
                <view class="insight-metric">
                  <text class="insight-metric__label">覆盖率</text>
                  <text class="insight-metric__value">{{ formatCoverage(currentBoardStats) }}</text>
                  <text class="insight-metric__note">{{ formatPercent(currentBoardStats.activeRegionRatio) }}</text>
                </view>
                <view class="insight-metric">
                  <text class="insight-metric__label">热点价位</text>
                  <text class="insight-metric__value">{{ formatHotPrice(currentBoardStats) }}</text>
                  <text class="insight-metric__note">{{ formatHotPriceMeta(currentBoardStats) }}</text>
                </view>
                <view class="insight-metric">
                  <text class="insight-metric__label">累计回流</text>
                  <text class="insight-metric__value">{{ currentBoardStats.totalOccurrences }}</text>
                  <text class="insight-metric__note">张</text>
                </view>
              </view>

              <text class="insight-copy">{{ buildBoardInsightSummary(currentBoardStats, 'current') }}</text>
            </template>
          </view>

          <view v-if="currentMatch" class="panel">
            <view class="section-heading section-heading--compact">
              <view>
                <text class="section-kicker">Decision Guide</text>
                <text class="section-title">帮用户抢票决策</text>
              </view>
              <view class="section-heading__actions">
                <text class="meta-pill">实时</text>
                <button class="collapse-pill" @tap="toggleCurrentSectionCollapsed('decision')">
                  {{ isCurrentSectionCollapsed('decision') ? '展开' : '收起' }}
                </button>
              </view>
            </view>

            <view v-if="!isCurrentSectionCollapsed('decision')" class="decision-list">
              <view v-for="line in currentDecisionLines" :key="line" class="decision-item">
                <text class="decision-item__dot" />
                <text class="decision-item__text">{{ line }}</text>
              </view>
            </view>
          </view>
        </template>
      </template>

      <template v-else>
        <view v-if="historyLoading" class="skeleton-stack">
          <view class="panel skeleton-panel">
            <view class="section-heading section-heading--compact">
              <view class="skeleton-copy-group">
                <view class="skeleton-line skeleton-line--kicker" />
                <view class="skeleton-line skeleton-line--section" />
              </view>
              <view class="skeleton-pill skeleton-pill--short" />
            </view>

            <view class="history-chip-list">
              <view v-for="index in 4" :key="`history-skeleton-chip-${index}`" class="skeleton-history-chip">
                <view class="skeleton-line skeleton-line--history-date" />
                <view class="skeleton-line skeleton-line--history-match" />
              </view>
            </view>
          </view>

          <view class="panel skeleton-panel">
            <view class="section-heading section-heading--compact">
              <view class="skeleton-copy-group">
                <view class="skeleton-line skeleton-line--kicker" />
                <view class="skeleton-line skeleton-line--section" />
              </view>
              <view class="skeleton-pill skeleton-pill--short" />
            </view>

            <view class="focus-grid focus-grid--triple">
              <view v-for="index in 3" :key="`history-skeleton-focus-${index}`" class="skeleton-focus-chip">
                <view class="skeleton-line skeleton-line--chip-name" />
                <view class="skeleton-line skeleton-line--chip-meta" />
                <view class="skeleton-line skeleton-line--chip-count" />
              </view>
            </view>
          </view>

          <view class="panel skeleton-panel">
            <view class="section-heading section-heading--compact">
              <view class="skeleton-copy-group">
                <view class="skeleton-line skeleton-line--kicker" />
                <view class="skeleton-line skeleton-line--section" />
              </view>
              <view class="skeleton-pill skeleton-pill--short" />
            </view>

            <view class="inventory-grid">
              <view v-for="index in 8" :key="`history-skeleton-cell-${index}`" class="skeleton-inventory-cell">
                <view class="skeleton-line skeleton-line--cell-name" />
                <view class="skeleton-line skeleton-line--cell-count" />
                <view class="skeleton-line skeleton-line--cell-time" />
              </view>
            </view>
          </view>
        </view>

        <view v-else-if="historyErrorMessage" class="state-card state-card--error">
          <text>{{ historyErrorMessage }}</text>
        </view>

        <template v-else>
          <view class="panel">
            <view class="section-heading section-heading--compact">
              <view>
                <text class="section-kicker">History Matches</text>
                <text class="section-title">选择一场历史比赛</text>
              </view>
              <text class="meta-pill">{{ historyMatches.length }} 场</text>
            </view>

            <scroll-view
              scroll-x
              class="history-chip-scroll"
              :scroll-left="historyChipScrollLeft"
              scroll-with-animation
              show-scrollbar="false"
            >
              <view class="history-chip-list">
                <button
                  v-for="match in historyMatches"
                  :id="`history-chip-${match.match_id}`"
                  :key="match.match_id"
                  class="history-chip"
                  :class="{ 'history-chip--active': selectedHistoryMatch?.match_id === match.match_id }"
                  @click="handleHistoryMatchSelect(match.match_id)"
                >
                  <text class="history-chip__date">{{ match.match_date }}</text>
                  <text class="history-chip__teams">{{ match.home_team_name }} VS {{ match.away_team_name }}</text>
                </button>
              </view>
            </scroll-view>
          </view>

          <view v-if="displayedHistoryMatch" class="panel watch-match-card">
            <view class="section-heading section-heading--compact">
              <view>
                <text class="section-kicker">Selected Match</text>
                <text class="section-title">{{ displayedHistoryMatch.home_team_name }} VS {{ displayedHistoryMatch.away_team_name }}</text>
              </view>
              <text class="meta-pill">{{ historySelectionLoading ? '切换中' : `第 ${displayedHistoryMatch.round_number} 轮` }}</text>
            </view>

            <view class="watch-match-card__meta">
              <text>{{ displayedHistoryMatch.match_date }} {{ displayedHistoryMatch.match_time }}</text>
              <text>{{ historyTotalOccurrences }} 张累计回流</text>
            </view>
          </view>

          <view v-if="displayedHistoryMatch && (historyFocusBlocks.length || historyInterestFocusBlocks.length)" class="panel focus-panel">
            <view class="section-heading section-heading--compact">
              <view class="focus-panel__heading">
                <view class="focus-panel__headline">
                  <image class="focus-panel__headline-icon" :src="fireIcon" mode="aspectFit" />
                  <text class="focus-panel__headline-title">热区速览</text>
                  <text class="focus-panel__headline-subtitle">先看双榜热区</text>
                </view>
              </view>
              <view class="section-heading__actions">
                <text class="meta-pill">
                  <image class="inline-image-icon inline-image-icon--small" :src="swapArrowsIcon" mode="aspectFit" />
                  <text>{{ historySelectionLoading ? '切换中' : '回流 + 钓友' }}</text>
                </text>
                <button class="collapse-pill" @tap="toggleHistorySectionCollapsed('focus')">
                  {{ isHistorySectionCollapsed('focus') ? '展开' : '收起' }}
                </button>
              </view>
            </view>

            <template v-if="!isHistorySectionCollapsed('focus')">
              <view v-if="historyFocusBlocks.length" class="focus-section">
                <view class="focus-section__head">
                  <view class="focus-section__title-row">
                    <image class="focus-section__title-icon-image" :src="refluxBarsIcon" mode="aspectFit" />
                    <text class="focus-section__title">回流热区</text>
                  </view>
                  <text class="focus-section__meta">复盘先看这 3 个区</text>
                </view>
                <view class="focus-grid focus-grid--triple">
                  <view
                    v-for="block in historyFocusBlocks"
                    :key="`history-focus-${block.block_name}`"
                    class="focus-chip focus-chip--compact"
                    :class="resolvePriceToneClass(block.price)"
                  >
                    <view class="focus-chip__name-row">
                      <text class="focus-chip__name">{{ block.block_name }}</text>
                      <image class="focus-chip__name-icon-image" :src="fireIcon" mode="aspectFit" />
                    </view>
                    <text class="focus-chip__meta">¥{{ block.price }}</text>
                    <text class="focus-chip__count">{{ block.occurrences }} 张</text>
                  </view>
                </view>
              </view>

              <view v-if="historyInterestFocusBlocks.length" class="focus-section">
                <view class="focus-section__head">
                  <view class="focus-section__title-row">
                    <image class="focus-section__title-icon-image" :src="friendUserIcon" mode="aspectFit" />
                    <text class="focus-section__title">钓友钓区</text>
                  </view>
                  <text class="focus-section__meta">这场大家更想蹲的 3 个区</text>
                </view>
                <view class="focus-grid focus-grid--triple">
                  <view
                    v-for="block in historyInterestFocusBlocks"
                    :key="`history-interest-focus-${block.block_name}`"
                    class="focus-chip focus-chip--compact focus-chip--interest"
                    :class="resolvePriceToneClass(block.price)"
                  >
                    <view class="focus-chip__name-row">
                      <text class="focus-chip__name">{{ block.block_name }}</text>
                      <image class="focus-chip__name-icon-image" :src="fireIcon" mode="aspectFit" />
                    </view>
                    <text class="focus-chip__meta">¥{{ block.price }}</text>
                    <text class="focus-chip__count focus-chip__count--interest">{{ block.interested_user_count }} 位钓友</text>
                  </view>
                </view>
              </view>

              <text class="focus-copy">先扫回流前三，再看钓友都蹲哪几个区，下面再看完整价位分布。</text>
            </template>
          </view>

          <view v-if="displayedHistoryMatch && prioritizedHistorySections.length" class="inventory-stack">
            <view
              v-for="section in prioritizedHistorySections"
              :key="`history-${section.price}`"
              class="panel inventory-panel"
              :class="resolvePriceToneClass(section.price)"
            >
              <view class="inventory-panel__header">
                <view>
                  <text class="inventory-panel__price">¥{{ section.price }}</text>
                  <text class="inventory-panel__meta">{{ section.region_count }} 个区域</text>
                </view>
                <view class="inventory-panel__actions">
                  <text class="inventory-panel__summary">
                    {{ section.available_region_count }} 区有回流 · 共 {{ section.total_occurrences }} 张
                  </text>
                  <button class="collapse-pill collapse-pill--inventory" @tap="toggleHistorySectionCollapsed(`inventory:${section.price}`)">
                    {{ isHistorySectionCollapsed(`inventory:${section.price}`) ? '展开' : '收起' }}
                  </button>
                </view>
              </view>

              <view v-if="!isHistorySectionCollapsed(`inventory:${section.price}`)" class="inventory-grid">
                <view
                  v-for="item in section.items"
                  :key="`history-${section.price}-${item.block_name}`"
                  class="inventory-cell"
                  :class="resolveInventoryCellClass(item, section, displayedHistoryMatch.match_id, 'history')"
                >
                  <text class="inventory-cell__name">{{ item.block_name }}</text>
                  <text class="inventory-cell__count">{{ item.has_inventory ? `${item.occurrences}张` : '--' }}</text>
                  <text class="inventory-cell__time">{{ item.has_inventory ? formatLatestTime(item.latest_time) : '无回流' }}</text>
                  <view v-if="item.interested_user_count > 0" class="inventory-cell__interest-strip">
                    <view class="inventory-cell__interest-meter">
                      <text
                        v-for="barIndex in 4"
                        :key="`history-interest-bar-${item.block_name}-${barIndex}`"
                        class="inventory-cell__interest-bar"
                        :class="{ 'inventory-cell__interest-bar--active': barIndex <= resolveBlockInterestHeatLevel(item.interested_user_count) }"
                      />
                    </view>
                    <text class="inventory-cell__interest-label">{{ item.interested_user_count }}位钓友</text>
                  </view>
                </view>
              </view>
            </view>
          </view>

          <view v-if="displayedHistoryMatch" class="panel">
            <view class="section-heading section-heading--compact">
              <view>
                <text class="section-kicker">Replay Signal</text>
                <text class="section-title">历史供给洞察</text>
              </view>
              <view class="section-heading__actions">
                <text class="meta-pill">{{ historyBoardStats.priceBandCount }} 个价位</text>
                <button class="collapse-pill" @tap="toggleHistorySectionCollapsed('insight')">
                  {{ isHistorySectionCollapsed('insight') ? '展开' : '收起' }}
                </button>
              </view>
            </view>

            <template v-if="!isHistorySectionCollapsed('insight')">
              <view class="insight-grid">
                <view class="insight-metric">
                  <text class="insight-metric__label">覆盖率</text>
                  <text class="insight-metric__value">{{ formatCoverage(historyBoardStats) }}</text>
                  <text class="insight-metric__note">{{ formatPercent(historyBoardStats.activeRegionRatio) }}</text>
                </view>
                <view class="insight-metric">
                  <text class="insight-metric__label">热点价位</text>
                  <text class="insight-metric__value">{{ formatHotPrice(historyBoardStats) }}</text>
                  <text class="insight-metric__note">{{ formatHotPriceMeta(historyBoardStats) }}</text>
                </view>
                <view class="insight-metric">
                  <text class="insight-metric__label">累计回流</text>
                  <text class="insight-metric__value">{{ historyBoardStats.totalOccurrences }}</text>
                  <text class="insight-metric__note">张</text>
                </view>
              </view>

              <text class="insight-copy">{{ buildBoardInsightSummary(historyBoardStats, 'history') }}</text>
            </template>
          </view>

          <view v-if="displayedHistoryMatch" class="panel">
            <view class="section-heading section-heading--compact">
              <view>
                <text class="section-kicker">Replay Decision</text>
                <text class="section-title">历史抢票决策</text>
              </view>
              <view class="section-heading__actions">
                <text class="meta-pill">复盘</text>
                <button class="collapse-pill" @tap="toggleHistorySectionCollapsed('decision')">
                  {{ isHistorySectionCollapsed('decision') ? '展开' : '收起' }}
                </button>
              </view>
            </view>

            <view v-if="!isHistorySectionCollapsed('decision')" class="decision-list">
              <view v-for="line in historyDecisionLines" :key="line" class="decision-item">
                <text class="decision-item__dot" />
                <text class="decision-item__text">{{ line }}</text>
              </view>
            </view>
          </view>
        </template>
      </template>
  </view>
  <view v-if="pendingInterestSelection" class="interest-confirm-mask" @tap="closeInterestConfirm">
    <view class="interest-confirm-dialog" @tap.stop>
      <text class="interest-confirm-dialog__title">确认钓位</text>
      <text class="interest-confirm-dialog__content">
        确认把 {{ pendingInterestSelection.blockName }} 设为你的钓位吗？
      </text>
      <view class="interest-confirm-dialog__actions">
        <button class="interest-confirm-dialog__button interest-confirm-dialog__button--ghost" @tap="closeInterestConfirm">
          取消
        </button>
        <button class="interest-confirm-dialog__button interest-confirm-dialog__button--primary" @tap="confirmPendingInterestSelection">
          确认
        </button>
      </view>
    </view>
  </view>
  <view v-else-if="!pageEntered" class="page page--entry-mask" />
</template>

<script setup lang="ts">
import { computed, getCurrentInstance, nextTick, ref, watch } from 'vue'
import { onHide, onReady, onShareAppMessage, onShow, onUnload } from '@dcloudio/uni-app'
import { getCurrentUser } from '../../api/auth'
import {
  getCurrentTicketWatchBoard,
  getTicketWatchBlockInterests,
  getTicketWatchInventorySince,
  getTicketWatchMatches,
  getTicketWatchRegions,
  getTicketWatchTrackedInterests,
  toggleTicketWatchBlockInterest,
} from '../../api/ticketWatch'
import type { CurrentUser } from '../../types/auth'
import type {
  TicketWatchBlockInterest,
  TicketWatchGroupedInventoryItem,
  TicketWatchGroupedInventorySection,
  TicketWatchMatchSummary,
  TicketWatchRegion,
  TicketWatchTrackedInterest,
} from '../../types/ticketWatch'
import { extractApiErrorMessage } from '../../utils/apiError'
import { resolveMembershipBenefitsLocked } from '../../utils/membershipBenefits'
import { loadSystemConfigUnderReview } from '../../utils/systemConfig'
import { reportPageActivity } from '../../utils/userActivity'
import {
  normalizeTicketWatchPollIntervalSeconds,
} from '../../utils/membershipRules'
import boardIcon from '../../static/ticket-watch/board.svg'
import crownIcon from '../../static/ticket-watch/crown.svg'
import fireIcon from '../../static/ticket-watch/fire.svg'
import friendUserIcon from '../../static/ticket-watch/friend-user.svg'
import playCircleIcon from '../../static/ticket-watch/play-circle.svg'
import radarIcon from '../../static/ticket-watch/radar.svg'
import refluxBarsIcon from '../../static/ticket-watch/reflux-bars.svg'
import shieldIcon from '../../static/ticket-watch/shield.svg'
import soccerBallIcon from '../../static/ticket-watch/soccer-ball.svg'
import swapArrowsIcon from '../../static/ticket-watch/swap-arrows.svg'
import targetIcon from '../../static/ticket-watch/target.svg'
import trendIcon from '../../static/ticket-watch/trend.svg'
import {
  applyBlockInterestToSections,
  applyBlockInterestsToSections,
  buildInventorySince,
  buildRecentRefluxBuckets,
  buildTrackedInterestSummary,
  formatTicketWatchMembershipBadgeTier,
  formatTrackedInterestTime,
  formatTrackedInterestWaitLabel,
  groupInventoryByPrice,
  isTicketWatchSectionCollapsed,
  prioritizeInventorySections,
  resolveCurrentBoardLoadStrategy,
  resolveBlockInterestHeatLevel,
  resolveInventoryHeatLevel,
  resolveInventoryPriceTone,
  isRecentRefluxBucketUnlocked,
  resolveRecentRefluxPanelMode,
  resolveRecentRefluxBucketRequiredTier,
  resolveHistoryBoardLoadStrategy,
  selectCompletedMatches,
  summarizeInventoryBoard,
  type TicketWatchBoardStats,
  type TicketWatchCollapsedSectionState,
  type TicketWatchHistoryLoadReason,
  type TicketWatchLoadReason,
  type TicketWatchRecentRefluxBucketKey,
  toggleTicketWatchSectionCollapsed,
} from './helpers'

type TicketWatchTab = 'current' | 'history'
type PendingInterestSelection = {
  match: TicketWatchMatchSummary
  item: TicketWatchGroupedInventoryItem
  mode: TicketWatchTab
  blockName: string
}

const activeTab = ref<TicketWatchTab>('current')
const currentLoading = ref(true)
const historyLoading = ref(true)
const currentErrorMessage = ref('')
const historyErrorMessage = ref('')
const currentMessage = ref('暂无当前比赛。')
const regions = ref<TicketWatchRegion[]>([])
const currentMatch = ref<TicketWatchMatchSummary | null>(null)
const historyMatches = ref<TicketWatchMatchSummary[]>([])
const selectedHistoryMatchId = ref<number | null>(null)
const currentSections = ref<TicketWatchGroupedInventorySection[]>([])
const historySections = ref<TicketWatchGroupedInventorySection[]>([])
const hasLoadedCurrentBoard = ref(false)
const hasLoadedHistoryBoard = ref(false)
const currentBoardRequestInFlight = ref(false)
const historyBoardRequestInFlight = ref(false)
const historySelectionLoading = ref(false)
const historyChipScrollLeft = ref(0)
const displayedHistoryMatchId = ref<number | null>(null)
const historySectionsCache = ref<Record<number, TicketWatchGroupedInventorySection[]>>({})
const pageEntered = ref(true)
const interestToggleLoading = ref<Record<string, boolean>>({})
const currentUser = ref<CurrentUser | null>(null)
const viewerMembershipTier = ref('V1')
const systemConfigUnderReview = ref(false)
const pendingInterestSelection = ref<PendingInterestSelection | null>(null)
const isMonitoringActive = ref(false)
const currentTrackedInterests = ref<TicketWatchTrackedInterest[]>([])
const currentCollapsedSections = ref<TicketWatchCollapsedSectionState>({})
const historyCollapsedSections = ref<TicketWatchCollapsedSectionState>({})
const freshnessNowMs = ref(Date.now())
let historySelectionRequestToken = 0

let pollTimer: ReturnType<typeof setInterval> | null = null
let freshnessTimer: ReturnType<typeof setInterval> | null = null
const instance = getCurrentInstance()

const selectedHistoryMatch = computed(() => {
  if (selectedHistoryMatchId.value === null) {
    return null
  }

  return historyMatches.value.find((match) => match.match_id === selectedHistoryMatchId.value) ?? null
})

const displayedHistoryMatch = computed(() => {
  if (displayedHistoryMatchId.value === null) {
    return null
  }

  return historyMatches.value.find((match) => match.match_id === displayedHistoryMatchId.value) ?? null
})

const currentAvailableRegionCount = computed(() =>
  currentSections.value.reduce((sum, section) => sum + section.available_region_count, 0),
)

const currentTotalOccurrences = computed(() =>
  currentSections.value.reduce((sum, section) => sum + section.total_occurrences, 0),
)

const historyTotalOccurrences = computed(() =>
  historySections.value.reduce((sum, section) => sum + section.total_occurrences, 0),
)

const currentBoardStats = computed(() => summarizeInventoryBoard(currentSections.value))
const historyBoardStats = computed(() => summarizeInventoryBoard(historySections.value))
const prioritizedCurrentSections = computed(() => prioritizeInventorySections(currentSections.value))
const prioritizedHistorySections = computed(() => prioritizeInventorySections(historySections.value))
const currentFocusBlocks = computed(() => currentBoardStats.value.topBlocks.slice(0, 3))
const historyFocusBlocks = computed(() => historyBoardStats.value.topBlocks.slice(0, 3))
const currentInterestFocusBlocks = computed(() => currentBoardStats.value.topInterestBlocks.slice(0, 3))
const historyInterestFocusBlocks = computed(() => historyBoardStats.value.topInterestBlocks.slice(0, 3))
const currentRecentRefluxBuckets = computed(() =>
  buildRecentRefluxBuckets(currentSections.value, new Date(freshnessNowMs.value).toISOString()),
)
const currentHasRecentReflux = computed(() =>
  currentRecentRefluxBuckets.value.some((bucket) => bucket.items.length > 0),
)
const currentRecentRefluxPanelMode = computed(() =>
  resolveRecentRefluxPanelMode(currentHasRecentReflux.value, viewerMembershipTier.value),
)
const currentTrackedInterestSummary = computed(() => buildTrackedInterestSummary(currentTrackedInterests.value))
const currentDecisionLines = computed(() => buildBoardDecisionLines(currentBoardStats.value, 'current'))
const historyDecisionLines = computed(() => buildBoardDecisionLines(historyBoardStats.value, 'history'))
const pollIntervalSeconds = computed(() =>
  normalizeTicketWatchPollIntervalSeconds(currentUser.value?.ticket_watch_poll_interval_seconds),
)
const membershipBadgeTier = computed(() => formatTicketWatchMembershipBadgeTier(viewerMembershipTier.value))
const membershipExclusiveLabel = computed(() => `${membershipBadgeTier.value}专享`)
const membershipBenefitsLocked = computed(() =>
  resolveMembershipBenefitsLocked(currentUser.value),
)

function buildCurrentSectionCollapseKey(section: string): string {
  return `current:${currentMatch.value?.match_id ?? 0}:${section}`
}

function buildHistorySectionCollapseKey(section: string): string {
  return `history:${displayedHistoryMatch.value?.match_id ?? 0}:${section}`
}

function isCurrentSectionCollapsed(section: string): boolean {
  return isTicketWatchSectionCollapsed(currentCollapsedSections.value, buildCurrentSectionCollapseKey(section))
}

function isHistorySectionCollapsed(section: string): boolean {
  return isTicketWatchSectionCollapsed(historyCollapsedSections.value, buildHistorySectionCollapseKey(section))
}

function toggleCurrentSectionCollapsed(section: string): void {
  currentCollapsedSections.value = toggleTicketWatchSectionCollapsed(
    currentCollapsedSections.value,
    buildCurrentSectionCollapseKey(section),
  )
}

function toggleHistorySectionCollapsed(section: string): void {
  historyCollapsedSections.value = toggleTicketWatchSectionCollapsed(
    historyCollapsedSections.value,
    buildHistorySectionCollapseKey(section),
  )
}

onShareAppMessage(() => {
  if (activeTab.value === 'history' && displayedHistoryMatch.value) {
    return {
      title: `${displayedHistoryMatch.value.home_team_name} VS ${displayedHistoryMatch.value.away_team_name} 回流复盘`,
      path: '/pages/ticket-watch/index',
    }
  }

  if (currentMatch.value) {
    return {
      title: `${currentMatch.value.home_team_name} VS ${currentMatch.value.away_team_name} 回流看板`,
      path: '/pages/ticket-watch/index',
    }
  }

  return {
    title: '回流看板：当前比赛盯实时，历史比赛看回流',
    path: '/pages/ticket-watch/index',
  }
})

function resolveFallbackMatchId(match: Pick<TicketWatchMatchSummary, 'match_id' | 'external_match_id'> | null): number | null {
  if (!match?.external_match_id) {
    return null
  }

  const parsed = Number.parseInt(match.external_match_id, 10)

  if (!Number.isFinite(parsed) || parsed <= 0 || parsed === match.match_id) {
    return null
  }

  return parsed
}

async function ensureRegions(): Promise<void> {
  if (regions.value.length) {
    return
  }

  regions.value = await getTicketWatchRegions()
}

async function loadViewerMembershipTier(): Promise<void> {
  if (systemConfigUnderReview.value) {
    currentUser.value = null
    viewerMembershipTier.value = 'V1'
    return
  }

  try {
    const user = await getCurrentUser()
    currentUser.value = user
    viewerMembershipTier.value = user?.membership_tier?.trim() || 'V1'
  } catch {
    currentUser.value = null
    viewerMembershipTier.value = 'V1'
  }
}

async function getMatchInterestSections(matchId: number): Promise<TicketWatchBlockInterest[]> {
  return getTicketWatchBlockInterests(matchId)
}

async function loadCurrentTrackedInterests(matchId: number): Promise<void> {
  if (!currentUser.value) {
    currentTrackedInterests.value = []
    return
  }

  currentTrackedInterests.value = await getTicketWatchTrackedInterests(matchId)
}

async function refreshMatchInterests(matchId: number, mode: TicketWatchTab): Promise<void> {
  const interests = await getMatchInterestSections(matchId)

  if (mode === 'current') {
    currentSections.value = applyBlockInterestsToSections(currentSections.value, interests)
    return
  }

  const nextSections = applyBlockInterestsToSections(historySections.value, interests)
  historySections.value = nextSections
  historySectionsCache.value = {
    ...historySectionsCache.value,
    [matchId]: nextSections,
  }
}

function buildInterestToggleKey(matchId: number, blockName: string): string {
  return `${matchId}:${blockName}`
}

function isInterestToggleLoading(matchId: number, blockName: string): boolean {
  return Boolean(interestToggleLoading.value[buildInterestToggleKey(matchId, blockName)])
}

async function loadCurrentBoard(reason: TicketWatchLoadReason = 'initial'): Promise<void> {
  if (currentBoardRequestInFlight.value) {
    return
  }

  const strategy = resolveCurrentBoardLoadStrategy(hasLoadedCurrentBoard.value, reason)
  currentBoardRequestInFlight.value = true

  if (strategy.showBlockingLoading) {
    currentLoading.value = true
  }

  if (strategy.clearErrorBeforeLoad) {
    currentErrorMessage.value = ''
  }

  try {
    await ensureRegions()
    const response = await getCurrentTicketWatchBoard()
    currentMatch.value = response.current_match
    currentMessage.value = response.message || (response.group_ticket_active ? '当前为套票窗口。' : '暂无当前比赛。')

    if (response.current_match) {
      currentSections.value = applyBlockInterestsToSections(
        groupInventoryByPrice(regions.value, response.inventory),
        response.block_interests,
      )
      currentTrackedInterests.value = response.tracked_interests
    } else {
      currentSections.value = []
      currentTrackedInterests.value = []
    }
    hasLoadedCurrentBoard.value = true
  } catch (error) {
    if (reason === 'initial' || !hasLoadedCurrentBoard.value) {
      currentErrorMessage.value = extractApiErrorMessage(error, '当前比赛回流加载失败，请稍后重试。')
    }
  } finally {
    currentLoading.value = false
    currentBoardRequestInFlight.value = false
  }
}

async function loadHistoryMatches(reason: TicketWatchHistoryLoadReason = 'initial'): Promise<void> {
  if (historyBoardRequestInFlight.value) {
    return
  }

  const strategy = resolveHistoryBoardLoadStrategy(hasLoadedHistoryBoard.value, reason)
  historyBoardRequestInFlight.value = true

  if (strategy.showBlockingLoading) {
    historyLoading.value = true
  }

  if (strategy.clearErrorBeforeLoad) {
    historyErrorMessage.value = ''
  }

  try {
    await ensureRegions()
    const matches = await getTicketWatchMatches()
    historyMatches.value = selectCompletedMatches(matches)

    if (!historyMatches.value.length) {
      selectedHistoryMatchId.value = null
      historySections.value = []
      return
    }

    if (
      selectedHistoryMatchId.value === null ||
      !historyMatches.value.some((match) => match.match_id === selectedHistoryMatchId.value)
    ) {
      selectedHistoryMatchId.value = historyMatches.value[0].match_id
    }

    await syncDisplayedHistoryMatch(selectedHistoryMatchId.value)
    hasLoadedHistoryBoard.value = true
  } catch (error) {
    historyErrorMessage.value = extractApiErrorMessage(error, '历史比赛库存加载失败，请稍后重试。')
  } finally {
    historyLoading.value = false
    historyBoardRequestInFlight.value = false
  }
}

async function getHistoryInventorySections(matchId: number): Promise<TicketWatchGroupedInventorySection[]> {
  const cached = historySectionsCache.value[matchId]
  if (cached) {
    return cached
  }

  const match = historyMatches.value.find((item) => item.match_id === matchId) ?? null
  const [inventory, interests] = await Promise.all([
    getTicketWatchInventorySince(
      matchId,
      buildInventorySince(match?.sale_start_at),
      resolveFallbackMatchId(match),
    ),
    getMatchInterestSections(matchId),
  ])
  const grouped = applyBlockInterestsToSections(groupInventoryByPrice(regions.value, inventory), interests)
  historySectionsCache.value = {
    ...historySectionsCache.value,
    [matchId]: grouped,
  }
  return grouped
}

async function syncDisplayedHistoryMatch(matchId: number | null): Promise<void> {
  if (matchId === null) {
    displayedHistoryMatchId.value = null
    historySections.value = []
    return
  }

  const grouped = await getHistoryInventorySections(matchId)
  displayedHistoryMatchId.value = matchId
  historySections.value = grouped
}

async function handleHistoryMatchSelect(matchId: number): Promise<void> {
  if (selectedHistoryMatchId.value === matchId) {
    return
  }

  selectedHistoryMatchId.value = matchId
  const requestToken = ++historySelectionRequestToken
  historySelectionLoading.value = true

  try {
    const grouped = await getHistoryInventorySections(matchId)

    if (requestToken !== historySelectionRequestToken || selectedHistoryMatchId.value !== matchId) {
      return
    }

    displayedHistoryMatchId.value = matchId
    historySections.value = grouped
  } catch (error) {
    if (requestToken !== historySelectionRequestToken) {
      return
    }

    uni.showToast({
      title: extractApiErrorMessage(error, '该场比赛库存加载失败，请稍后重试。'),
      icon: 'none',
    })
  } finally {
    if (requestToken === historySelectionRequestToken) {
      historySelectionLoading.value = false
    }
  }
}

async function handleBlockInterestToggle(
  match: TicketWatchMatchSummary,
  item: TicketWatchGroupedInventoryItem,
  mode: TicketWatchTab,
): Promise<void> {
  if (mode !== 'current') {
    return
  }

  if (!item.viewer_interested) {
    pendingInterestSelection.value = {
      match,
      item,
      mode,
      blockName: item.block_name,
    }
    return
  }

  await submitBlockInterestToggle(match, item, mode)
}

async function submitBlockInterestToggle(
  match: TicketWatchMatchSummary,
  item: TicketWatchGroupedInventoryItem,
  mode: TicketWatchTab,
): Promise<void> {
  const toggleKey = buildInterestToggleKey(match.match_id, item.block_name)
  if (interestToggleLoading.value[toggleKey]) {
    return
  }

  interestToggleLoading.value = {
    ...interestToggleLoading.value,
    [toggleKey]: true,
  }

  try {
    const updatedInterest = await toggleTicketWatchBlockInterest(match.match_id, item.block_name)

    if (mode === 'current') {
      currentSections.value = applyBlockInterestToSections(currentSections.value, updatedInterest)
      await loadCurrentTrackedInterests(match.match_id)
    } else {
      const nextSections = applyBlockInterestToSections(historySections.value, updatedInterest)
      historySections.value = nextSections
      historySectionsCache.value = {
        ...historySectionsCache.value,
        [match.match_id]: nextSections,
      }
    }

    await refreshMatchInterests(match.match_id, mode)

    uni.showToast({
      title: updatedInterest.viewer_interested
        ? `已标记想抢 ${item.block_name}`
        : `已取消 ${item.block_name}`,
      icon: 'none',
    })
  } catch (error) {
    uni.showToast({
      title: extractApiErrorMessage(error, '标记想抢区域失败，请稍后重试。'),
      icon: 'none',
    })
  } finally {
    const nextState = { ...interestToggleLoading.value }
    delete nextState[toggleKey]
    interestToggleLoading.value = nextState
  }
}

function closeInterestConfirm(): void {
  pendingInterestSelection.value = null
}

async function confirmPendingInterestSelection(): Promise<void> {
  const selection = pendingInterestSelection.value
  if (!selection) {
    return
  }

  pendingInterestSelection.value = null
  await submitBlockInterestToggle(selection.match, selection.item, selection.mode)
}

function hasRectShape(value: unknown): value is { left: number; width: number } {
  return !!value
    && typeof value === 'object'
    && typeof (value as { left?: unknown }).left === 'number'
    && typeof (value as { width?: unknown }).width === 'number'
}

function hasScrollLeft(value: unknown): value is { scrollLeft: number } {
  return !!value
    && typeof value === 'object'
    && typeof (value as { scrollLeft?: unknown }).scrollLeft === 'number'
}

async function centerSelectedHistoryChip(): Promise<void> {
  if (!instance || selectedHistoryMatchId.value === null) {
    return
  }

  await nextTick()

  const query = uni.createSelectorQuery().in(instance)
  query.select('.history-chip-scroll').boundingClientRect()
  query.select('.history-chip-scroll').scrollOffset(() => {})
  query.select(`#history-chip-${selectedHistoryMatchId.value}`).boundingClientRect()
  query.exec((result) => {
    const [rawScrollRect, rawScrollOffset, rawChipRect] = (result ?? []) as unknown[]

    if (!hasRectShape(rawScrollRect) || !hasScrollLeft(rawScrollOffset) || !hasRectShape(rawChipRect)) {
      return
    }

    const scrollRect = rawScrollRect
    const scrollOffset = rawScrollOffset
    const chipRect = rawChipRect
    const delta = (chipRect.left + chipRect.width / 2) - (scrollRect.left + scrollRect.width / 2)
    const nextScrollLeft = Math.max(0, Math.round(scrollOffset.scrollLeft + delta))

    if (nextScrollLeft !== historyChipScrollLeft.value) {
      historyChipScrollLeft.value = nextScrollLeft
    }
  })
}

function startPolling(): void {
  if (activeTab.value !== 'current' || !isMonitoringActive.value) {
    return
  }

  stopPolling()
  pollTimer = setInterval(() => {
    void loadCurrentBoard('poll')
  }, pollIntervalSeconds.value * 1000)
}

function stopPolling(): void {
  if (pollTimer) {
    clearInterval(pollTimer)
    pollTimer = null
  }
}

function startFreshnessClock(): void {
  stopFreshnessClock()
  freshnessNowMs.value = Date.now()
  freshnessTimer = setInterval(() => {
    freshnessNowMs.value = Date.now()
  }, 15 * 1000)
}

function stopFreshnessClock(): void {
  if (freshnessTimer) {
    clearInterval(freshnessTimer)
    freshnessTimer = null
  }
}

async function handleStartMonitoring(): Promise<void> {
  if (isMonitoringActive.value) {
    return
  }

  isMonitoringActive.value = true
  await loadCurrentBoard('poll')
  startPolling()
}

function handleStopMonitoring(): void {
  if (!isMonitoringActive.value) {
    return
  }

  isMonitoringActive.value = false
  stopPolling()
}

async function toggleMonitoring(): Promise<void> {
  if (systemConfigUnderReview.value) {
    return
  }

  if (membershipBenefitsLocked.value) {
    uni.showToast({
      title: '当前账号已取关公众号，会员权益已暂停',
      icon: 'none',
      duration: 2200,
    })
    return
  }

  if (isMonitoringActive.value) {
    handleStopMonitoring()
    return
  }

  await handleStartMonitoring()
}

function formatLatestTime(value: string): string {
  return formatTrackedInterestTime(value)
}

function formatRecentRefluxMinuteLabel(minutesAgo: number): string {
  if (minutesAgo <= 0) {
    return '刚刚'
  }

  return `${minutesAgo}分钟前`
}

function isCurrentRecentRefluxBucketUnlocked(bucketKey: TicketWatchRecentRefluxBucketKey): boolean {
  return isRecentRefluxBucketUnlocked(bucketKey, viewerMembershipTier.value)
}

function formatRecentRefluxBucketRequiredTier(bucketKey: TicketWatchRecentRefluxBucketKey): string {
  return resolveRecentRefluxBucketRequiredTier(bucketKey)
}

function resolveInventoryCellClass(
  item: TicketWatchGroupedInventoryItem,
  section: TicketWatchGroupedInventorySection,
  matchId: number,
  mode: TicketWatchTab = 'current',
): Array<string | Record<string, boolean>> {
  const occurrences = item.occurrences
  const maxOccurrences = section.items.reduce((selected, item) => Math.max(selected, item.occurrences), 0)
  const heatLevel = resolveInventoryHeatLevel(occurrences, maxOccurrences)

  return [
    resolvePriceToneClass(section.price),
    {
      'inventory-cell--active': occurrences > 0,
      'inventory-cell--heat-1': heatLevel === 1,
      'inventory-cell--heat-2': heatLevel === 2,
      'inventory-cell--heat-3': heatLevel === 3,
      'inventory-cell--heat-4': heatLevel === 4,
      'inventory-cell--wanted': item.viewer_interested,
      'inventory-cell--busy': isInterestToggleLoading(matchId, item.block_name),
      'inventory-cell--readonly': mode === 'history',
    },
  ]
}

function goToUserPage(): void {
  uni.switchTab({ url: '/pages/user/index' })
}

function goToMembershipPurchase(): void {
  if (systemConfigUnderReview.value) {
    return
  }

  if (!currentUser.value?.has_wechat_binding) {
    goToUserPage()
    return
  }

  uni.navigateTo({ url: '/pages/membership-purchase/index' })
}

function resolvePriceToneClass(price: string): string {
  return `price-tone--${resolveInventoryPriceTone(price)}`
}

function formatPercent(value: number): string {
  return `${Math.round(value * 100)}%`
}

function formatPrice(price: string): string {
  return `¥${price}`
}

function formatCoverage(stats: TicketWatchBoardStats): string {
  return `${stats.activeRegionCount}/${stats.totalRegionCount}`
}

function formatHotPrice(stats: TicketWatchBoardStats): string {
  return stats.hottestPrice ? formatPrice(stats.hottestPrice.price) : '--'
}

function formatHotPriceMeta(stats: TicketWatchBoardStats): string {
  if (!stats.hottestPrice) {
    return '暂无热点'
  }

  return `${stats.hottestPrice.available_region_count} 区 / ${stats.hottestPrice.total_occurrences} 张`
}

function buildBoardInsightSummary(stats: TicketWatchBoardStats, mode: 'current' | 'history'): string {
  if (!stats.totalRegionCount) {
    return mode === 'current'
      ? '当前还没有足够的分区回流信号，先等下一波回流。'
      : '这场历史比赛暂时没有足够样本，先不要把它当成固定规律。'
  }

  const hottestPriceLabel = stats.hottestPrice ? formatPrice(stats.hottestPrice.price) : '暂无热点价位'
  const modeLabel = mode === 'current' ? '当前供给面' : '这场比赛当时的供给面'

  return `${modeLabel}里，${stats.activeRegionCount}/${stats.totalRegionCount} 个区域出现过回流，覆盖率 ${formatPercent(stats.activeRegionRatio)}。${hottestPriceLabel} 是最活跃的价格带，累计回流 ${stats.totalOccurrences} 张。`
    + ' 统计口径只看开售 10 分钟后的回流。'
}

function buildBoardDecisionLines(
  stats: TicketWatchBoardStats,
  mode: 'current' | 'history',
): string[] {
  if (!stats.totalRegionCount) {
    return [
      mode === 'current'
        ? '先观察，不要盲刷，当前还没有足够的分区回流信号。'
        : '这场比赛样本不足，先别把它当成下次抢票的稳定参考。',
    ]
  }

  const lines: string[] = []

  if (stats.hottestPrice) {
    lines.push(
      `${mode === 'current' ? '先盯' : '复盘先看'} ${formatPrice(stats.hottestPrice.price)}：${stats.hottestPrice.available_region_count} 区有回流，共 ${stats.hottestPrice.total_occurrences} 张回流。`,
    )
  }

  if (stats.topBlocks.length) {
    lines.push(
      `${mode === 'current' ? '重点刷新' : '历史高频区'} ${stats.topBlocks.map((item) => item.block_name).join('、')}：这些区域的回流最密。`,
    )
  }

  if (stats.cheapestActivePrice) {
    if (stats.hottestPrice && stats.cheapestActivePrice.price === stats.hottestPrice.price) {
      lines.push(`${formatPrice(stats.cheapestActivePrice.price)} 同时也是当前最低仍有回流的价位，预算优先就从这里进。`)
    } else {
      lines.push(`${mode === 'current' ? '想压预算' : '想找低价经验'}，先看 ${formatPrice(stats.cheapestActivePrice.price)}：这是最低仍有回流的价位。`)
    }
  }

  return lines.slice(0, 3)
}

onShow(() => {
  reportPageActivity('ticket_watch')
  void (async () => {
    pageEntered.value = true
    startFreshnessClock()
    systemConfigUnderReview.value = await loadSystemConfigUnderReview()
    if (systemConfigUnderReview.value) {
      stopPolling()
      currentLoading.value = false
      historyLoading.value = false
      currentUser.value = null
      viewerMembershipTier.value = 'V1'
      currentSections.value = []
      historySections.value = []
      return
    }
    await loadViewerMembershipTier()
    if (membershipBenefitsLocked.value) {
      stopPolling()
      return
    }
    await Promise.all([loadCurrentBoard('initial'), loadHistoryMatches()])
    if (isMonitoringActive.value) {
      startPolling()
    }
  })()
})

onReady(() => {
  void nextTick(() => {
    pageEntered.value = true
  })
})

watch(activeTab, (value) => {
  if (value === 'current' && isMonitoringActive.value) {
    startPolling()
    return
  }

  stopPolling()
})

watch(pollIntervalSeconds, () => {
  if (activeTab.value === 'current' && isMonitoringActive.value) {
    startPolling()
  }
})

watch(selectedHistoryMatchId, () => {
  void centerSelectedHistoryChip()
})

onHide(() => {
  stopPolling()
  stopFreshnessClock()
})

onUnload(() => {
  stopPolling()
  stopFreshnessClock()
  pageEntered.value = false
})
</script>

<style scoped lang="css">
.page {
  min-height: 100vh;
  padding: 20rpx 20rpx 36rpx;
  display: flex;
  flex-direction: column;
  gap: 18rpx;
  background: #f8f5ef;
}

.page--entry-mask {
  padding: 0;
}
.skeleton-stack {
  display: grid;
  gap: 18rpx;
}
.skeleton-panel {
  position: relative;
  overflow: hidden;
}
.skeleton-panel::before {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(
    90deg,
    rgba(255, 255, 255, 0) 0%,
    rgba(255, 255, 255, 0.62) 48%,
    rgba(255, 255, 255, 0) 100%
  );
  transform: translateX(-100%);
  animation: skeleton-shimmer 1.35s ease-in-out infinite;
  pointer-events: none;
}
.skeleton-copy-group {
  display: grid;
  gap: 12rpx;
  width: 100%;
}
.skeleton-line,
.skeleton-pill,
.skeleton-button,
.skeleton-focus-chip,
.skeleton-history-chip,
.skeleton-inventory-cell {
  background: linear-gradient(180deg, rgba(242, 244, 248, 0.98), rgba(233, 237, 243, 0.92));
  border: 2rpx solid rgba(230, 233, 239, 0.95);
}
.skeleton-line {
  border-radius: 999rpx;
}
.skeleton-line--kicker {
  width: 168rpx;
  height: 24rpx;
}
.skeleton-line--title {
  width: 430rpx;
  max-width: 100%;
  height: 56rpx;
  border-radius: 24rpx;
}
.skeleton-line--section {
  width: 320rpx;
  max-width: 100%;
  height: 40rpx;
  border-radius: 24rpx;
}
.skeleton-line--meta {
  margin-top: 18rpx;
  height: 24rpx;
}
.skeleton-line--meta-wide {
  width: 72%;
}
.skeleton-line--label {
  width: 110rpx;
  height: 22rpx;
}
.skeleton-line--metric {
  width: 120rpx;
  height: 42rpx;
  margin-top: 12rpx;
  border-radius: 20rpx;
}
.skeleton-line--body {
  width: 88%;
  height: 26rpx;
  margin-top: 18rpx;
}
.skeleton-pill {
  width: 150rpx;
  height: 52rpx;
  border-radius: 999rpx;
  flex-shrink: 0;
}
.skeleton-pill--short {
  width: 120rpx;
}
.skeleton-button {
  width: 100%;
  height: 72rpx;
  margin-top: 18rpx;
  border-radius: 999rpx;
}
.skeleton-metric {
  min-height: 134rpx;
}
.skeleton-focus-chip {
  min-height: 132rpx;
  border-radius: 22rpx;
  padding: 18rpx 14rpx;
  display: grid;
  align-content: start;
  gap: 10rpx;
}
.skeleton-line--chip-name {
  width: 64rpx;
  height: 28rpx;
}
.skeleton-line--chip-meta {
  width: 82rpx;
  height: 18rpx;
}
.skeleton-line--chip-count {
  width: 96rpx;
  height: 22rpx;
}
.skeleton-history-chip {
  min-width: 250rpx;
  border-radius: 26rpx;
  padding: 20rpx 22rpx;
  display: grid;
  gap: 12rpx;
}
.skeleton-line--history-date {
  width: 120rpx;
  height: 22rpx;
}
.skeleton-line--history-match {
  width: 190rpx;
  height: 24rpx;
}
.skeleton-inventory-cell {
  min-height: 170rpx;
  border-radius: 24rpx;
  padding: 22rpx 14rpx 16rpx;
  display: grid;
  justify-items: center;
  align-content: center;
  gap: 10rpx;
}
.skeleton-line--cell-name {
  width: 74rpx;
  height: 28rpx;
}
.skeleton-line--cell-count {
  width: 82rpx;
  height: 22rpx;
}
.skeleton-line--cell-time {
  width: 96rpx;
  height: 18rpx;
}
@keyframes skeleton-shimmer {
  0% {
    transform: translateX(-100%);
  }
  100% {
    transform: translateX(100%);
  }
}
.interest-confirm-mask {
  position: fixed;
  inset: 0;
  z-index: 20;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 32rpx;
  background: rgba(17, 19, 24, 0.28);
}
.interest-confirm-dialog {
  width: 100%;
  max-width: 560rpx;
  border-radius: 32rpx;
  background: rgba(255, 255, 255, 0.98);
  border: 2rpx solid rgba(236, 236, 241, 0.96);
  box-shadow: 0 28rpx 60rpx rgba(26, 28, 36, 0.18);
  padding: 32rpx 28rpx 24rpx;
}
.interest-confirm-dialog__title {
  display: block;
  color: #14161b;
  font-size: 32rpx;
  font-weight: 400;
  line-height: 1.1;
}
.interest-confirm-dialog__content {
  display: block;
  margin-top: 16rpx;
  color: #5f6570;
  font-size: 26rpx;
  line-height: 1.65;
}
.interest-confirm-dialog__actions {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14rpx;
  margin-top: 28rpx;
}
.interest-confirm-dialog__button {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 76rpx;
  padding: 0 24rpx;
  border-radius: 999rpx;
  font-size: 28rpx;
  font-weight: 700;
  line-height: 1;
  text-align: center;
}
.interest-confirm-dialog__button::after {
  border: none;
}
.interest-confirm-dialog__button--ghost {
  background: #f3f4f8;
  color: #737985;
}
.interest-confirm-dialog__button--primary {
  background: #15161b;
  color: #ffffff;
}
.hero-card, .panel, .state-card {
  animation: none;
  background: rgba(255,255,255,0.96);
  border-radius: 28rpx;
  padding: 24rpx;
  border: 2rpx solid rgba(238, 233, 224, 0.95);
  box-shadow: 0 12rpx 26rpx rgba(46, 38, 27, 0.06);
}
.hero-card {
  display: flex;
  align-items: center;
  gap: 18rpx;
}
.hero-card__top, .section-heading {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16rpx;
}
.hero-card__body {
  flex: 1;
  min-width: 0;
}
.hero-card__icon {
  flex-shrink: 0;
}
.hero-card__icon-box {
  width: 64rpx;
  height: 64rpx;
  border-radius: 18rpx;
  background: linear-gradient(180deg, #f7efe1, #f1e3ca);
  border: 2rpx solid rgba(220, 201, 165, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: inset 0 2rpx 0 rgba(255, 255, 255, 0.85);
}
.hero-card__icon-mark {
  color: #927445;
  font-size: 26rpx;
  font-weight: 800;
  line-height: 1;
}
.hero-card__action {
  flex-shrink: 0;
}
.hero-card__icon-image {
  width: 34rpx;
  height: 34rpx;
}
.inline-image-icon {
  width: 20rpx;
  height: 20rpx;
  margin-right: 8rpx;
  vertical-align: middle;
  flex-shrink: 0;
}
.inline-image-icon--small {
  width: 18rpx;
  height: 18rpx;
}
.section-heading__actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 12rpx;
  flex-shrink: 0;
}
.eyebrow, .section-kicker {
  margin: 0;
  color: #8f7c5f;
  font-size: 20rpx;
  font-weight: 400;
  letter-spacing: 0;
}
.hero-card__title, .section-title {
  display: block;
  margin-top: 6rpx;
  color: #121212;
  font-size: 32rpx;
  line-height: 1.18;
  font-weight: 400;
}
.section-title { font-size: 40rpx; }
.section-title-row {
  display: flex;
  align-items: center;
  gap: 10rpx;
  margin-top: 6rpx;
}
.section-title-icon-image {
  width: 40rpx;
  height: 40rpx;
  flex-shrink: 0;
}
.section-title--match {
  margin-top: 0;
  font-size: 34rpx;
  line-height: 1.12;
  font-weight: 400;
  white-space: normal;
  overflow: visible;
  text-overflow: clip;
}
.hero-card__summary {
  display: block;
  margin-top: 6rpx;
  color: #988f84;
  font-size: 20rpx;
  line-height: 1.45;
}
.hero-card__badge, .meta-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  white-space: nowrap;
  line-height: 1;
  padding: 12rpx 20rpx;
  border-radius: 999rpx;
  border: 2rpx solid rgba(230, 220, 198, 0.92);
  background: linear-gradient(180deg, rgba(255, 251, 242, 0.98), rgba(248, 241, 227, 0.94));
  color: #9c855c;
  font-size: 22rpx;
}
.collapse-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  min-height: 48rpx;
  padding: 0 20rpx;
  border-radius: 999rpx;
  border: 2rpx solid rgba(232, 226, 214, 0.96);
  background: #fffdfa;
  color: #8f836c;
  font-size: 22rpx;
  font-weight: 700;
  line-height: 1;
  text-align: center;
}
.collapse-pill::after {
  border: none;
}
.collapse-pill--inventory {
  min-height: 50rpx;
  padding: 0 22rpx;
}
.tab-switch {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12rpx;
  padding: 0;
  background: transparent;
  border: 0;
}
.tab-switch__item {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 64rpx;
  padding: 0 24rpx;
  border-radius: 999rpx;
  border: 2rpx solid rgba(235, 230, 221, 0.95);
  background: rgba(255, 255, 255, 0.96);
  color: #8a8d95;
  font-size: 24rpx;
  font-weight: 700;
  text-align: center;
  box-shadow: 0 8rpx 18rpx rgba(24, 28, 36, 0.04);
}
.tab-switch__item--active {
  border-color: #15161b;
  background: #15161b;
  color: #ffffff;
  box-shadow: 0 10rpx 20rpx rgba(21, 22, 27, 0.12);
}
.watch-match-card {
  display: flex;
  flex-direction: column;
  gap: 14rpx;
}
.watch-match-card--reference {
  gap: 20rpx;
  padding: 34rpx 32rpx 28rpx;
  border-radius: 34rpx;
  border-color: rgba(229, 231, 238, 0.98);
  background: linear-gradient(180deg, rgba(255, 255, 255, 1), rgba(250, 250, 252, 1));
  box-shadow:
    inset 0 1rpx 0 rgba(255, 255, 255, 0.96),
    0 12rpx 28rpx rgba(26, 28, 36, 0.05);
}
.watch-match-card__topline {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16rpx;
}
.watch-match-card__identity {
  display: flex;
  align-items: center;
  gap: 14rpx;
  flex: 1;
  min-width: 0;
}
.watch-match-card__ball {
  width: 40rpx;
  height: 40rpx;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.watch-match-card__ball-image {
  width: 40rpx;
  height: 40rpx;
}
.watch-match-card__title-wrap {
  flex: 1;
  min-width: 0;
  padding-top: 0;
}
.watch-match-card__membership-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6rpx;
  flex-shrink: 0;
  min-height: 40rpx;
  padding: 0 14rpx 0 12rpx;
  border-radius: 999rpx;
  border: 2rpx solid rgba(236, 222, 191, 1);
  background: linear-gradient(180deg, #fff8ea, #f8edd7);
  color: #9d7932;
  font-size: 18rpx;
  font-weight: 400;
  line-height: 1;
  box-shadow: inset 0 1rpx 0 rgba(255, 255, 255, 0.98);
}
.watch-match-card__membership-icon {
  width: 18rpx;
  height: 18rpx;
  flex-shrink: 0;
}
.watch-match-card__round {
  display: block;
  color: #848996;
  font-size: 22rpx;
  font-weight: 500;
  line-height: 1;
}
.watch-match-card__meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  color: #8d918f;
  font-size: 22rpx;
}
.watch-match-card__meta--reference {
  color: #848996;
  font-size: 22rpx;
  line-height: 1;
  padding-left: 68rpx;
  margin-top: -4rpx;
}
.watch-match-card__date {
  color: #6d7480;
  font-size: 22rpx;
  font-weight: 500;
}
.watch-metric__head {
  display: flex;
  align-items: center;
  justify-content: flex-start;
}
.watch-metric__icon-image {
  position: absolute;
  right: 18rpx;
  top: 50%;
  transform: translateY(-50%);
  width: 46rpx;
  height: 46rpx;
  flex-shrink: 0;
  opacity: 0.72;
}
.watch-match-card__note {
  display: block;
  color: #9c9387;
  font-size: 20rpx;
  line-height: 1.4;
  text-align: center;
}
.watch-match-card__note--reference {
  color: #8a909c;
  font-size: 20rpx;
  line-height: 1.2;
}
.watch-match-card__note-icon-image {
  width: 20rpx;
  height: 20rpx;
  margin-right: 8rpx;
  vertical-align: middle;
  opacity: 0.9;
}
.watch-metrics {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16rpx;
}
.watch-monitor-actions {
  display: flex;
  justify-content: center;
}
.watch-monitor-actions__button {
  width: 100%;
  min-height: 78rpx;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12rpx;
  border-radius: 22rpx;
  background: linear-gradient(180deg, #20242c, #191d26);
  color: #ffffff;
  font-size: 30rpx;
  font-weight: 800;
  text-align: center;
  box-shadow:
    inset 0 1rpx 0 rgba(255, 255, 255, 0.08),
    0 8rpx 18rpx rgba(21, 22, 27, 0.14);
}
.watch-monitor-actions__button-icon-image {
  width: 30rpx;
  height: 30rpx;
  flex-shrink: 0;
}
.watch-monitor-actions__button::after {
  border: none;
}
.watch-monitor-actions__button--ghost {
  background: linear-gradient(180deg, #eef1f6, #e5e9f0);
  color: #5c6370;
  box-shadow: inset 0 1rpx 0 rgba(255, 255, 255, 0.82);
}
.membership-lock-panel {
  display: flex;
  flex-direction: column;
  gap: 18rpx;
}
.membership-lock-panel__copy {
  color: #606574;
  font-size: 26rpx;
  line-height: 1.7;
}
.membership-lock-panel__action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  align-self: flex-start;
  min-width: 220rpx;
  min-height: 76rpx;
  padding: 0 28rpx;
  border-radius: 999rpx;
  background: #15161b;
  color: #ffffff;
  font-size: 28rpx;
  font-weight: 700;
  line-height: 1;
  text-align: center;
}
.membership-lock-panel__action::after {
  border: none;
}
.watch-metric {
  position: relative;
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
  min-height: 106rpx;
  border-radius: 22rpx;
  background: linear-gradient(180deg, #fbfbfd, #f6f7fb);
  border: 2rpx solid rgba(228, 231, 239, 1);
  padding: 16rpx 78rpx 18rpx 18rpx;
  box-shadow: inset 0 1rpx 0 rgba(255, 255, 255, 0.96);
}
.watch-metric__label {
  display: block;
  color: #9096a3;
  font-size: 22rpx;
  line-height: 1;
}
.watch-metric__value {
  display: block;
  margin-top: 10rpx;
  color: #16181d;
  font-size: 44rpx;
  font-weight: 800;
  line-height: 1;
}
.inventory-stack {
  display: grid;
  gap: 14rpx;
}
.price-tone--neutral {
  --ticket-tone-rgb: 107, 112, 123;
  --ticket-tone-strong-rgb: 62, 65, 72;
  --ticket-tone-text: #3e4148;
}
.price-tone--vip {
  --ticket-tone-rgb: 141, 69, 31;
  --ticket-tone-strong-rgb: 141, 69, 31;
  --ticket-tone-text: #8d451f;
}
.price-tone--s {
  --ticket-tone-rgb: 219, 47, 41;
  --ticket-tone-strong-rgb: 219, 47, 41;
  --ticket-tone-text: #db2f29;
}
.price-tone--a {
  --ticket-tone-rgb: 242, 191, 66;
  --ticket-tone-strong-rgb: 242, 191, 66;
  --ticket-tone-text: #b67f08;
}
.price-tone--b {
  --ticket-tone-rgb: 76, 167, 86;
  --ticket-tone-strong-rgb: 76, 167, 86;
  --ticket-tone-text: #2f7f39;
}
.price-tone--c {
  --ticket-tone-rgb: 47, 109, 167;
  --ticket-tone-strong-rgb: 47, 109, 167;
  --ticket-tone-text: #245888;
}
.price-tone--d {
  --ticket-tone-rgb: 21, 31, 59;
  --ticket-tone-strong-rgb: 21, 31, 59;
  --ticket-tone-text: #151f3b;
}
.price-tone--e {
  --ticket-tone-rgb: 94, 48, 142;
  --ticket-tone-strong-rgb: 94, 48, 142;
  --ticket-tone-text: #5e308e;
}
.recent-reflux-panel {
  position: relative;
  overflow: hidden;
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.98), rgba(247, 255, 250, 0.98));
  border-color: rgba(197, 226, 212, 0.95);
  box-shadow:
    inset 0 1rpx 0 rgba(255, 255, 255, 0.96),
    0 14rpx 28rpx rgba(28, 80, 56, 0.08);
}
.recent-reflux-panel__heading {
  display: grid;
  grid-template-columns: 34rpx minmax(0, 1fr);
  column-gap: 8rpx;
  align-items: center;
}
.recent-reflux-panel__heading .section-title {
  grid-column: 2;
}
.recent-reflux-panel__icon {
  grid-column: 1;
  align-self: center;
  width: 34rpx;
  height: 34rpx;
  border-radius: 12rpx;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(180deg, rgba(228, 248, 238, 0.98), rgba(247, 255, 251, 0.98));
  border: 2rpx solid rgba(56, 166, 115, 0.18);
}
.recent-reflux-panel__icon-image {
  width: 20rpx;
  height: 20rpx;
}
.recent-reflux-lock {
  position: relative;
  overflow: hidden;
  display: grid;
  grid-template-columns: 108rpx minmax(0, 1fr);
  gap: 18rpx;
  margin-top: 18rpx;
  padding: 24rpx;
  border-radius: 20rpx;
  background:
    linear-gradient(135deg, rgba(255,255,255,0.84), rgba(255,255,255,0.36)),
    linear-gradient(180deg, rgba(255, 250, 235, 0.96), rgba(242, 251, 246, 0.94));
  border: 2rpx solid rgba(214, 225, 203, 0.94);
}
.recent-reflux-lock::after {
  content: '';
  position: absolute;
  right: -38rpx;
  top: -48rpx;
  width: 190rpx;
  height: 190rpx;
  border-radius: 999rpx;
  background: radial-gradient(circle, rgba(239, 207, 123, 0.28), rgba(255,255,255,0) 68%);
  pointer-events: none;
}
.recent-reflux-lock__preview {
  position: relative;
  z-index: 1;
  height: 112rpx;
  border-radius: 18rpx;
  background: rgba(255, 255, 255, 0.68);
  border: 2rpx solid rgba(223, 229, 219, 0.94);
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8rpx;
}
.recent-reflux-lock__bar {
  width: 14rpx;
  border-radius: 999rpx;
  box-shadow: 0 0 18rpx currentColor;
}
.recent-reflux-lock__bar--red {
  height: 74rpx;
  color: rgba(224, 65, 55, 0.54);
  background: rgba(224, 65, 55, 0.78);
}
.recent-reflux-lock__bar--yellow {
  height: 56rpx;
  color: rgba(226, 168, 35, 0.5);
  background: rgba(226, 168, 35, 0.76);
}
.recent-reflux-lock__bar--green {
  height: 42rpx;
  color: rgba(40, 168, 109, 0.48);
  background: rgba(40, 168, 109, 0.74);
}
.recent-reflux-lock__body {
  position: relative;
  z-index: 1;
  min-width: 0;
}
.recent-reflux-lock__title {
  display: block;
  color: #15181d;
  font-size: 30rpx;
  font-weight: 800;
  line-height: 1.2;
}
.recent-reflux-lock__copy {
  display: block;
  margin-top: 10rpx;
  color: #667168;
  font-size: 22rpx;
  line-height: 1.5;
}
.recent-reflux-lock__action {
  position: relative;
  z-index: 1;
  grid-column: 1 / -1;
  height: 72rpx;
  line-height: 72rpx;
  border-radius: 999rpx;
  background: linear-gradient(135deg, #171a20, #2a302b);
  color: #ffffff;
  font-size: 26rpx;
  font-weight: 800;
  text-align: center;
}
.recent-reflux-lock__action::after {
  border: none;
}
.recent-reflux-buckets {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10rpx;
  margin-top: 18rpx;
}
.recent-reflux-bucket {
  min-width: 0;
  border-radius: 18rpx;
  padding: 16rpx 12rpx 14rpx;
  background: rgba(255, 255, 255, 0.82);
  border: 2rpx solid rgba(219, 231, 225, 0.92);
}
.recent-reflux-bucket--within3 {
  background: linear-gradient(180deg, rgba(232, 250, 240, 0.96), rgba(255, 255, 255, 0.9));
  border-color: rgba(40, 168, 109, 0.36);
}
.recent-reflux-bucket--within10 {
  background: linear-gradient(180deg, rgba(244, 250, 247, 0.98), rgba(255, 255, 255, 0.9));
}
.recent-reflux-bucket--within30 {
  background: linear-gradient(180deg, rgba(250, 249, 244, 0.98), rgba(255, 255, 255, 0.9));
}
.recent-reflux-bucket--locked {
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.88), rgba(249, 250, 247, 0.78));
  border-color: rgba(217, 226, 219, 0.9);
}
.recent-reflux-bucket__head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8rpx;
}
.recent-reflux-bucket__title {
  display: block;
  color: #15181d;
  font-size: 22rpx;
  font-weight: 800;
  line-height: 1.1;
}
.recent-reflux-bucket__subtitle,
.recent-reflux-bucket__empty {
  display: block;
  margin-top: 6rpx;
  color: #8a938d;
  font-size: 18rpx;
  line-height: 1.1;
}
.recent-reflux-bucket-lock {
  position: relative;
  overflow: hidden;
  margin-top: 14rpx;
  min-height: 82rpx;
  border-radius: 14rpx;
  padding: 12rpx;
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 8rpx;
  background:
    linear-gradient(135deg, rgba(255, 255, 255, 0.88), rgba(244, 247, 243, 0.82));
  border: 2rpx dashed rgba(179, 196, 185, 0.72);
  box-sizing: border-box;
}
.recent-reflux-bucket-lock:active {
  transform: scale(0.985);
  background:
    linear-gradient(135deg, rgba(246, 251, 247, 0.94), rgba(235, 244, 238, 0.9));
}
.recent-reflux-bucket-lock__icon {
  position: absolute;
  right: 12rpx;
  top: 14rpx;
  width: 34rpx;
  height: 34rpx;
  opacity: 0.72;
}
.recent-reflux-bucket-lock__shackle {
  position: absolute;
  left: 8rpx;
  top: 2rpx;
  width: 18rpx;
  height: 18rpx;
  border: 4rpx solid rgba(77, 98, 85, 0.58);
  border-bottom: 0;
  border-radius: 16rpx 16rpx 0 0;
  box-sizing: border-box;
}
.recent-reflux-bucket-lock__body {
  position: absolute;
  left: 4rpx;
  bottom: 2rpx;
  width: 26rpx;
  height: 20rpx;
  border-radius: 7rpx;
  background: rgba(77, 98, 85, 0.62);
  box-shadow: inset 0 2rpx 0 rgba(255, 255, 255, 0.34);
}
.recent-reflux-bucket-lock__body::after {
  content: '';
  position: absolute;
  left: 11rpx;
  top: 6rpx;
  width: 4rpx;
  height: 8rpx;
  border-radius: 999rpx;
  background: rgba(255, 255, 255, 0.72);
}
.recent-reflux-bucket-lock__tag {
  position: relative;
  z-index: 1;
  display: inline-flex;
  align-self: flex-start;
  max-width: 100%;
  height: 30rpx;
  line-height: 30rpx;
  padding: 0 10rpx;
  border-radius: 999rpx;
  background: rgba(22, 27, 25, 0.88);
  color: #ffffff;
  font-size: 17rpx;
  font-weight: 800;
  box-sizing: border-box;
}
.recent-reflux-bucket-lock__copy {
  position: relative;
  z-index: 1;
  display: block;
  color: #65706a;
  font-size: 18rpx;
  font-weight: 700;
  line-height: 1.1;
}
.recent-reflux-list {
  display: grid;
  gap: 8rpx;
  margin-top: 14rpx;
}
.recent-reflux-item {
  --recent-reflux-beam-rgb: 40, 168, 109;
  position: relative;
  overflow: hidden;
  min-width: 0;
  border-radius: 14rpx;
  padding: 10rpx 10rpx 9rpx;
  background: rgba(var(--ticket-tone-rgb), 0.08);
  border: 2rpx solid rgba(var(--ticket-tone-rgb), 0.16);
}
.recent-reflux-item--within3 {
  --recent-reflux-beam-rgb: 224, 65, 55;
  border-color: rgba(224, 65, 55, 0.36);
}
.recent-reflux-item--within10 {
  --recent-reflux-beam-rgb: 226, 168, 35;
  border-color: rgba(226, 168, 35, 0.36);
}
.recent-reflux-item--within30 {
  --recent-reflux-beam-rgb: 40, 168, 109;
  border-color: rgba(40, 168, 109, 0.34);
}
.recent-reflux-item__beam {
  position: absolute;
  z-index: 0;
  pointer-events: none;
  left: -72rpx;
  top: -3rpx;
  width: 72rpx;
  height: 6rpx;
  border-radius: 999rpx;
  background: linear-gradient(
    90deg,
    transparent,
    rgba(var(--recent-reflux-beam-rgb), 0.95),
    transparent
  );
  box-shadow: 0 0 18rpx rgba(var(--recent-reflux-beam-rgb), 0.38);
  animation: recent-reflux-item-border-beam 3.8s linear infinite;
}
.recent-reflux-item__name {
  display: block;
  color: var(--ticket-tone-text);
  font-size: 24rpx;
  font-weight: 800;
  line-height: 1;
  position: relative;
  z-index: 1;
}
.recent-reflux-item__price,
.recent-reflux-item__time {
  display: block;
  margin-top: 6rpx;
  font-size: 16rpx;
  line-height: 1;
  position: relative;
  z-index: 1;
}
.recent-reflux-item__price {
  color: rgba(var(--ticket-tone-strong-rgb), 0.72);
  font-weight: 700;
}
.recent-reflux-item__time {
  color: #65706a;
}
@keyframes recent-reflux-item-border-beam {
  0% {
    left: -72rpx;
    top: -3rpx;
    width: 72rpx;
    height: 6rpx;
  }
  34% {
    left: 100%;
    top: -3rpx;
    width: 72rpx;
    height: 6rpx;
  }
  35% {
    left: auto;
    right: -3rpx;
    top: -54rpx;
    width: 6rpx;
    height: 54rpx;
  }
  50% {
    left: auto;
    right: -3rpx;
    top: 100%;
    width: 6rpx;
    height: 54rpx;
  }
  51% {
    right: -72rpx;
    left: auto;
    top: auto;
    bottom: -3rpx;
    width: 72rpx;
    height: 6rpx;
  }
  84% {
    right: 100%;
    left: auto;
    top: auto;
    bottom: -3rpx;
    width: 72rpx;
    height: 6rpx;
  }
  85% {
    right: auto;
    left: -3rpx;
    bottom: -54rpx;
    width: 6rpx;
    height: 54rpx;
  }
  100% {
    right: auto;
    left: -3rpx;
    bottom: 100%;
    width: 6rpx;
    height: 54rpx;
  }
}
.focus-panel {
  background: linear-gradient(180deg, rgba(255, 252, 245, 0.98), rgba(255,255,255,0.98));
}
.focus-panel__heading {
  min-width: 0;
}
.focus-panel__headline {
  display: flex;
  align-items: center;
  gap: 10rpx;
  min-width: 0;
}
.focus-panel__headline-icon {
  width: 32rpx;
  height: 32rpx;
  flex-shrink: 0;
}
.focus-panel__headline-title {
  color: #17191f;
  font-size: 34rpx;
  font-weight: 400;
  line-height: 1.1;
  flex-shrink: 0;
}
.focus-panel__headline-subtitle {
  color: #9d9689;
  font-size: 20rpx;
  line-height: 1.2;
  white-space: nowrap;
}
.focus-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10rpx;
  margin-top: 14rpx;
}
.focus-grid--triple {
  grid-template-columns: repeat(3, minmax(0, 1fr));
  margin-top: 0;
}
.focus-section {
  display: flex;
  flex-direction: column;
  gap: 12rpx;
}
.focus-section + .focus-section {
  margin-top: 16rpx;
}
.focus-section__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16rpx;
}
.focus-section__title-row {
  display: flex;
  align-items: center;
  gap: 8rpx;
}
.focus-section__title-icon-image {
  width: 22rpx;
  height: 22rpx;
  flex-shrink: 0;
}
.focus-section__title {
  color: #17191f;
  font-size: 24rpx;
  font-weight: 400;
}
.focus-section__meta {
  color: #9a9489;
  font-size: 20rpx;
}
.focus-chip {
  position: relative;
  overflow: hidden;
  border-radius: 20rpx;
  padding: 18rpx 16rpx;
  background: linear-gradient(180deg, rgba(var(--ticket-tone-rgb), 0.08), rgba(var(--ticket-tone-rgb), 0.02));
  border: 2rpx solid rgba(var(--ticket-tone-rgb), 0.14);
  box-shadow: inset 0 1rpx 0 rgba(255,255,255,0.75);
}
.focus-chip--compact {
  min-height: 118rpx;
  padding: 16rpx 14rpx;
  border-radius: 18rpx;
}
.focus-chip__name-row {
  display: flex;
  align-items: center;
  gap: 6rpx;
}
.focus-chip__name {
  display: block;
  color: var(--ticket-tone-text);
  font-size: 30rpx;
  font-weight: 400;
  line-height: 1;
}
.focus-chip__name-icon-image {
  width: 16rpx;
  height: 16rpx;
  flex-shrink: 0;
}
.focus-chip--compact .focus-chip__name {
  font-size: 28rpx;
}
.focus-chip__meta {
  display: block;
  margin-top: 8rpx;
  color: rgba(var(--ticket-tone-strong-rgb), 0.7);
  font-size: 18rpx;
  font-weight: 700;
}
.focus-chip--compact .focus-chip__meta {
  font-size: 22rpx;
}
.focus-chip__count {
  display: block;
  margin-top: 8rpx;
  color: var(--ticket-tone-text);
  font-size: 24rpx;
  font-weight: 800;
}
.focus-chip--compact .focus-chip__count {
  font-size: 28rpx;
}
.focus-chip__count--interest {
  color: var(--ticket-tone-text);
}
.focus-copy {
  display: block;
  margin-top: 14rpx;
  color: #8f867b;
  font-size: 24rpx;
  line-height: 1.5;
}
.tracked-interest-panel {
  background: linear-gradient(180deg, rgba(255, 251, 244, 0.98), rgba(255,255,255,0.98));
}
.tracked-interest-summary {
  display: block;
  color: #847a6f;
  font-size: 24rpx;
  line-height: 1.6;
}
.tracked-interest-summary--empty {
  color: #8a7b61;
}
.tracked-interest-list {
  display: grid;
  gap: 10rpx;
  margin-top: 16rpx;
}
.tracked-interest-item {
  border-radius: 18rpx;
  padding: 18rpx 20rpx;
  background: linear-gradient(180deg, rgba(255,255,255,0.96), rgba(255,248,233,0.92));
  border: 2rpx solid rgba(226, 197, 132, 0.26);
}
.tracked-interest-item__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20rpx;
}
.tracked-interest-item__name {
  color: #17191f;
  font-size: 30rpx;
  font-weight: 400;
}
.tracked-interest-item__status {
  color: #8a7b61;
  font-size: 24rpx;
  font-weight: 700;
}
.tracked-interest-item__status--hit {
  color: #1f7a45;
}
.tracked-interest-item__meta {
  display: grid;
  gap: 8rpx;
  margin-top: 10rpx;
  color: #6b707b;
  font-size: 22rpx;
  line-height: 1.5;
}
.insight-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10rpx;
  margin-top: 16rpx;
}
.insight-metric {
  border-radius: 18rpx;
  background: linear-gradient(180deg, rgba(248, 249, 252, 0.96), rgba(255,255,255,1));
  border: 2rpx solid rgba(236, 236, 241, 0.94);
  padding: 16rpx;
}
.insight-metric__label {
  display: block;
  color: #8d919b;
  font-size: 18rpx;
}
.insight-metric__value {
  display: block;
  margin-top: 8rpx;
  color: #121212;
  font-size: 30rpx;
  font-weight: 800;
  line-height: 1.05;
}
.insight-metric__note {
  display: block;
  margin-top: 6rpx;
  color: #8d919b;
  font-size: 18rpx;
}
.insight-copy {
  display: block;
  margin-top: 14rpx;
  color: #847a6f;
  font-size: 20rpx;
  line-height: 1.6;
}
.decision-list {
  display: grid;
  gap: 10rpx;
  margin-top: 16rpx;
}
.decision-item {
  display: flex;
  align-items: flex-start;
  gap: 14rpx;
  padding: 16rpx 18rpx;
  border-radius: 18rpx;
  background: linear-gradient(180deg, rgba(250, 248, 242, 0.94), rgba(255,255,255,1));
  border: 2rpx solid rgba(229, 223, 205, 0.76);
}
.decision-item__dot {
  width: 14rpx;
  height: 14rpx;
  margin-top: 10rpx;
  flex-shrink: 0;
  border-radius: 999rpx;
  background: #17181b;
}
.decision-item__text {
  color: #3e4148;
  font-size: 20rpx;
  line-height: 1.55;
}
.inventory-panel__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16rpx;
}
.inventory-panel__actions {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 12rpx;
}
.inventory-panel__price {
  display: block;
  color: var(--ticket-tone-text);
  font-size: 44rpx;
  font-weight: 800;
  line-height: 1;
}
.inventory-panel__meta, .inventory-panel__summary {
  display: block;
  margin-top: 6rpx;
  color: #9a958a;
  font-size: 24rpx;
}
.inventory-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 10rpx;
  margin-top: 14rpx;
}
.inventory-cell {
  position: relative;
  overflow: hidden;
  border-radius: 18rpx;
  background: #fbfbfc;
  padding: 18rpx 10rpx 14rpx;
  min-height: 138rpx;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  gap: 6rpx;
  transition: transform 0.18s ease, box-shadow 0.18s ease, border-color 0.18s ease;
}
.inventory-cell::after {
  border: none;
}
.inventory-cell--pressed {
  transform: scale(0.98);
}
.inventory-cell--wanted {
  box-shadow: 0 16rpx 30rpx rgba(var(--ticket-tone-rgb), 0.16);
}
.inventory-cell--busy {
  opacity: 0.72;
}
.inventory-cell--readonly {
  cursor: default;
}
.inventory-cell--active {
  background: linear-gradient(180deg, rgba(var(--ticket-tone-rgb), 0.16), rgba(var(--ticket-tone-rgb), 0.07));
  border: 2rpx solid rgba(var(--ticket-tone-rgb), 0.28);
}
.inventory-cell--heat-1 {
  background: linear-gradient(180deg, rgba(var(--ticket-tone-rgb), 0.12), rgba(255, 255, 255, 0.98));
  border: 2rpx solid rgba(var(--ticket-tone-rgb), 0.32);
}
.inventory-cell--heat-2 {
  background: linear-gradient(180deg, rgba(var(--ticket-tone-rgb), 0.22), rgba(255, 255, 255, 0.98));
  border: 2rpx solid rgba(var(--ticket-tone-rgb), 0.4);
}
.inventory-cell--heat-3 {
  background: linear-gradient(180deg, rgba(var(--ticket-tone-rgb), 0.34), rgba(255, 255, 255, 0.98));
  border: 2rpx solid rgba(var(--ticket-tone-strong-rgb), 0.48);
}
.inventory-cell--heat-4 {
  background: linear-gradient(180deg, rgba(var(--ticket-tone-rgb), 0.48), rgba(255, 255, 255, 0.98));
  border: 2rpx solid rgba(var(--ticket-tone-strong-rgb), 0.62);
  box-shadow: inset 0 0 0 2rpx rgba(255,255,255,0.2);
}
.inventory-cell__name {
  color: #121212;
  font-size: 34rpx;
  font-weight: 400;
  line-height: 1;
  position: relative;
  z-index: 1;
}
.inventory-cell__count {
  color: var(--ticket-tone-text);
  font-size: 24rpx;
  font-weight: 700;
  position: relative;
  z-index: 1;
}
.inventory-cell__time {
  color: #838892;
  font-size: 20rpx;
  position: relative;
  z-index: 1;
}
.inventory-cell__interest-strip {
  margin-top: auto;
  width: 100%;
  min-height: 34rpx;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: flex-end;
  gap: 6rpx;
  padding: 0 6rpx;
  position: relative;
  z-index: 1;
}
.inventory-cell__interest-meter {
  display: flex;
  align-items: center;
  gap: 4rpx;
}
.inventory-cell__interest-bar {
  width: 12rpx;
  height: 5rpx;
  border-radius: 999rpx;
  background: rgba(171, 53, 44, 0.14);
}
.inventory-cell__interest-bar--active {
  background: #ab352c;
  box-shadow: 0 2rpx 8rpx rgba(171, 53, 44, 0.16);
}
.inventory-cell__interest-label {
  color: #ab352c;
  font-size: 16rpx;
  font-weight: 700;
  line-height: 1;
}
.history-chip-scroll {
  width: 100%;
  margin-top: 14rpx;
}
.history-chip-list {
  display: inline-flex;
  gap: 10rpx;
  padding-bottom: 6rpx;
}
.history-chip {
  min-width: 240rpx;
  border-radius: 20rpx;
  padding: 16rpx 18rpx;
  background: #f3f4f8;
  text-align: left;
}
.history-chip--active {
  background: #15161b;
}
.history-chip__date {
  display: block;
  color: #7f838d;
  font-size: 18rpx;
}
.history-chip__teams {
  display: block;
  margin-top: 8rpx;
  color: #17181b;
  font-size: 20rpx;
  font-weight: 400;
  line-height: 1.45;
}
.history-chip--active .history-chip__date,
.history-chip--active .history-chip__teams {
  color: #ffffff;
}
.state-card--error text, .state-card--empty text {
  color: #6b707b;
  font-size: 28rpx;
  line-height: 1.7;
}
</style>
