// Replay a private camera capture without adding it to the repository.
// Input is row-major 8-bit luminance, one byte per pixel, with no header.
// Usage: node scripts/test-camera-frame.mjs frame.gray width height expected-text
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {Worker} from 'node:worker_threads';
import {once} from 'node:events';
const [path,w,h,expected]=process.argv.slice(2),width=Number(w),height=Number(h);
assert(path&&expected!==undefined&&Number.isInteger(width)&&Number.isInteger(height),
 'Usage: node scripts/test-camera-frame.mjs frame.gray width height expected-text');
assert(width>=32&&height>=32&&width<=2048&&height<=2048,'Dimensions must be 32..2048');
const source=await readFile(path);
assert.equal(source.length,width*height,'Expected one luminance byte per pixel');
const url=new URL('../site/worker.js',import.meta.url).href;
const worker=new Worker(`const {parentPort}=require('node:worker_threads');
globalThis.onmessage=null;
globalThis.postMessage=(data,transfer)=>parentPort.postMessage(data,transfer);
globalThis.fetch=async url=>new Response(await require('node:fs/promises').readFile(url));
import(${JSON.stringify(url)}).then(()=>{parentPort.on('message',data=>onmessage({data}));parentPort.postMessage({ready:true});});`,{eval:true});
try{
 const [ready]=await once(worker,'message');assert(ready.ready);
 // The second read exercises the actual worker's tracked-region confirmation.
 for(let id=1;id<=3;id++){
  const gray=id===3?new Uint8Array(width*height).fill(255):Uint8Array.from(source);
  const next=once(worker,'message');worker.postMessage({gray:gray.buffer,width,height,id},[gray.buffer]);
  const [message]=await next;assert(!message.error,message.error);
  assert.equal(message.id,id);assert.equal(message.gray.byteLength,width*height);
  if(id===3)assert.equal(message.result,null,'A missing code must not return the tracked payload');
  else assert.equal(message.result?.text,expected,`Camera frame ${id} must decode`);
  console.log(JSON.stringify({frame:id,width,height,decoded:!!message.result,ms:+message.ms.toFixed(1)}));
 }
}finally{await worker.terminate();}
