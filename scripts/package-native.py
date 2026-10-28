from pathlib import Path
import shutil, sys
triple=sys.argv[1]
source=Path('target')/triple/'release'
dest=Path('artifacts/native')/triple
dest.mkdir(parents=True,exist_ok=True)
for name in ['libnanocode.a','libnanocode.so','libnanocode.dylib','nanocode.dll','nanocode.lib','nanocode.dll.lib','nanocode','nanocode.exe']:
    if (source/name).exists(): shutil.copy(source/name,dest/name)
shutil.copy('include/nanocode.h',dest)
shutil.copy('LICENSE',dest)
shutil.make_archive('artifacts/nanocode-'+triple,'gztar','artifacts/native',triple)
