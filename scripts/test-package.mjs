import {mkdtemp,readFile,writeFile,readdir} from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {tmpdir} from 'node:os';
import {join,resolve} from 'node:path';
const folder=await mkdtemp(join(tmpdir(),'nanocode-package-'));
execFileSync('npm',['pack','--workspace','@nikgariel/nanocode','--pack-destination',folder],{stdio:'pipe'});
const file=(await readdir(folder)).find(n=>n.endsWith('.tgz'));
execFileSync('tar',['-xzf',join(folder,file),'-C',folder]);
const {loadNanoCode}=await import('file://'+join(folder,'package/index.js'));
const api=await loadNanoCode(await readFile(join(folder,'package/nanocode.wasm')));
if(!api.svg('BRAND:01').includes('<svg'))throw Error('Packed SVG failed');
for(const f of api.formats){
 if(api.formatFor(new Uint8Array(f.capacity),f.id).capacity!==f.capacity)throw Error('Format failed');
}
if(api.assess({one:0,zero:0,background:0}).allowed)throw Error('Palette failed');
const suggestions=api.secondaryColors('#00a6a1','#ffffff');
if(!suggestions.length||!suggestions.every(s=>api.assess({one:'#00a6a1',zero:s.color,background:'#ffffff'}).recommended))throw Error('Packed recommendations failed');
const dot='data:image/png;base64,iVBORw0KGgo=';
if(!api.svg('BRAND:01',{logoDataUrl:dot}).includes('<image'))throw Error('Logo embedding failed');
for(const url of ['https://example.com/logo.png','data:image/svg+xml;base64,PHN2Zz4=']){
 let rejected=false;try{api.svg('BRAND:01',{logoDataUrl:url});}catch{rejected=true;}if(!rejected)throw Error('Unsafe logo accepted');
}
const source=`import {loadNanoCode, type Detection, type Options} from '${resolve('packages/js/index.js')}';\nconst o:Options={format:2};\nconst api=await loadNanoCode(new Uint8Array());\nconst x:Detection|null=api.scan(new Uint8Array(128*128),128,128);\napi.svg('x',o);\nconsole.log(x?.bytes);\n`;
await writeFile(join(folder,'consumer.mts'),source);
console.log(`Packed ESM/WASM consumer passed; TypeScript fixture: ${folder}/consumer.mts`);
