/** Native Rust codec and detector, compiled unchanged to WebAssembly. */
export async function loadNanoCode(source = new URL('./nanocode.wasm', import.meta.url)) {
  let binary=source;
  if(!(source instanceof Uint8Array)&&!(source instanceof ArrayBuffer)&&!(source instanceof WebAssembly.Module)){
    const response=await fetch(source);
    if(!response.ok)throw new Error('NanoCode WASM could not be loaded');
    binary=await response.arrayBuffer();
  }
  const instantiated=await WebAssembly.instantiate(binary,{});
  const api=(instantiated.instance||instantiated).exports;
  const encoder = new TextEncoder();
  const bytes = () => new Uint8Array(api.memory.buffer);
  const color = value => {
    if(typeof value==='string' && /^#[0-9a-f]{6}$/i.test(value)) return parseInt(value.slice(1),16);
    if(Number.isInteger(value)&&value>=0&&value<=0xffffff) return value;
    throw new Error('Color must be #RRGGBB');
  };
  const hex = n => '#'+n.toString(16).padStart(6,'0');
  const defaults={one:'#00a6a1',zero:'#77dddd',background:'#ffffff'};
  function palette(p={}) {return ['one','zero','background'].map(key=>color(p[key]??defaults[key]));}
  function alloc(n) { const p=api.nc_alloc(n);if(!p)throw new Error('Allocation failed');return p; }
  function check(n) {if(n<0)throw new Error(({'-2':'Payload does not fit the chosen format','-5':'Insufficient brightness separation; choose a different palette'})[n]||`NanoCode error: ${n}`);return n;}
  const formats=[1,2,3,4].map(id=>({id,name:['S','M','L','XL'][id-1],capacity:api.nc_format_capacity(id),rings:api.nc_format_rings(id)}));
  function input(data,fn) {
    const b=typeof data==='string'?encoder.encode(data):data;
    if(!(b instanceof Uint8Array)||b.length>40)throw new Error('Maximum 40 UTF-8 bytes');
    const p=alloc(Math.max(1,b.length));
    try{bytes().set(b,p);return fn(p,b.length);}finally{api.nc_free(p,Math.max(1,b.length));}
  }
  return {
    formats,
    formatFor(data,id=0) {
      const n=typeof data==='string'?encoder.encode(data).length:data.length;
      if(!Number.isInteger(id)||id<0||id>4)throw new Error('Unknown format');
      const chosen=id||check(api.nc_format_for_len(n));const f=formats[chosen-1];if(n>f.capacity)throw new Error('Payload does not fit the chosen format');return f;
    },
    assess(p={}) {
      const c=palette(p),flags=api.nc_palette_flags(...c),score=api.nc_palette_score(...c);
      return {flags,score,allowed:flags===0,recommended:flags===0&&score>=55,issues:[...(flags&2?['Arc colors have insufficient brightness separation']:[]),...(flags&4?['Background brightness is too close to an arc color']:[])]};
    },
    backgrounds(one,zero) {
      const a=color(one),b=color(zero),n=api.nc_backgrounds(a,b,0,0);if(!n)return [];
      const p=alloc(n*4);try{api.nc_backgrounds(a,b,p,n);return Array.from(new Uint32Array(api.memory.buffer,p,n),value=>({color:hex(value),score:api.nc_palette_score(a,b,value)}));}finally{api.nc_free(p,n*4);}
    },
    svg(data,{format=0,logoDataUrl,...p}={}) {
      if(!Number.isInteger(format)||format<0||format>4)throw new Error('Unknown format');
      const c=palette(p);return input(data,(ptr,n)=>{const size=check(api.nc_svg_ex(ptr,n,format,...c,0,0));const out=alloc(size);try{check(api.nc_svg_ex(ptr,n,format,...c,out,size));let svg=new TextDecoder().decode(bytes().slice(out,out+size));
        if(logoDataUrl){if(!/^data:image\/(png|jpeg|webp);base64,[a-zA-Z0-9+/=]+$/.test(logoDataUrl)||logoDataUrl.length>1500000)throw new Error('Logo must be a PNG, JPEG or WebP data URL under 1 MB');svg=svg.replace('</svg>',`<image x="-0.18" y="-0.18" width="0.36" height="0.36" preserveAspectRatio="xMidYMid meet" href="${logoDataUrl}"/></svg>`);}return svg;}finally{api.nc_free(out,size);}});
    },
    scan(gray,width,height) {
      if(!(gray instanceof Uint8Array)||!Number.isInteger(width)||!Number.isInteger(height)||width<32||height<32||width>2048||height>2048||gray.length!==width*height)throw new Error('Invalid luminance frame');
      const p=alloc(gray.length),out=alloc(40),corners=alloc(64),meta=alloc(8);
      try{bytes().set(gray,p);const n=api.nc_scan_ex(p,width,height,out,40,corners,meta);if(n===-3)return null;check(n);const payload=bytes().slice(out,out+n);const [format,corrected]=new Uint32Array(api.memory.buffer,meta,2);return {bytes:payload,text:new TextDecoder().decode(payload),format,formatName:format?formats[format-1].name:'v1',corrected,corners:Array.from(new Float64Array(api.memory.buffer,corners,8))};}
      finally{api.nc_free(p,gray.length);api.nc_free(out,40);api.nc_free(corners,64);api.nc_free(meta,8);}
    }
  };
}
