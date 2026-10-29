// Deterministic camera-like fixtures, not a substitute for captured phone frames.
// Usage: node scripts/bench-scanner.mjs [path/to/nanocode.wasm]
import {readFile} from 'node:fs/promises';
import {loadNanoCode} from '../packages/js/index.js';
const binary=await readFile(process.argv[2]||'packages/js/nanocode.wasm');
const api=await loadNanoCode(binary);
const {instance:{exports:e}}=await WebAssembly.instantiate(binary,{});
const fixtures=[];
function code(format,dark=false,payload='ID000123'){
 const data=new TextEncoder().encode(payload),p=e.nc_alloc(data.length),q=e.nc_alloc(384*384);
 try{new Uint8Array(e.memory.buffer).set(data,p);e.nc_render_ex(p,data.length,format,384,0x00a6a1,0x77dddd,dark?0x101010:0xffffff,q,384*384);return new Uint8Array(e.memory.buffer).slice(q,q+384*384);}
 finally{e.nc_free(p,data.length);e.nc_free(q,384*384);}
}
function monitor(tile){
 const w=960,h=640,g=new Uint8Array(w*h).fill(44);
 for(let y=20;y<620;y++)for(let x=15;x<320;x++)g[y*w+x]=y%28<9&&x<280?190:31;
 for(let y=25;y<90;y++)for(let x=340;x<940;x++)g[y*w+x]=220;
 for(let y=0;y<384;y++)g.set(tile.subarray(y*384,(y+1)*384),(y+130)*w+445);
 return {gray:g,width:w,height:h};
}
function blur(g,w,h){
 return Uint8Array.from(g,(_,i)=>{
  const x=i%w,y=Math.floor(i/w);let sum=0,n=0;
  for(let dy=-1;dy<=1;dy++)for(let dx=-1;dx<=1;dx++){
   const xx=Math.max(0,Math.min(w-1,x+dx)),yy=Math.max(0,Math.min(h-1,y+dy));sum+=g[yy*w+xx];n++;
  }return Math.round(sum/n);
 });
}
for(const format of [1,2,3,4])for(const dark of [false,true]){
 const frame=monitor(code(format,dark));
 fixtures.push({name:`monitor-${format}-${dark?'dark':'light'}`,format,...frame});
}
const base=code(2);
fixtures.push({name:'close-up',format:2,gray:base,width:384,height:384});
// Exact N polygon from the landing page, rasterized independently of its SVG.
const logo=[[-.13,.12],[-.13,-.12],[-.075,-.12],[.075,.055],[.075,-.12],[.13,-.12],[.13,.12],[.075,.12],[-.075,-.055],[-.075,.12]];
function inside(x,y){let hit=false;for(let i=0,j=logo.length-1;i<logo.length;j=i++){
 const [a,b]=logo[i],[c,d]=logo[j];if((b>y)!==(d>y)&&x<(c-a)*(y-b)/(d-b)+a)hit=!hit;
}return hit;}
const branded=Uint8Array.from(base,(v,i)=>inside(2*(i%384)/384-1,2*Math.floor(i/384)/384-1)?((77*0x15+150*0x35+29*0x2f)>>8):v);
fixtures.push({name:'landing-N',format:2,gray:branded,width:384,height:384});
fixtures.push({name:'monitor-landing-N',format:2,...monitor(branded)});
for(const withLogo of [false,true])for(const [name,fn] of [
 ['exposure-gradient',(v,x,y)=>v*(.45+.55*x/383)],
 ['screen-banding',(v,x,y)=>v*(.72+.20*Math.sin(y/13))],
 ['moire',(v,x,y)=>v+18*Math.sin(x*2.3+y*.8)+12*Math.cos(y*2.1)],
 ['washed-out',(v)=>55+v*.65],
]){
 const gray=Uint8Array.from(withLogo?branded:base,(v,i)=>Math.max(0,Math.min(255,Math.round(fn(v,i%384,Math.floor(i/384))))));
 fixtures.push({name:(withLogo?'N-':'')+name,format:2,gray:blur(gray,384,384),width:384,height:384});
}
fixtures.push({name:'blank',gray:new Uint8Array(640*480).fill(128),width:640,height:480});
fixtures.push({name:'stripes-no-code',gray:Uint8Array.from({length:640*480},(_,i)=>[30,130,240][Math.floor(i%640/9)%3]),width:640,height:480});
api.scan(new Uint8Array(128*128).fill(255),128,128);
const results=[];
for(const f of fixtures){
 const times=[];let found;
 for(let n=0;n<3;n++){const t=performance.now();found=api.scan(f.gray,f.width,f.height);times.push(performance.now()-t);}
 times.sort((a,b)=>a-b);
 const valid=f.format?found?.text==='ID000123'&&found?.format===f.format:found===null;
 results.push({name:f.name,ok:valid,decoded:found?.text??null,medianMs:+times[1].toFixed(1),maxMs:+times[2].toFixed(1)});
}
console.log(JSON.stringify({runtime:process.version,platform:process.platform,arch:process.arch,fixtures:'synthetic',results},null,2));
