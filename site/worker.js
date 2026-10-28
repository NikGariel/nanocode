import {loadNanoCode} from '../packages/js/index.js';
const engine = loadNanoCode();
onmessage = async ({data}) => {
  try {const api=await engine;const started=performance.now();const result=api.scan(new Uint8Array(data.gray),data.width,data.height);postMessage({result,ms:performance.now()-started,id:data.id});}
  catch(e) {postMessage({error:e.message,id:data.id});}
};
