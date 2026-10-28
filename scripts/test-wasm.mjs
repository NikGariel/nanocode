import {readFile} from 'node:fs/promises';
import assert from 'node:assert/strict';
const {instance:{exports:e}}=await WebAssembly.instantiate(await readFile('packages/js/nanocode.wasm'),{});
const payload=new TextEncoder().encode('00Привет');
const p=e.nc_alloc(payload.length),img=e.nc_alloc(512*512),out=e.nc_alloc(24),corners=e.nc_alloc(64);
try {
 new Uint8Array(e.memory.buffer).set(payload,p);
 assert.equal(e.nc_render(p,payload.length,512,img,512*512),512*512);
 assert.equal(e.nc_scan(img,512,512,out,24,corners),payload.length);
 assert.deepEqual(new Uint8Array(e.memory.buffer).slice(out,out+payload.length),payload);
 assert.equal(e.nc_scan(img,0,512,out,24,corners),-1);
 console.log('WASM ABI: UTF-8 image roundtrip passed');
}finally{e.nc_free(p,payload.length);e.nc_free(img,512*512);e.nc_free(out,24);e.nc_free(corners,64);}
// v2: auto format, metadata, polarity and capacity guard across the actual WASM ABI.
for(const [length,id] of [[8,1],[16,2],[24,3],[40,4]]) {
 const data=new Uint8Array(length).fill(65),p=e.nc_alloc(length),img=e.nc_alloc(512*512),out=e.nc_alloc(40),meta=e.nc_alloc(8);
 try {
  new Uint8Array(e.memory.buffer).set(data,p);
  assert.equal(e.nc_format_for_len(length),id);
  assert.equal(e.nc_palette_flags(0xdddddd,0x777777,0x101010),0);
  assert.equal(e.nc_render_ex(p,length,0,512,0xdddddd,0x777777,0x101010,img,512*512),512*512);
  new Uint8Array(e.memory.buffer).fill(0xaa,out,out+40);
  assert.equal(e.nc_scan_ex(img,512,512,out,length-1,0,meta),-4);
  assert(new Uint8Array(e.memory.buffer).slice(out,out+40).every(x=>x===0xaa));
  assert.equal(e.nc_scan_ex(img,512,512,out,40,0,meta),length);
  assert.deepEqual(new Uint8Array(e.memory.buffer).slice(out,out+length),data);
  assert.equal(new Uint32Array(e.memory.buffer,meta,2)[0],id);
  assert.equal(e.nc_svg_ex(p,length,0,0x111111,0x121212,0xffffff,0,0),-5);
 }finally{e.nc_free(p,length);e.nc_free(img,512*512);e.nc_free(out,40);e.nc_free(meta,8);}
}
console.log('WASM v2: S/M/L/XL, dark background, metadata, palette rejection and capacity guard passed');
// Exercise the public JS adapter, including background recommendations.
const {loadNanoCode}=await import('../packages/js/index.js');
const wasm=await readFile('packages/js/nanocode.wasm');
const engine=await loadNanoCode('data:application/wasm;base64,'+wasm.toString('base64'));
assert.equal(engine.formatFor('1234567890123456').name,'M');
assert.equal(engine.formatFor('Я'.repeat(16)).name,'XL');
assert(engine.svg('small').includes('#00a6a1'));
assert(engine.backgrounds('#111111','#121212').length===0);
assert.throws(()=>engine.svg('x',{one:'#111111',zero:'#121212'}));
assert(engine.backgrounds('#00a6a1','#77dddd').every(p=>engine.assess({background:p.color}).allowed));
console.log('JavaScript API: auto size, UTF-8 byte capacity, palette suggestions passed');
// The scanner receives the whole monitor view, not a pre-cropped code image.
{
 const payload=new TextEncoder().encode('ID000123'),p=e.nc_alloc(payload.length),pixels=e.nc_alloc(384*384);
 try {
  new Uint8Array(e.memory.buffer).set(payload,p);
  assert.equal(e.nc_render_ex(p,payload.length,2,384,0x00a6a1,0x77dddd,0x101010,pixels,384*384),384*384);
  const code=new Uint8Array(e.memory.buffer).slice(pixels,pixels+384*384),w=960,h=640,scene=new Uint8Array(w*h).fill(44);
  for(let y=20;y<620;y++)for(let x=15;x<320;x++)scene[y*w+x]=y%28<9&&x<280?190:31;
  for(let y=25;y<90;y++)for(let x=340;x<940;x++)scene[y*w+x]=220;
  for(let y=0;y<384;y++)scene.set(code.subarray(y*384,(y+1)*384),(y+130)*w+445);
  const begin=performance.now(),result=engine.scan(scene,w,h);
  assert.deepEqual(result.bytes,payload);assert.equal(result.format,2);
  assert(result.corners[0]>445&&result.corners[1]>130);
  console.log(`WASM monitor scene: located and decoded in ${(performance.now()-begin).toFixed(0)} ms (synthetic, this machine)`);
 }finally{e.nc_free(p,payload.length);e.nc_free(pixels,384*384);}
}
