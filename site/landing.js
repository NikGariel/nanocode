import QRCode from 'qrcode';
import {loadNanoCode} from '../packages/js/index.js';
import {locales} from './content/locales.mjs';
const lang=document.documentElement.lang, l=locales[lang]||locales.en;
const api=await loadNanoCode();
const brand=svg=>svg.replace('</svg>','<path fill="#15352f" d="M -.13 .12 V -.12 H -.075 L .075 .055 V -.12 H .13 V .12 H .075 L -.075 -.055 V .12 Z"/></svg>');
const scenes=['screen','building','scooter'];
const payloads=['ORBIT:EVT:042','ATELIER:DOOR:07','LOOP:RIDE:024'];
function choose(name){
 const i=scenes.indexOf(name);if(i<0)return;
 document.querySelector('.scene').dataset.photo=name;
 document.querySelectorAll('[data-scene]').forEach(b=>b.setAttribute('aria-selected',String(b.dataset.scene===name)));
 document.querySelector('#scene-image').src=new URL(`./scene-${name}.jpg`+new URL(import.meta.url).search,import.meta.url);
 document.querySelector('#scene-image').alt=l.scenes[i][0];
 document.querySelector('#scene-code').innerHTML=brand(api.svg(payloads[i]));
 document.querySelector('#scene-category').textContent=['ORBIT / EVENTS','ATELIER / SPACES','LOOP / MOBILITY'][i];
 document.querySelector('#scene-title').textContent=l.scenes[i][0];
 document.querySelector('#scene-description').textContent=l.scenes[i][1];
 document.querySelector('#scene-payload').textContent=payloads[i];
}
const tabs=[...document.querySelectorAll('[data-scene]')];
tabs.forEach((b,i)=>{b.onclick=()=>choose(b.dataset.scene);b.onkeydown=e=>{if(['ArrowLeft','ArrowRight'].includes(e.key)){e.preventDefault();const next=tabs[(i+(e.key==='ArrowRight'?1:2))%3];next.focus();choose(next.dataset.scene);}};});
const local=['localhost','127.0.0.1',''].includes(location.hostname);
const link=document.querySelector('#phone-link');
const url=local?`https://nikgariel.github.io/nanocode/${lang}/scan.html`:link.href;
link.href=url;link.textContent=url;
await QRCode.toCanvas(document.querySelector('#phone-qr'),url,{width:228,margin:2,errorCorrectionLevel:'M',color:{dark:'#15352f',light:'#ffffff'}});
if(local)document.querySelector('#share-status').textContent=l.labels[16];
const tile=document.querySelector('#scene-code');
const dialog=document.createElement('dialog');dialog.className='code-dialog';
const close=document.createElement('button');close.textContent='×';close.setAttribute('aria-label',l.nav[0]+' ←');
const full=document.createElement('div');close.onclick=()=>dialog.close();dialog.append(close,full);document.body.append(dialog);
function enlarge(){full.innerHTML=tile.innerHTML;dialog.showModal();}
document.querySelector('#enlarge-code').onclick=enlarge;
tile.onclick=enlarge;tile.onkeydown=e=>{if(e.key==='Enter'||e.key===' '){e.preventDefault();enlarge();}};
dialog.onclick=e=>{if(e.target===dialog)dialog.close();};
