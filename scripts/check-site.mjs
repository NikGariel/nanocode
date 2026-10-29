import {readFile,stat} from 'node:fs/promises';
import {resolve,dirname} from 'node:path';
import assert from 'node:assert/strict';
import {locales} from '../site/content/locales.mjs';
import {ui} from '../site/content/ui.mjs';
const root=resolve('dist');let checked=0;
for(const [lang,l] of Object.entries(locales)) {
 assert.deepEqual(Object.keys(l),Object.keys(locales.en),lang);
 assert.deepEqual(Object.keys(ui[lang]),Object.keys(ui.en),lang);
 assert.equal(l.headings.length,l.sections.length);
 for(const name of ['index.html','docs.html','playground.html','scan.html']){
  const file=resolve(root,lang,name),html=await readFile(file,'utf8');
  assert(html.includes(`<html lang="${lang}">`));
  assert.equal((html.match(/hreflang="x-default"/g)||[]).length,1);
  for(const other of Object.keys(locales))assert(html.includes(`rel="alternate" hreflang="${other}"`));
  assert(html.includes('rel="canonical"'));
  JSON.parse(html.match(/<script type="application\/ld\+json">([^]*?)<\/script>/)[1]);
  if(name==='docs.html'){
   assert.equal((html.match(/<section id=/g)||[]).length,12);
   for(const title of l.headings)assert(html.includes(title));
   assert(html.includes('nc_svg_ex')&&html.includes('logoDataUrl'));
  }
  for(const m of html.matchAll(/(?:href|src)="([^"#]+)(?:#[^"]*)?"/g)){
   const href=m[1];if(/^(https?:|data:|mailto:)/.test(href))continue;
   const target=resolve(dirname(file),href.split("?")[0]);
   assert(target.startsWith(root+'/'),`Outside site: ${href}`);
   assert((await stat(target)).isFile(),`Missing: ${target}`);checked++;
  }
  assert(!html.includes('undefined'),`${file}: undefined content`);
 }
 const md=await readFile(`docs/i18n/README.${lang}.md`,'utf8');
 assert.equal((md.match(/^## /gm)||[]).length,12);
 assert(md.includes('40')&&md.includes(JSON.parse(await readFile('package.json')).version));
}
const sitemap=await readFile('dist/sitemap.xml','utf8');
assert.equal((sitemap.match(/<loc>/g)||[]).length,Object.keys(locales).length*4);
assert((await readFile('dist/llms-full.txt','utf8')).includes('Hamming'));
console.log(`Site: ${Object.keys(locales).length} complete locales, 36 pages, ${checked} internal links, sitemap, metadata, Markdown OK.`);
