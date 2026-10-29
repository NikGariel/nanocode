import {loadNanoCode} from '../packages/js/index.js';
import {ui} from './content/ui.mjs';
import {containedFrame,projectedRing} from './scan-geometry.js';
import {scanDelay,scanSize} from './scan-policy.js';
import {locales} from './content/locales.mjs';
const lang=document.documentElement.lang;
const t=ui[lang]||ui.en, content=locales[lang]||locales.en;
let logoDataUrl;
const $=id=>document.getElementById(id);
let api,svg='',stream=null,worker=null,busy=false,sequence=0,stable=0,last='',timer=null;
let cameraRequest=0,deviceListRequest=0,cameraOpening=false;
let readHistory=[],lastGood=null,lastGoodAt=0;
let frameCallback=null,lastVideoTime=-1,scanStarted=0,workStarted=0,grayBuffer=null,attempt=0,idleTimer=null;
const capture=document.createElement('canvas');const ctx=capture.getContext('2d',{willReadFrequently:true});
function error(e){$('error').textContent=e.message||String(e);}
function generate(){
  const palette={one:$('one').value,zero:$('zero').value,background:$('background').value};
  try{
    const assessment=api.assess(palette);
    $('palette-status').dataset.bad=String(!assessment.allowed);
    $('palette-status').textContent=assessment.allowed?(assessment.recommended?t.paletteGood:t.paletteLow):t.paletteBad+': '+assessment.issues.join('. ');
    $('backgrounds').replaceChildren();
    for(const item of api.backgrounds(palette.one,palette.zero)){const button=document.createElement('button');button.style.background=item.color;button.title=`${item.color} · ${item.score}/255`;button.setAttribute('aria-label',`${t.swatch} ${item.color}`);button.onclick=()=>{$('background').value=item.color;generate();};$('backgrounds').append(button);}
    if(!$('backgrounds').children.length)$('backgrounds').textContent=t.paletteBad;
    const format=Number($('format').value),f=api.formatFor($('payload').value,format);
    $('count').textContent=`${f.name} · ${f.rings} ${content.labels[14]} · ${new TextEncoder().encode($('payload').value).length} / ${f.capacity} ${content.labels[15]}`;
    svg=api.svg($('payload').value,{format,...palette,logoDataUrl});$('code').innerHTML=svg;$('error').textContent='';$('download').disabled=false;$('selftest').disabled=false;
  }catch(e){svg='';$('code').replaceChildren();$('download').disabled=true;$('selftest').disabled=true;error(e);}
}
function luminance(image,w,h){if(capture.width!==w)capture.width=w;if(capture.height!==h)capture.height=h;ctx.drawImage(image,0,0,w,h);const rgba=ctx.getImageData(0,0,w,h).data;const gray=grayBuffer?.byteLength===w*h?new Uint8Array(grayBuffer):new Uint8Array(w*h);grayBuffer=null;for(let i=0;i<gray.length;i++)gray[i]=(77*rgba[4*i]+150*rgba[4*i+1]+29*rgba[4*i+2])>>8;return gray;}
function draw(result,w,h){
 const canvas=$('overlay');canvas.width=w;canvas.height=h;
 const box=containedFrame(w,h);
 for(const key of ['width','height','left','top'])canvas.style[key]=box[key]+'%';
 const guide=$('stage').querySelector('.reticle');
 guide.style.width=(Math.min(box.width,box.height)*.72)+'%';
 const pen=canvas.getContext('2d');if(!result)return;
 const points=projectedRing(result.corners);if(!points.length)return;
 pen.strokeStyle='#caf39a';pen.lineWidth=Math.max(2,w/260);pen.lineJoin='round';
 pen.shadowColor='#caf39a88';pen.shadowBlur=12;
 pen.beginPath();points.forEach(([x,y],i)=>i===0?pen.moveTo(x,y):pen.lineTo(x,y));pen.closePath();pen.stroke();
}
function show(result,w,h,confirmed=true){draw(result,w,h);$('stage').classList.toggle('found',!!result&&confirmed);if(result&&confirmed)experience(result);$('result').textContent=result?(confirmed?`${t.read}: ${result.text || t.empty} · ${result.formatName}`:t.confirm):t.search;}
function stop(){
  ++cameraRequest;
  cameraOpening=false;
  clearTimeout(timer);
  clearTimeout(idleTimer);
  if(frameCallback!==null)$('video').cancelVideoFrameCallback?.(frameCallback);
  frameCallback=null;lastVideoTime=-1;grayBuffer=null;attempt=0;
  stream?.getTracks().forEach(track=>track.stop());
  stream=null;
  $('save-frame').disabled=true;
  $('video').srcObject=null;
  worker?.terminate();worker=null;
  busy=false;stable=0;last='';readHistory=[];lastGood=null;lastGoodAt=0;
  $('camera').textContent=t.start;
  $('camera-device').disabled=false;
  $('stage').classList.remove('scanning','found');
  $('timing').textContent='';
  draw(null,640,640);$('experience')?.replaceChildren();
}
async function refreshCameras(){
  if(!navigator.mediaDevices?.enumerateDevices){
    $('camera-hint').textContent=t.secure;
    return;
  }
  const request=++deviceListRequest;
  try{
    const devices=await navigator.mediaDevices.enumerateDevices();
    if(request!==deviceListRequest)return;
    const cameras=devices.filter(device=>device.kind==='videoinput');
    const selected=$('camera-device').value;
    const active=stream?.getVideoTracks()[0]?.getSettings().deviceId;
    const options=[new Option(t.cameraAuto,'')];
    cameras.forEach((device,index)=>{
      if(device.deviceId)options.push(new Option(device.label||`${t.camera} ${index+1}`,device.deviceId));
    });
    $('camera-device').replaceChildren(...options);
    const preferred=active||selected;
    $('camera-device').value=cameras.some(device=>device.deviceId===preferred)?preferred:'';
    $('camera-hint').textContent=cameras.some(device=>device.label)
      ?t.cameraReady
      :t.cameraPermission;
    if(active&&!cameras.some(device=>device.deviceId===active)){
      stop();
      $('result').textContent=t.noCamera;
    }
  }catch{
    if(request===deviceListRequest)$('camera-hint').textContent=t.noCamera;
  }
}
async function start(){
  if(stream||cameraOpening){stop();$('result').textContent=t.stopped;return;}
  const request=++cameraRequest;
  cameraOpening=true;
  $('camera').textContent=t.cancel;
  $('camera-device').disabled=true;
  $('error').textContent='';
  try{
    if(!navigator.mediaDevices?.getUserMedia)throw new Error(t.secure);
    const deviceId=$('camera-device').value;
    const video={width:{ideal:1280},height:{ideal:720},frameRate:{ideal:15,max:24},...(deviceId?{deviceId:{exact:deviceId}}:{facingMode:{ideal:'environment'}})};
    const acquired=await navigator.mediaDevices.getUserMedia({video,audio:false});
    // Permission prompts can outlive a cancellation or a tab being hidden.
    if(request!==cameraRequest){acquired.getTracks().forEach(track=>track.stop());return;}
    stream=acquired;
    // Optional camera controls must never prevent an otherwise supported stream.
    const cameraTrack=acquired.getVideoTracks()[0],caps=cameraTrack.getCapabilities?.()||{};
    const controls={};
    if(caps.focusMode?.includes('continuous'))controls.focusMode='continuous';
    if(caps.exposureMode?.includes('continuous'))controls.exposureMode='continuous';
    if(Object.keys(controls).length)void cameraTrack.applyConstraints({advanced:[controls]}).catch(()=>{});
    $('save-frame').disabled=false;
    $('video').srcObject=acquired;
    await $('video').play();
    if(request!==cameraRequest)return;
    cameraOpening=false;
    $('camera-device').disabled=false;
    $('camera').textContent=t.stop;
    $('stage').classList.add('scanning');
    draw(null,$('video').videoWidth,$('video').videoHeight);
    $('result').textContent=t.search;
    for(const track of acquired.getVideoTracks())track.addEventListener('ended',()=>{
      if(request!==cameraRequest)return;
      stop();$('result').textContent=t.noCamera;
      void refreshCameras();
    });
    worker=new Worker(new URL('./worker.js'+new URL(import.meta.url).search,import.meta.url),{type:'module'});
    worker.onmessage=({data})=>{
      if(request!==cameraRequest)return;
      busy=false;
      grayBuffer=data.gray;
      if(data.error){error(data.error);stop();return;}
      const now=performance.now();
      const key=data.result?JSON.stringify([data.result.format,...data.result.bytes]):null;
      readHistory.push({key,time:now});
      readHistory=readHistory.filter(read=>now-read.time<2500).slice(-6);
      stable=key===null?0:readHistory.filter(read=>read.key===key).length;
      if(data.result&&stable>=2){lastGood=data.result;lastGoodAt=now;}
      if(!data.result&&lastGood&&now-lastGoodAt<2000){
        // Keep a verified result briefly, but never draw stale pose coordinates on a new frame.
        draw(null,capture.width,capture.height);
        $('stage').classList.remove('found');
        $('result').textContent=`${t.read}: ${lastGood.text||t.empty} · ${lastGood.formatName}`;
      }else show(data.result,capture.width,capture.height,stable>=2);
      $('timing').textContent=`${capture.width}×${capture.height} · ${data.ms.toFixed(0)} ms · ${t.confirmation} ${Math.min(stable,2)}/2`;
      if(data.result&&stable>=2){
        // A demo scan is a one-shot action. Release the camera and worker after
        // confirmation, while retaining the exact processed frame and its pose.
        const result=data.result,w=capture.width,h=capture.height,timing=$('timing').textContent;
        stop();show(result,w,h,true);
        const pen=$('overlay').getContext('2d');pen.globalCompositeOperation='destination-over';pen.drawImage(capture,0,0);pen.globalCompositeOperation='source-over';
        $('timing').textContent=timing;
        $('camera').textContent=t.scanAgain;
        return;
      }
      scheduleScan(scanDelay(now-workStarted,now-scanStarted));
    };
    worker.onerror=event=>{if(request===cameraRequest){error(event.message);stop();}};
    scanStarted=performance.now();scheduleScan(0);
    idleTimer=setTimeout(()=>{if(request===cameraRequest){stop();$('result').textContent=t.scanPaused;}},60000);
    void refreshCameras();
  }catch(e){
    if(request!==cameraRequest)return;
    stop();
    const messages={NotAllowedError:t.denied,NotFoundError:t.noCamera,OverconstrainedError:t.noCamera,NotReadableError:t.noCamera};
    error(messages[e.name]||e);
    $('result').textContent=t.failed;
    void refreshCameras();
  }
}
$('camera-device').addEventListener('change',()=>{
  if(stream||cameraOpening){stop();void start();}
});
navigator.mediaDevices?.addEventListener('devicechange',()=>void refreshCameras());
function scheduleScan(delay){
  if(!stream)return;
  timer=setTimeout(()=>{
    if(!stream)return;
    const v=$('video');
    if(v.requestVideoFrameCallback)frameCallback=v.requestVideoFrameCallback(()=>{frameCallback=null;tick();});
    else tick();
  },delay);
}
function tick(){
  if(!stream||busy)return;
  const v=$('video');
  if(!v.videoWidth||v.readyState<2||v.currentTime===lastVideoTime){scheduleScan(90);return;}
  lastVideoTime=v.currentTime;workStarted=performance.now();
  const {width:w,height:h}=scanSize(v.videoWidth,v.videoHeight,attempt++,stable>0);
  const gray=luminance(v,w,h);busy=true;
  worker.postMessage({gray:gray.buffer,width:w,height:h,id:++sequence},[gray.buffer]);
}
async function scanBlob(blob){stop();const url=URL.createObjectURL(blob);try{const img=new Image();img.src=url;await img.decode();const scale=Math.min(1,1000/Math.max(img.width,img.height));const w=Math.round(img.width*scale),h=Math.round(img.height*scale);const gray=luminance(img,w,h);const result=api.scan(gray,w,h);show(result,w,h);const pen=$('overlay').getContext('2d');pen.globalCompositeOperation='destination-over';pen.drawImage(img,0,0,w,h);pen.globalCompositeOperation='source-over';if(!result)$('result').textContent=t.notFound;}finally{URL.revokeObjectURL(url);}}
for(const id of ['payload','format','one','zero','background'])$(id).addEventListener('input',()=>api&&generate());$('camera').onclick=start;
$('download').onclick=()=>{const url=URL.createObjectURL(new Blob([svg],{type:'image/svg+xml'}));const a=document.createElement('a');a.href=url;a.download='nanocode.svg';a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);};
$('selftest').onclick=()=>scanBlob(new Blob([svg],{type:'image/svg+xml'})).catch(error);
$('image').onchange=e=>{if(api&&e.target.files[0])scanBlob(e.target.files[0]).catch(error);};
window.addEventListener('pagehide',stop);document.addEventListener('visibilitychange',()=>{if(document.hidden&&(stream||cameraOpening))stop();});
try{api=await loadNanoCode();generate();$('camera').disabled=false;$('result').textContent=t.ready;}catch(e){error(e);$('result').textContent=t.failed;}

