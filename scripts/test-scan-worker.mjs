import assert from 'node:assert/strict';
import {Worker} from 'node:worker_threads';
import {readFile} from 'node:fs/promises';
import {once} from 'node:events';
const {instance:{exports:e}}=await WebAssembly.instantiate(await readFile('packages/js/nanocode.wasm'),{});
const size=384,p=e.nc_alloc(8),img=e.nc_alloc(size*size);
let code;
try{
 new Uint8Array(e.memory.buffer).set(new TextEncoder().encode('ID000123'),p);
 assert.equal(e.nc_render_ex(p,8,2,size,0x00a6a1,0x77dddd,0xffffff,img,size*size),size*size);
 code=new Uint8Array(e.memory.buffer).slice(img,img+size*size);
}finally{e.nc_free(p,8);e.nc_free(img,size*size);}
// Exercise the actual browser worker in Node with only its transport/fetch adapted.
const url=new URL('../site/worker.js',import.meta.url).href;
const bootstrap=`const {parentPort}=require('node:worker_threads');
globalThis.onmessage=null;
globalThis.postMessage=(data,transfer)=>parentPort.postMessage(data,transfer);
globalThis.fetch=async url=>new Response(await require('node:fs/promises').readFile(url));
import(${JSON.stringify(url)}).then(()=>{parentPort.on('message',data=>onmessage({data}));parentPort.postMessage({ready:true});});`;
const worker=new Worker(bootstrap,{eval:true});
try{
 const [ready]=await once(worker,'message');assert(ready.ready);
 for(let id=1;id<=3;id++){
  const width=640,height=480,gray=new Uint8Array(width*height).fill(255);
  if(id!==3)for(let y=0;y<size;y++)gray.set(code.subarray(y*size,(y+1)*size),(y+40)*width+100);
  const next=once(worker,'message');worker.postMessage({gray:gray.buffer,width,height,id},[gray.buffer]);
  assert.equal(gray.byteLength,0);
  const [message]=await next;
  assert.equal(message.id,id);assert(!message.error,message.error);
  assert.equal(message.gray.byteLength,width*height);
  if(id===3)assert.equal(message.result,null,'Tracking must never repeat a stale payload');
  else {assert.equal(message.result.text,'ID000123');assert.equal(message.result.format,2);assert(message.result.corners.every((v,i)=>v>(i%2?40:100)));}
 }
 console.log('Worker: decoding, tracked crop coordinates, transferred buffer reuse and lost-code rejection passed');
}finally{await worker.terminate();}
