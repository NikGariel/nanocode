// Match CSS object-fit: contain inside a square viewport, in percentages.
export function containedFrame(width,height){
  const scale=100/Math.max(width,height);
  const w=width*scale,h=height*scale;
  return {width:w,height:h,left:(100-w)/2,top:(100-h)/2};
}
// The native pose points correspond to (+/- .72, +/- .72). Project a ring
// through the square-to-quadrilateral homography, preserving actual perspective.
export function projectedRing(corners,radius=.97,steps=96){
  const [x0,y0,x1,y1,x2,y2,x3,y3]=corners;
  const dx1=x1-x2,dx2=x3-x2,dx3=x0-x1+x2-x3;
  const dy1=y1-y2,dy2=y3-y2,dy3=y0-y1+y2-y3;
  const det=dx1*dy2-dx2*dy1;
  if(!Number.isFinite(det)||Math.abs(det)<1e-8)return [];
  const g=(dx3*dy2-dx2*dy3)/det,h=(dx1*dy3-dx3*dy1)/det;
  const a=x1-x0+g*x1,b=x3-x0+h*x3,d=y1-y0+g*y1,e=y3-y0+h*y3;
  return Array.from({length:steps+1},(_,i)=>{
    const angle=i*2*Math.PI/steps,u=.5+radius*Math.cos(angle)/1.44,v=.5+radius*Math.sin(angle)/1.44,z=g*u+h*v+1;
    return [(a*u+b*v+x0)/z,(d*u+e*v+y0)/z];
  });
}
