import { expect, test } from 'bun:test'
import { CHENGDU_RONGCHENG_CREST_IMAGE_URL, SEASON_RECAP_POSTER_BG_IMAGE_URL } from '../../config/assets'
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
  createLinearGradient() { return {addColorStop(){}} },
  setFillStyle(){}, setStrokeStyle(){}, setLineWidth(){}, setTextAlign(){}, setGlobalAlpha(){},
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
  [SEASON_RECAP_POSTER_BG_IMAGE_URL,image('/bg')],
  [CHENGDU_RONGCHENG_CREST_IMAGE_URL,image('/chengdu')],
  ['https://crest',image('/beijing')],
 ]))
 expect(c.calls[0]).toEqual(['image','/bg',0,0,750,1334])
 expect(c.calls.filter(call=>call[0]==='image').map(call=>call[1])).toEqual(['/bg','/beijing','/code'])
 expect(c.calls.filter(call=>call[0]==='clip').length).toBe(1)
 const texts=c.calls.filter(call=>call[0]==='text').map(call=>call[1])
 expect(texts.filter(value=>['3000','北京','主场里程','蓉城同行者','自报观赛 · 客场为城市间直线往返估算'].includes(String(value)))).toEqual([
  '3000','蓉城同行者','主场里程','北京','自报观赛 · 客场为城市间直线往返估算',
 ])
 expect(c.depth()).toBe(0)
})

test('missing decorations or image decode errors still render city, stats and code',()=>{
 for(const images of [new Map<string,RecapPosterImage>(),new Map([
  [SEASON_RECAP_POSTER_BG_IMAGE_URL,image('/bad-bg')],
  [CHENGDU_RONGCHENG_CREST_IMAGE_URL,image('/bad-chengdu')],
  ['https://crest',image('/bad-opponent')],
 ])]) {
  const c=canvas(true)
  drawRecapPoster(c.context,content,'/code',images)
  expect(c.calls.find(call=>call[0]==='rect')).toEqual(['rect',0,0,750,1334])
  expect(c.calls.find(call=>call[0]==='text'&&call[1]==='北京')).toEqual(['text','北京'])
  expect(c.calls.find(call=>call[0]==='image'&&call[1]==='/code')).toEqual(['image','/code',536,1124,144,144])
  expect(c.depth()).toBe(0)
 }
})
