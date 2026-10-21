import { expect, test } from 'bun:test'
import { CHENGDU_RONGCHENG_CREST_IMAGE_URL, SEASON_RECAP_POSTER_BACKGROUNDS } from '../../config/assets'
import { buildRecapPosterContent } from './poster'
import { drawRecapPoster } from './posterDrawing'
import type { RecapPosterImage } from './posterImages'

function canvas(rejectImages = false) {
 const calls: Array<[string, ...unknown[]]> = []
 let depth = 0
 const context = {
  font:'',
  measureText(value:string) { return {width:[...value].length * (Number(/(\d+)px/.exec(this.font)?.[1]) || 16)} },
  drawImage(path:string, ...args:number[]) { calls.push(['image',path,...args]); if(rejectImages && path!=='/code') throw new Error('decode') },
  fillText(value:string) { calls.push(['text',value]) },
  fillRect(...args:number[]) { calls.push(['rect',...args]) },
  createLinearGradient(...args:number[]) { calls.push(['gradient',...args]); return {addColorStop(){}} },
  setFillStyle(){}, setStrokeStyle(){}, setLineWidth(){}, setLineCap(){}, setLineJoin(){}, setTextAlign(){}, setTextBaseline(){}, setGlobalAlpha(){},
  beginPath(){}, moveTo(){}, lineTo(){}, arc(){}, arcTo(){}, closePath(){}, fill(){}, stroke(){},
  clip(){ calls.push(['clip']) },
  save(){ depth++ }, restore(){ depth-- },
 }
 return {context:context as unknown as UniApp.CanvasContext,calls,depth:()=>depth}
}
const content=buildRecapPosterContent({season:2026,matches:[{match_id:1,match_date:'2026-01-01',opponent:'北京国安',opponent_avatar_url:'https://crest',is_home:false,city:'北京'}],selected_match_ids:[1],home_one_way_km:null,summary:{away_km:3000,home_km:null,total_km:3000,matches:1,home_matches:0,away_matches:1,cities:1}})
const image=(path:string):RecapPosterImage=>({path,width:100,height:100})

test('photo is drawn first with local paths, route crests are clipped and all information survives',()=>{
 const c=canvas()
 drawRecapPoster(c.context,content,'/code',new Map([
  [SEASON_RECAP_POSTER_BACKGROUNDS.dark,image('/bg')],
  [CHENGDU_RONGCHENG_CREST_IMAGE_URL,image('/chengdu')],
  ['https://crest',image('/beijing')],
 ]))
 expect(c.calls[0]).toEqual(['image','/bg',0,0,750,1333])
 expect(c.calls.filter(call=>call[0]==='image').map(call=>call[1])).toEqual(['/bg','/chengdu','/beijing','/code'])
 expect(c.calls.filter(call=>call[0]==='clip').length).toBe(0)
 const texts=c.calls.filter(call=>call[0]==='text').map(call=>call[1])
 expect(texts.filter(value=>['3000','北京','主场里程','蓉城同行者','自报观赛 · 客场为城市间直线往返估算'].includes(String(value)))).toEqual([
  '3000','蓉城同行者','主场里程','北京','自报观赛 · 客场为城市间直线往返估算',
 ])
 expect(c.depth()).toBe(0)
})

test('missing decorations or image decode errors still render city, stats and code',()=>{
 for(const images of [new Map<string,RecapPosterImage>(),new Map([
  [SEASON_RECAP_POSTER_BACKGROUNDS.dark,image('/bad-bg')],
  [CHENGDU_RONGCHENG_CREST_IMAGE_URL,image('/bad-chengdu')],
  ['https://crest',image('/bad-opponent')],
 ])]) {
  const c=canvas(true)
  drawRecapPoster(c.context,content,'/code',images)
  expect(c.calls.find(call=>call[0]==='rect')).toEqual(['rect',0,0,750,1503])
  expect(c.calls.find(call=>call[0]==='text'&&call[1]==='北京')).toEqual(['text','北京'])
  expect(c.calls.find(call=>call[0]==='image'&&call[1]==='/code')).toEqual(['image','/code',593,1369,100,100])
  expect(c.depth()).toBe(0)
 }
})

test('both themes draw real progress without exposing algorithm or caption text',()=>{
 for(const theme of ['dark','light'] as const) {
  const c=canvas()
  const withProgress={...content,progress:{away:2/15,home:14/15,total:16/30}}
  drawRecapPoster(c.context,withProgress,'/code',new Map([[SEASON_RECAP_POSTER_BACKGROUNDS[theme],image('/background')]]),theme)
  expect(c.calls[0]).toEqual(['image','/background',0,0,750,1333])
  for(const barY of [859,973,1087]) expect(c.calls.some(call=>call[0]==='rect'&&call[2]===barY)).toBe(true)
  const texts=c.calls.filter(call=>call[0]==='text').map(call=>String(call[1]))
  expect(texts.some(value=>value.includes('已赴')||value.includes('加权')||value.includes('%'))).toBe(false)
  expect(c.depth()).toBe(0)
 }
})

test('background keeps its aspect ratio above a footer strip, bars fade over their filled length',()=>{
 const c=canvas()
 const withProgress={...content,progress:{away:0.25,home:null,total:0.5}}
 drawRecapPoster(c.context,withProgress,'/code',new Map([[SEASON_RECAP_POSTER_BACKGROUNDS.light,image('/background')]]),'light')
 expect(c.calls.filter(call=>call[0]==='rect').slice(0,2)).toEqual([['rect',0,1263,750,70],['rect',0,1333,750,170]])
 // 进度条按段铺色：x=73，轨道宽 604；第一、三条按进度结束，主场缺数据不画。
 const barEnd=(barY:number)=>Math.max(...c.calls.filter(call=>call[0]==='rect'&&call[2]===barY).map(call=>Number(call[1])+Number(call[3])))
 expect(Math.abs(barEnd(859)-(73+604*0.25-8))).toBeLessThan(6)
 expect(c.calls.some(call=>call[0]==='rect'&&call[2]===973)).toBe(false)
 expect(Math.abs(barEnd(1087)-(73+604*0.5-8))).toBeLessThan(6)
})

test('route chevrons are stroked instead of a glyph that some fonts render as a bracket',()=>{
 const c=canvas()
 const twoCities=buildRecapPosterContent({season:2026,matches:[
  {match_id:1,match_date:'2026-03-01',opponent:'重庆铜梁龙',opponent_avatar_url:null,is_home:false,city:'重庆'},
  {match_id:2,match_date:'2026-04-01',opponent:'武汉三镇',opponent_avatar_url:null,is_home:false,city:'武汉'},
 ],selected_match_ids:[1,2],home_one_way_km:null,summary:{away_km:2490,home_km:null,total_km:2490,matches:2,home_matches:0,away_matches:2,cities:2}})
 drawRecapPoster(c.context,twoCities,'/code')
 const texts=c.calls.filter(call=>call[0]==='text').map(call=>String(call[1]))
 expect(texts.includes('重庆')).toBe(true)
 expect(texts.includes('武汉')).toBe(true)
 expect(texts.some(value=>value.includes('›'))).toBe(false)
})
