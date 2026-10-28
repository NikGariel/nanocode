from pathlib import Path
import hashlib,sys
root=Path(sys.argv[1]);lines=[]
for p in sorted(root.iterdir()):
    if p.is_file() and p.name!='SHA256SUMS.txt': lines.append(hashlib.sha256(p.read_bytes()).hexdigest()+'  '+p.name)
(root/'SHA256SUMS.txt').write_text('\n'.join(lines)+'\n')
