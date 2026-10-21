import { expect, test } from 'bun:test'
import { buildRecapPosterContent, recapBadge, recapProgress, recapConversion, recapRoute, recapRouteEntries, saveRecapPoster, supportsRecapWebp } from './poster'
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
const totals={home_matches:15,away_matches:15,completed_matches:28}
const attendance=(home:number,away:number)=>({...zero,home_matches:home,away_matches:away,matches:home+away})
test('weighted tiers give away matches twice the weight without rounding thresholds',()=>{
 expect(recapBadge(attendance(1,0),totals)).toBe('蓉城同行者')
 expect(recapBadge(attendance(0,6),totals)).toBe('主场死忠')
 expect(recapBadge(attendance(0,12),totals)).toBe('蓉城铁杆')
 expect(recapBadge(attendance(10,12),totals)).toBe('铁血死忠')
 expect(recapBadge(attendance(15,14),totals)).toBe('铁血死忠')
 const sixteen={home_matches:8,away_matches:8,completed_matches:16}
 expect(recapBadge(attendance(4,1),sixteen)).toBe('主场死忠') // exact 25%
 expect(recapBadge(attendance(4,4),sixteen)).toBe('蓉城铁杆') // exact 50%
 expect(recapBadge(attendance(4,7),sixteen)).toBe('铁血死忠') // exact 75%
 expect(recapBadge(attendance(14,2),totals)).toBe('主场死忠')
})
test('full attendance requires every home and away match and the complete schedule finished',()=>{
 expect(recapBadge(attendance(15,15),{...totals,completed_matches:30})).toBe('全勤死忠')
 expect(recapBadge(attendance(15,15),totals)).toBe('铁血死忠')
 expect(recapBadge(attendance(15,14),{...totals,completed_matches:30})).toBe('铁血死忠')
 expect(recapBadge(attendance(0,0),{home_matches:0,away_matches:0,completed_matches:0})).toBe('尚未启程')
 expect(recapBadge(attendance(15,15))).toBe('蓉城同行者')
})
test('home specialist starts at ten home matches and no away attendance',()=>{
 expect(recapBadge(attendance(9,0),totals)).toBe('蓉城同行者')
 expect(recapBadge(attendance(10,0),totals)).toBe('主场守护者')
 expect(recapBadge(attendance(15,0),totals)).toBe('主场守护者')
 expect(recapBadge(attendance(10,1),totals)).toBe('主场死忠')
})
test('unweighted progress uses full schedule, tolerates zero totals and missing legacy metadata',()=>{
 expect(recapProgress(attendance(14,2),totals)).toEqual({home:14/15,away:2/15,total:16/30})
 expect(recapProgress(zero,{home_matches:0,away_matches:0,completed_matches:0})).toEqual({home:0,away:0,total:0})
 expect(recapProgress(zero)).toEqual({home:null,away:null,total:null})
 expect(recapProgress(attendance(20,20),totals)).toEqual({home:1,away:1,total:1})
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
test('route entries order by date, aggregate unique same-city crests and tolerate missing avatars',()=>{
 const matches=[
  {...match(4,false,' 上海 '),opponent_avatar_url:' https://cdn/shenhua.png '},
  {...match(3,false,'上海'),opponent_avatar_url:'https://cdn/haigang.png'},
  {...match(2,false,'北京'),opponent_avatar_url:null},
  {...match(1,true,'成都'),opponent_avatar_url:'https://cdn/unused.png'},
  {...match(5,false,'上海'),opponent_avatar_url:'https://cdn/shenhua.png'},
  {...match(6,false,'天津'),opponent_avatar_url:'https://cdn/unselected.png'},
 ]
 expect(recapRouteEntries(matches,[5,4,3,2,1])).toEqual({entries:[
  {city:'北京',avatarUrls:[]},
  {city:'上海',avatarUrls:['https://cdn/haigang.png','https://cdn/shenhua.png']},
 ],totalCities:2})
 expect(recapRouteEntries(matches,[1])).toBeNull()
 expect(matches.map(m=>m.match_id)).toEqual([4,3,2,1,5,6])
 const far=Array.from({length:8},(_,i)=>match(i+1,false,`城${i+1}`))
 expect(recapRouteEntries(far,far.map(m=>m.match_id))).toEqual({entries:far.slice(0,6).map(m=>({city:m.city!,avatarUrls:[]})),totalCities:8})
})
test('webp is conservative on unknown platforms and older iOS',()=>{
 expect(supportsRecapWebp('android','Android 13')).toBe(true)
 expect(supportsRecapWebp('ios','iOS 14.0')).toBe(true)
 expect(supportsRecapWebp('ios','iOS 13.7')).toBe(false)
 expect(supportsRecapWebp('devtools','Windows 10')).toBe(false)
 expect(supportsRecapWebp('ios','unknown')).toBe(false)
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
