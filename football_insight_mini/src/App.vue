<script setup lang="ts">
import { onLaunch, onShow, onHide } from "@dcloudio/uni-app";
import { initWechatCloud } from "./utils/wechatCloud";
onLaunch(() => {
  initWechatCloud();
  console.log("Football Insight mini launch");
});
onShow(() => {
  console.log("Football Insight mini show");
});
onHide(() => {
  console.log("Football Insight mini hide");
});
</script>
<style lang="css">
@import "./uni.css";

page {
  min-height: 100%;
  background:
    linear-gradient(180deg, rgba(233, 234, 241, 0.95) 0, rgba(244, 243, 246, 0.6) 180rpx),
    #f6f5f2;
  color: #15161b;
  font-family: "PingFang SC", "Helvetica Neue", sans-serif;
}

view,
text,
button,
input,
textarea,
scroll-view {
  box-sizing: border-box;
}

button {
  margin: 0;
  padding: 0;
  background: transparent;
  border: 0;
  line-height: inherit;
  transition: transform 180ms ease, opacity 180ms ease, background-color 180ms ease;
}

button::after {
  border: 0;
}

button:active {
  transform: scale(0.985);
  opacity: 0.92;
}

.hero-card,
.panel,
.state-card,
.briefing-card,
.standings-launcher-card,
.match-card,
.team-board-launcher__card {
  animation: fi-fade-up 240ms ease both;
}

.ranking-row__avatar,
.team-board-post__author-avatar,
.team-board-snapshot__avatar,
.contribution-row__avatar,
.insights-team-chip__avatar,
.briefing-card__entity-avatar {
  background:
    radial-gradient(circle at 30% 28%, rgba(255, 255, 255, 0.92), rgba(245, 246, 250, 0.96) 70%, rgba(236, 238, 244, 0.98) 100%);
  box-shadow: 0 8rpx 18rpx rgba(18, 18, 18, 0.06);
}

@keyframes fi-fade-up {
  from {
    opacity: 0;
    transform: translateY(10rpx);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes fi-overlay-fade-in {
  from {
    opacity: 0;
  }

  to {
    opacity: 1;
  }
}

@keyframes fi-sheet-up {
  from {
    opacity: 0;
    transform: translateY(28rpx) scale(0.985);
  }

  68% {
    opacity: 1;
    transform: translateY(-4rpx) scale(1.01);
  }

  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

@keyframes fi-value-pop {
  from {
    opacity: 0.36;
    transform: scale(0.92);
  }

  58% {
    opacity: 1;
    transform: scale(1.08);
  }

  to {
    opacity: 1;
    transform: scale(1);
  }
}

@keyframes fi-rise-in {
  from {
    opacity: 0;
    transform: translateY(12rpx);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
