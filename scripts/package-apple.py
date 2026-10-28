import pathlib, shutil, sys, zipfile
source = pathlib.Path(sys.argv[1])
root = pathlib.Path('packages/apple')
dest = root / 'CNanoCode.xcframework'
if dest.exists(): shutil.rmtree(dest)
shutil.copytree(source, dest)
shutil.copy('LICENSE', root / 'LICENSE')
with zipfile.ZipFile('artifacts/nanocode-apple.zip','w',zipfile.ZIP_DEFLATED) as archive:
    for p in root.rglob('*'):
        relative=p.relative_to(root)
        if p.is_file() and not any(part.startswith('.') for part in relative.parts): archive.write(p,relative)