void refreshCameras();

$('save-frame').onclick=()=>{
  if(!stream||!$('video').videoWidth)return;
  const frame=document.createElement('canvas'),v=$('video');
  frame.width=v.videoWidth;frame.height=v.videoHeight;
  frame.getContext('2d').drawImage(v,0,0);
  frame.toBlob(blob=>{if(!blob)return;const url=URL.createObjectURL(blob),a=document.createElement('a');a.href=url;a.download='nanocode-camera-frame.png';a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);},'image/png');
};

function experience(result){
  const box=$('experience');if(!box)return;
  const index=['ORBIT:EVT:042','ATELIER:DOOR:07','LOOP:RIDE:024'].indexOf(result.text);
  box.replaceChildren();if(index<0)return;
  const title=document.createElement('h3'),body=document.createElement('p'),note=document.createElement('small');
  title.textContent=content.scenes[index][0];body.textContent=content.scenes[index][1];note.textContent=content.labels[10];
  box.append(title,body,note);
}
$('logo-file')?.addEventListener('change',async e=>{
 const file=e.target.files[0];if(!file)return;
 try{
  if(!['image/png','image/jpeg','image/webp'].includes(file.type)||file.size>1000000)throw Error(t.logoError);
  const data=await new Promise((resolve,reject)=>{const r=new FileReader();r.onload=()=>resolve(r.result);r.onerror=reject;r.readAsDataURL(file);});
  const img=new Image();img.src=data;await img.decode();
  logoDataUrl=data;generate();
 }catch(e){error(e);}
});
$('logo-clear')?.addEventListener('click',()=>{logoDataUrl=undefined;$('logo-file').value='';generate();});
