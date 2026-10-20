import { expect, test } from 'bun:test'
import { buildRecapPosterContent, recapBadge, recapConversion, recapRoute, saveRecapPoster } from './poster'
import type { RecapMatch, SeasonRecap } from '../../types/seasonRecap'
const zero={away_km:0,home_km:null,total_km:0,matches:0,home_matches:0,away_matches:0,cities:0}
function match(id:number,is_home:boolean,city?:string):RecapMatch {
 return {match_id:id,match_date:`2026-03-${String(id).padStart(2,'0')}`,opponent:`对手${id}`,is_home,...(city?{city}:{})}
}
test('conversion switches from marathon to equator at one tenth of a lap and hides zero mileage',()=>{
 expect(recapConversion(0)).toBe(null)
 expect(recapConversion(-5)).toBe(null)
 expect(recapConversion(Number.NaN)).toBe(null)
 expect(recapConversion(21)).toBe('相当于跑了 1 个马拉松')
 expect(recapConversion(100)).toBe('相当于跑了 2 个马拉松')
 expect(recapConversion(4007.4)).toBe('相当于跑了 95 个马拉松')
 expect(recapConversion(4007.5)).toBe('相当于绕赤道 0.10 圈')
 expect(recapConversion(8015)).toBe('相当于绕赤道 0.20 圈')
})
test('badge tiers prefer expedition over attendance and stay dignified',()=>{
 expect(recapBadge({...zero,matches:20,away_matches:8})).toBe('铁血远征军')
 expect(recapBadge({...zero,matches:16,away_matches:3})).toBe('远征先锋')
 expect(recapBadge({...zero,matches:15,away_matches:2})).toBe('全勤死忠')
 expect(recapBadge({...zero,matches:5,away_matches:0})).toBe('主场守护者')
 expect(recapBadge({...zero,matches:2,away_matches:1})).toBe('蓉城同行者')
 expect(recapBadge(zero)).toBe('蓉城同行者')
})
test('route dedupes away cities in match date order and caps with 等 N 城',()=>{
 const matches=[match(3,true,'成都'),match(1,false,'北京'),match(2,false,'上海'),match(4,false,'北京')]
 expect(recapRoute(matches,[1,2,3,4])).toBe('成都 → 北京 → 上海')
 expect(recapRoute(matches,[3])).toBe(null)
 expect(recapRoute(matches,[])).toBe(null)
 const far=Array.from({length:7},(_,index)=>match(index+1,false,`城${index+1}`))
 expect(recapRoute(far,far.map(m=>m.match_id))).toBe('成都 → 城1 → 城2 → 城3 → 城4 → 城5 → 城6 等 7 城')
 const six=Array.from({length:6},(_,index)=>match(index+1,false,`城${index+1}`))
 expect(recapRoute(six,six.map(m=>m.match_id))).toBe('成都 → 城1 → 城2 → 城3 → 城4 → 城5 → 城6')
})
test('route hides when a selected away match has no city from a legacy backend',()=>{
 const matches=[match(1,false),match(2,false,'上海')]
 expect(recapRoute(matches,[1,2])).toBe(null)
 expect(recapRoute(matches,[2])).toBe('成都 → 上海')
})
test('poster content composes backend summary without recomputing mileage',()=>{
 const view:SeasonRecap={season:2026,matches:[match(1,true,'成都'),match(2,false,'北京')],selected_match_ids:[1,2],home_one_way_km:12.25,summary:{away_km:3016,home_km:25,total_km:3041,matches:2,home_matches:1,away_matches:1,cities:2}}
 const content=buildRecapPosterContent(view)
 expect(content.kicker).toBe('成都蓉城 · 2026 赛季')
 expect(content.totalKm).toBe(3041)
 expect(content.homeKm).toBe(25)
 expect(content.conversion).toBe('相当于跑了 72 个马拉松')
 expect(content.badge).toBe('蓉城同行者')
 expect(content.route).toBe('成都 → 北京')
})
test('album permission denial offers settings and does not pretend saving succeeded',async()=>{
 let offered=false,opened=false,saved=false
 const platform={getSetting:(o:any)=>o.success({authSetting:{'scope.writePhotosAlbum':false}}),showModal:(o:any)=>{offered=true;o.success({confirm:true})},openSetting:()=>{opened=true},saveImageToPhotosAlbum:()=>{saved=true},showToast:()=>{}}
 await saveRecapPoster('poster.png',platform as any)
 expect(offered).toBe(true);expect(opened).toBe(true);expect(saved).toBe(false)
})
test('album saving failure is visible and success alone shows saved',async()=>{
 const messages:string[]=[]
 const platform={getSetting:(o:any)=>o.success({authSetting:{}}),showModal:()=>{},openSetting:()=>{},saveImageToPhotosAlbum:(o:any)=>o.fail({errMsg:'arbitrary failure'}),showToast:(o:any)=>messages.push(o.title)}
 await saveRecapPoster('poster.png',platform as any);expect(messages).toEqual(['保存失败，请重试'])
 platform.saveImageToPhotosAlbum=(o:any)=>o.success({});await saveRecapPoster('poster.png',platform as any);expect(messages.at(-1)).toBe('已保存到相册')
})
