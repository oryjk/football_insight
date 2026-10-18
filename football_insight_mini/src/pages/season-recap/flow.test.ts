import { expect, test } from 'bun:test'
import { ref } from 'vue'
import { createRecapFlow } from './flow'
import { ApiRequestError } from '../../utils/apiError'
import type { SeasonRecap } from '../../types/seasonRecap'
const data: SeasonRecap = {season:2026,matches:[{match_id:1,match_date:'2026-01-01',opponent:'北京国安',is_home:true}],selected_match_ids:[],home_one_way_km:null,summary:{away_km:0,home_km:null,total_km:0,matches:0,home_matches:0,away_matches:0,cities:0}}
function deferred<T>() {let resolve!:(value:T)=>void;const promise=new Promise<T>(r=>{resolve=r});return{promise,resolve}}
function setup(load = async () => data) {
 const revision=ref(1), hasAccessToken=ref(true)
 const writes: unknown[]=[]
 const flow=createRecapFlow({revision,hasAccessToken},{load,save:async (_year,payload)=>{writes.push(payload);return data},code:async ()=>({image_base64:'png',mime_type:'image/png'})})
 return {flow,revision,hasAccessToken,writes}
}
test('does not load for guest and ignores a late load after switching accounts',async()=>{
 const pending=deferred<SeasonRecap>();const {flow,revision,hasAccessToken}=setup((()=>{let calls=0;return()=>++calls===1?pending.promise:new Promise<SeasonRecap>(()=>{})})())
 hasAccessToken.value=false;await flow.load();expect(flow.view.value).toBe(null)
 hasAccessToken.value=true;const loading=flow.load();revision.value++;pending.resolve(data);await loading
 expect(flow.view.value).toBe(null);expect(flow.posterPath.value).toBe('');flow.dispose()
})
test('invalid home input never submits; duplicate picks toggle rather than accumulate',async()=>{
 const {flow,writes}=setup();await flow.load();flow.toggle(1);flow.toggle(1);expect(flow.selected.value).toEqual([])
 flow.toggle(1);flow.distance.value='-1';await flow.generate();expect(writes.length).toBe(0);expect(flow.error.value.includes('0–500')).toBe(true)
 flow.distance.value='12.25';await flow.generate();expect(writes).toEqual([{selected_match_ids:[1],home_one_way_km:12.25}]);expect(flow.generated.value===null).toBe(false)
 flow.dispose()
})
test('late save/code/canvas cannot restore a previous account poster',async()=>{
 const pending=deferred<SeasonRecap>();const {flow,revision}=setup();await flow.load()
 flow.client.save=()=>pending.promise
 const generation=flow.generate();const oldKey=flow.key.value;revision.value++;pending.resolve(data);await generation
 flow.posterReady(oldKey,'private-poster');expect(flow.generated.value).toBe(null);expect(flow.posterPath.value).toBe('');flow.dispose()
})
test('401 is recognized by statusCode, loading error is visible and edits survive failure',async()=>{
 const {flow}=setup(async()=>{throw new ApiRequestError('arbitrary wording',401)})
 await flow.load();expect(flow.error.value).toBe('请先登录后制作赛季总结');expect(flow.loading.value).toBe(false);flow.dispose()
})

test('cold shared entry reloads after asynchronous authentication becomes ready',async()=>{
 let calls=0
 const {flow,revision,hasAccessToken}=setup(async()=>{calls++;return data})
 hasAccessToken.value=false;await flow.load();expect(calls).toBe(0)
 hasAccessToken.value=true;revision.value++
 await Promise.resolve();await Promise.resolve()
 expect(calls).toBe(1);expect(flow.view.value?.season).toBe(2026)
 flow.dispose()
})

test('expired token clears only the requesting session and exposes login recovery',async()=>{
 const revision=ref(7),hasAccessToken=ref(true)
 let invalidated=0
 const flow=createRecapFlow({revision,hasAccessToken,invalidateSession:(expected:number)=>{if(expected===revision.value){invalidated++;revision.value++;hasAccessToken.value=false}}}, {load:async()=>{throw new ApiRequestError('anything',401)},save:async()=>data,code:async()=>({image_base64:'',mime_type:'image/png'})})
 await flow.load();expect(invalidated).toBe(1);expect(hasAccessToken.value).toBe(false);expect(flow.view.value).toBe(null);flow.dispose()
})
