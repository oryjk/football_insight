<template>
 <page-meta :page-style="navigationPageStyle + pageLockStyle" />
 <FiSecondaryNav @back="goBackFromSecondaryPage('/pages/user/index')" @share="copyPageShareLink(sharePath)" />
 <view class="recap-page">
  <view class="recap-heading">
   <text class="recap-title">{{season}} 我的赛季足迹</text>
   <text class="recap-caption">勾选去过的蓉城比赛，生成你的里程海报</text>
  </view>
  <view v-if="error && session.hasAccessToken.value" class="recap-error">
   <text class="recap-error__text">{{error}}</text>
   <button v-if="!view" class="recap-error__retry" hover-class="pressed" @tap="load">重新加载</button>
  </view>
  <FiPanel v-if="!session.hasAccessToken.value">
   <text class="caption">登录后记录观赛足迹，换设备也能继续查看。</text>
   <button class="action" hover-class="pressed" @tap="login">登录并制作</button>
  </FiPanel>
  <FiLoading v-else-if="loading" title="正在整理赛季比赛" caption="只展示成都蓉城一线队已结束的比赛" />
  <template v-if="view">
   <FiPanel>
    <SeasonRecapSummary :summary="summaryPreview" :season="season" :dirty="dirty" />
   </FiPanel>
   <FiPanel>
    <SeasonRecapMatches :matches="view.matches" :selected="selected" :disabled="generating" @toggle="toggle" @select="setSelected" />
    <view v-if="hasHomeSelected" class="distance">
     <view class="distance__field">
      <text class="distance__label">去主场单边距离</text>
      <input class="distance__input" type="digit" :value="distance" :disabled="generating" maxlength="6" placeholder="0–500" aria-label="去主场的单边距离，公里" @input="handleDistanceInput" />
      <text class="distance__unit">公里</text>
     </view>
     <text class="distance__hint">填一次即可，会记住</text>
    </view>
   </FiPanel>
   <view class="formula">
    <button class="formula__toggle" hover-class="pressed" :aria-expanded="formulaOpen" @tap="formulaOpen=!formulaOpen">
     <text class="formula__label">里程怎么算</text>
     <text class="formula__arrow" :class="{'formula__arrow--open':formulaOpen}">›</text>
    </button>
    <FiCollapse :open="formulaOpen">
     <text class="formula__body">客场：成都至对手主场城市的大圆直线距离 × 2，每场取整后相加；主场：单边距离 × 2 × 场次，合计取整；取消或未结束的比赛不计入。</text>
    </FiCollapse>
   </view>
  </template>
  <SeasonRecapPoster v-if="generated" :key="generated.key" :payload="generated" @ready="handlePosterReady" @failed="handlePosterFailed" />
 </view>
 <SeasonRecapDock v-if="showDock" :stats="selectionStats" :generating="generating" :has-poster="!!posterPath" @generate="handleGenerate" @open="posterSheetVisible=true" />
 <SeasonRecapPosterSheet v-model:visible="posterSheetVisible" :poster-path="posterPath" @save="saveImage" @preview="preview" @copy="copyPageShareLink(sharePath)" />
