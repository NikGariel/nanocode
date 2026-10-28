"""Reproducible black-box experiment; never executes generated payloads."""
import subprocess, pathlib, hashlib, json, math, re, xml.etree.ElementTree as ET
BIN = '/Library/Developer/AppClipCodeGenerator/AppClipCodeGenerator.bundle/Contents/MacOS/AppClipCodeGenerator'
OUT = pathlib.Path('/tmp/nanocode-appclip-research')
OUT.mkdir(exist_ok=True)
records = []
for name, url, kind in [('a','https://example.com/a','cam'),('a-repeat','https://example.com/a','cam'),('b','https://example.com/b','cam'),('query','https://example.com/a?id=1','cam'),('nfc','https://example.com/a','nfc'),('long','https://example.com/'+'a'*150,'cam')]:
    path = OUT / (name+'.svg')
    result = subprocess.run([BIN,'generate','--url',url,'--index','1','--logo','none','--type',kind,'--output',str(path)],capture_output=True,text=True)
    row = dict(name=name,url=url,kind=kind,exit=result.returncode,stdout=result.stdout.strip(),stderr=result.stderr.strip())
    if result.returncode==0 and path.exists():
        raw=path.read_bytes(); root=ET.fromstring(raw); rings=[]
        for g in root.iter('{http://www.w3.org/2000/svg}g'):
            if g.get('name','').startswith('ring-'):
                paths=list(g)
                arcs=[]
                for element in paths:
                    numbers=list(map(float,re.findall(r'-?\d+(?:\.\d+)?',element.get('d',''))))
                    if len(numbers)!=9: continue
                    x,y,r,_,_,_,_,u,v=numbers
                    start=math.atan2(y-400,x-400)*180/math.pi
                    end=math.atan2(v-400,u-400)*180/math.pi
                    arcs.append(dict(radius=r,start=start,end=end,span=(start-end)%360))
                for i,arc in enumerate(arcs):
                    arc['gap_to_next']=(arcs[(i+1)%len(arcs)]['end']-arc['start'])%360
                rings.append(dict(arcs=arcs,name=g.get('name'),transform=g.get('transform'),count=len(paths),colors=''.join(p.get('data-color','?') for p in paths),first=paths[0].attrib if paths else None))
        row.update(sha256=hashlib.sha256(raw).hexdigest(),rings=rings)
    records.append(row)
pathlib.Path('docs/research/appclip-observations.json').write_text(json.dumps(records,indent=2)+'\n')
print(json.dumps([{k:v for k,v in r.items() if k!='rings'} for r in records],indent=2))
