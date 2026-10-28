import assert from 'node:assert/strict';
import {containedFrame,projectedRing} from '../site/scan-geometry.js';
assert.deepEqual(containedFrame(1920,1080),{width:100,height:56.25,left:0,top:21.875});
assert.deepEqual(containedFrame(1080,1920),{width:56.25,height:100,left:21.875,top:0});
assert.deepEqual(containedFrame(512,512),{width:100,height:100,left:0,top:0});
const points=projectedRing([-72,-72,72,-72,72,72,-72,72]);
for(const [x,y] of points)assert(Math.abs(Math.hypot(x,y)-97)<1e-6);
// Pose can be an arbitrary quadrilateral: all projected samples must stay finite.
assert(projectedRing([20,30,450,60,390,430,50,390]).every(p=>p.every(Number.isFinite)));
assert.deepEqual(projectedRing([0,0,0,0,0,0,0,0]),[]);
console.log('Square viewport: landscape/portrait containment and projected circular pose passed.');
