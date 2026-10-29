import assert from 'node:assert/strict';
import {scanDelay,scanSize} from '../site/scan-policy.js';
// A slower decoder must get more rest, never run back-to-back indefinitely.
for(const age of [0,14999,15000,60000])for(const work of [1,10,100,1000,3000]){
 const delay=scanDelay(work,age),limit=age<15000?.30:.15;
 assert(work/(work+delay)<=limit+1e-9);
 assert(delay>=90);
}
assert(scanDelay(100,20000)>scanDelay(100,0));
assert.deepEqual(scanSize(1920,1080,0),{width:640,height:360});
assert.deepEqual(scanSize(1080,1920,3),{width:540,height:960});
// Confirm a high-resolution detection on the next frame, before it ages out.
assert.deepEqual(scanSize(1080,1920,4,true),{width:540,height:960});
assert.deepEqual(scanSize(384,384,1),{width:384,height:384});
console.log('Scan scheduling: bounded duty, long-session backoff, multi-resolution retry passed');
