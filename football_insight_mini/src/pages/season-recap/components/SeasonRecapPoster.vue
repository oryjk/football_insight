<template>
 <canvas canvas-id="seasonRecapCanvas" class="recap-canvas" :style="canvasStyle" />
</template>
<script setup lang="ts">
import { getCurrentInstance, onMounted, nextTick } from 'vue'
import { exportCanvasToTempFile, flushCanvas, traceRoundRect } from '../../../utils/canvasPoster'
import type { RecapPosterPayload } from '../flow'
import { materializeCode, posterLines, POSTER_WIDTH, POSTER_HEIGHT, POSTER_PALETTE as color } from '../poster'
const props=defineProps<{payload:RecapPosterPayload}>()
const emit=defineEmits<{(event:'ready',key:number,path:string):void;(event:'failed',key:number):void}>()
const instance=getCurrentInstance()
// Export bitmap geometry is in pixels; hidden canvas must match drawing coordinates on both runtimes.
const canvasStyle={width:`${POSTER_WIDTH}px`,height:`${POSTER_HEIGHT}px`}
onMounted(async()=>{
 const payload=props.payload
 let cleanup=()=>{}
 try {
  await nextTick()
  const code=await materializeCode(payload.code);cleanup=code.cleanup
  const image=await new Promise<UniApp.GetImageInfoSuccessData>((resolve,reject)=>uni.getImageInfo({src:code.path,success:resolve,fail:reject}))
  const context=uni.createCanvasContext('seasonRecapCanvas',instance)
  context.setFillStyle(color.background);context.fillRect(0,0,POSTER_WIDTH,POSTER_HEIGHT)
  context.setFillStyle(color.paper);traceRoundRect(context,28,28,694,1024,28);context.fill()
  const text=(value:string,x:number,y:number,size:number,fill=color.ink)=>{context.setFillStyle(fill);context.setFontSize(size);context.fillText(value,x,y)}
  const summary=payload.view.summary,lines=posterLines(summary)
  text('成都蓉城 · 我的赛季总结',60,100,28)
  text(String(payload.view.season),60,166,54,color.accent)
  text('为蓉城走过的每一公里',60,216,28)
  text('客场里程',60,300,26,color.muted);text(lines[0]!,60,372,58,color.accent)
  text(lines[1]!,60,416,22,color.muted)
  text('主场里程',60,492,26,color.muted);text(lines[2]!,60,560,48)
  text(lines[3]!,60,646,30)
  text(`合计 ${summary.total_km} 公里`,60,700,26,color.muted)
  context.drawImage(image.path,510,764,160,160)
  text('足球洞察',60,818,32)
  text('这一年，与你同场',60,866,24,color.muted)
  text('扫码制作我的赛季总结',460,958,20,color.muted)
  text('自报观赛 · 客场为城市间直线往返估算',60,1008,20,color.muted)
  await flushCanvas(context)
  const path=await exportCanvasToTempFile({canvasId:'seasonRecapCanvas',width:POSTER_WIDTH,height:POSTER_HEIGHT,instance})
  emit('ready',payload.key,path)
 } catch {emit('failed',payload.key)} finally {cleanup()}
})
</script>
<style scoped>
.recap-canvas { position: fixed; left: -10000rpx; top: -10000rpx; pointer-events: none; }
</style>