</template>
<script setup lang="ts">
import { computed, ref } from 'vue'
import { onLoad,onShow,onUnload,onShareAppMessage,onShareTimeline } from '@dcloudio/uni-app'
import FiSecondaryNav from '../../components/FiSecondaryNav.vue'
import FiPanel from '../../components/FiPanel.vue'
import FiLoading from '../../components/FiLoading.vue'
import FiCollapse from '../../components/FiCollapse.vue'
import SeasonRecapSummary from './components/SeasonRecapSummary.vue'
import SeasonRecapMatches from './components/SeasonRecapMatches.vue'
import SeasonRecapDock from './components/SeasonRecapDock.vue'
import SeasonRecapPosterSheet from './components/SeasonRecapPosterSheet.vue'
import SeasonRecapPoster from './components/SeasonRecapPoster.vue'
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import { useAppSession } from '../../stores/appSession'
import { getSeasonRecap,saveSeasonRecap,getSeasonRecapCode } from '../../api/seasonRecap'
import { setPostLoginRedirectTarget } from '../../utils/authStorage'
import { copyPageShareLink,goBackFromSecondaryPage } from '../../utils/secondaryNavigation'
import type { RecapSummary } from '../../types/seasonRecap'
import { recapSharePath,recapSelectionStats,previewRecapSummary,resolveRecapSeason } from './helpers'
import { createRecapFlow } from './flow'
import { saveRecapPoster } from './poster'
const session=useAppSession()
const {navigationPageStyle}=useNavigationLayout()
const flow=createRecapFlow(session,{load:getSeasonRecap,save:saveSeasonRecap,code:getSeasonRecapCode})
const {season,view,selected,distance,loading,generating,error,posterPath,generated,dirty,load,toggle,setSelected,editDistance,generate,posterReady,posterFailed}=flow
const sharePath=computed(()=>recapSharePath(season.value))
const posterSheetVisible=ref(false)
const formulaOpen=ref(false)
const pageLockStyle=computed(()=>posterSheetVisible.value?'overflow: hidden;':'')
const showDock=computed(()=>session.hasAccessToken.value&&!loading.value&&!!view.value)
const hasHomeSelected=computed(()=>!!view.value&&view.value.matches.some(m=>m.is_home&&selected.value.includes(m.match_id)))
const selectionStats=computed(()=>recapSelectionStats(view.value?.matches??[],selected.value))
// 勾选/距离修改实时联动总览卡；旧后端缺 city/round_trip_km 时回退到已保存结果。
const EMPTY_SUMMARY:RecapSummary={away_km:0,home_km:null,total_km:0,matches:0,home_matches:0,away_matches:0,cities:0}
const summaryPreview=computed<RecapSummary>(()=>{
 const current=view.value
 if(!current)return EMPTY_SUMMARY
 return previewRecapSummary(current.matches,selected.value,distance.value)??current.summary
})
onLoad(query=>{season.value=resolveRecapSeason(query??{})})
onShow(()=>{if(!view.value)void load()})
onUnload(flow.dispose)
// vue-tsc resolves <input> to the H5 InputEvent; uni-app supplies the same detail.value payload on both runtimes.
function handleDistanceInput(event: InputEvent) {
 const detail = event.detail as unknown as { value: string }
 editDistance(detail.value)
}
// dock 在页面底部，生成失败时顶部错误可能不可见，同步用 toast 提示。
async function handleGenerate(){error.value='';await generate();if(error.value)uni.showToast({title:error.value,icon:'none'})}
function handlePosterReady(generation: number, path: string){posterReady(generation,path);if(posterPath.value===path)posterSheetVisible.value=true}
function handlePosterFailed(generation: number){posterFailed(generation);if(error.value==='海报生成失败，请重试')uni.showToast({title:error.value,icon:'none'})}
function login(){setPostLoginRedirectTarget({url:sharePath.value,type:'navigateTo'});uni.switchTab({url:'/pages/user/index'})}
function preview(){if(posterPath.value)uni.previewImage({urls:[posterPath.value]})}
function saveImage(){void saveRecapPoster(posterPath.value)}
// Share metadata contains no account or selection identifiers; recipients make their own recap.
onShareAppMessage(()=>({title:`我的 ${season.value} 蓉城赛季足迹`,path:sharePath.value,...(posterPath.value?{imageUrl:posterPath.value}:{})}))
onShareTimeline(()=>({title:`我的 ${season.value} 蓉城赛季足迹`,query:`season=${season.value}&src=share&cid=recap${season.value}`,...(posterPath.value?{imageUrl:posterPath.value}:{})}))
</script>
<style scoped>
.recap-page {min-height:100vh;box-sizing:border-box;display:flex;flex-direction:column;gap:var(--fi-space-card-gap);padding:calc(var(--fi-brand-nav-height) + var(--fi-space-24)) var(--fi-space-24) calc(var(--fi-space-64) + var(--fi-space-64) + var(--fi-space-40) + env(safe-area-inset-bottom));background:var(--fi-color-page);color:var(--fi-color-text-primary);}
.recap-heading {margin-bottom:var(--fi-space-12);}
.recap-title {display:block;color:var(--fi-color-text-primary);font-size:var(--fi-font-36);font-weight:var(--fi-weight-extrabold);}
.recap-caption {display:block;margin-top:var(--fi-space-12);color:var(--fi-color-text-secondary);font-size:var(--fi-font-24);}
.caption {display:block;margin-top:var(--fi-space-4);color:var(--fi-color-text-secondary);font-size:var(--fi-font-24);line-height:var(--fi-leading-normal);}
.recap-error {display:flex;flex-direction:column;gap:var(--fi-space-8);}
.recap-error__text {color:var(--fi-color-primary);font-size:var(--fi-font-28);line-height:var(--fi-leading-normal);}
.recap-error__retry {align-self:flex-start;margin:0;padding:var(--fi-space-12) var(--fi-space-28);background:var(--fi-color-surface);color:var(--fi-color-primary);border:0;border-radius:var(--fi-radius-round);font-size:var(--fi-font-26);font-weight:var(--fi-weight-bold);line-height:var(--fi-leading-normal);}
.recap-error__retry::after {border:0;}
.action {margin:var(--fi-space-20) 0 0;padding:var(--fi-space-20) var(--fi-space-24);background:var(--fi-color-primary);color:var(--fi-color-surface);border:0;border-radius:var(--fi-radius-round);font-size:var(--fi-font-28);font-weight:var(--fi-weight-bold);line-height:var(--fi-leading-normal);}
.action::after {border:0;}
.pressed {opacity:var(--fi-opacity-pressed);}
.distance {margin-top:var(--fi-space-16);padding-top:var(--fi-space-16);border-top:var(--fi-border-width) solid var(--fi-color-divider-soft);}
.distance__field {display:flex;align-items:center;gap:var(--fi-space-16);font-size:var(--fi-font-28);}
.distance__label {flex-shrink:0;font-size:var(--fi-font-26);font-weight:var(--fi-weight-bold);color:var(--fi-color-text-primary);}
.distance__input {flex:1;min-width:0;height:var(--fi-space-64);box-sizing:border-box;padding:0 var(--fi-space-20);background:var(--fi-color-page);border-radius:var(--fi-radius-sm);font-size:var(--fi-font-32);line-height:var(--fi-leading-normal);}
.distance__unit {flex-shrink:0;font-size:var(--fi-font-26);color:var(--fi-color-text-secondary);}
.distance__hint {display:block;margin-top:var(--fi-space-8);font-size:var(--fi-font-22);color:var(--fi-color-text-muted);}
.formula__toggle {display:flex;align-items:center;gap:var(--fi-space-8);margin:0;padding:var(--fi-space-8) var(--fi-space-4);background:transparent;border:0;line-height:var(--fi-leading-normal);}
.formula__toggle::after {border:0;}
.formula__label {font-size:var(--fi-font-26);color:var(--fi-color-text-secondary);}
.formula__arrow {font-size:var(--fi-font-26);color:var(--fi-color-text-muted);transition:transform var(--fi-motion-duration-base) var(--fi-motion-ease-out);}
.formula__arrow--open {transform:rotate(90deg);}
.formula__body {display:block;padding:var(--fi-space-4) var(--fi-space-4) 0;font-size:var(--fi-font-24);line-height:var(--fi-leading-relaxed);color:var(--fi-color-text-muted);}
</style>
