import {loadNanoCode} from '../packages/js/index.js';
import {projectedRing} from './scan-geometry.js';
const engine = loadNanoCode();
let tracked=null;
function cropScan(api,gray,width,height,box) {
 const x=Math.max(0,Math.floor(box.x)),y=Math.max(0,Math.floor(box.y));
 const w=Math.min(width-x,Math.ceil(box.w)),h=Math.min(height-y,Math.ceil(box.h));
 if(w<64||h<64)return null;
 const crop=new Uint8Array(w*h);
 for(let row=0;row<h;row++)crop.set(gray.subarray((y+row)*width+x,(y+row)*width+x+w),row*w);
 const result=api.scan(crop,w,h);
 if(result)result.corners=result.corners.map((v,i)=>v+(i%2?y:x));
 return result;
}
onmessage = async ({data}) => {
  try {
    const api=await engine,started=performance.now(),gray=new Uint8Array(data.gray),{width,height}=data;
    let result=null;
    if(tracked){
      result=cropScan(api,gray,width,height,{x:tracked.x*width,y:tracked.y*height,w:tracked.w*width,h:tracked.h*height});
    }
    if(!result)result=api.scan(gray,width,height);
    if(result){
      const points=projectedRing(result.corners),xs=points.map(p=>p[0]),ys=points.map(p=>p[1]);
      const left=Math.min(...xs),top=Math.min(...ys),w=Math.max(...xs)-left,h=Math.max(...ys)-top;
      tracked=points.length?{x:(left-w*.08)/width,y:(top-h*.08)/height,w:w*1.16/width,h:h*1.16/height}:null;
    }else tracked=null;
    // Return the transferred buffer so capture can reuse it on the next frame.
    postMessage({result,ms:performance.now()-started,id:data.id,gray:data.gray},[data.gray]);
  }
  catch(e) {postMessage({error:e.message,id:data.id});}
};
