"""Assemble a Flutter package from built native artifacts. Run after release downloads."""
from pathlib import Path
import shutil, sys, tarfile
root = Path('packages/flutter')
source = Path('artifacts/native')
# Explicit architecture: do not silently ship x86_64 desktop libraries for ARM64.
arch = sys.argv[1] if len(sys.argv)>1 else 'x86_64'
for platform, target, name in [('linux',f'{arch}-unknown-linux-gnu','libnanocode.so'),('windows',f'{arch}-pc-windows-msvc','nanocode.dll')]:
    dest=root/platform/'libs'; dest.mkdir(parents=True,exist_ok=True)
    shutil.copy(source/target/name,dest/name)
for abi,target in [('arm64-v8a','aarch64-linux-android'),('armeabi-v7a','armv7-linux-androideabi'),('x86_64','x86_64-linux-android'),('x86','i686-linux-android')]:
    dest=root/'android'/'libs'/abi; dest.mkdir(parents=True,exist_ok=True)
    shutil.copy(Path('artifacts/android')/abi/'libnanocode.so',dest/'libnanocode.so')
for platform in ['ios','macos']:
    dest=root/platform/'CNanoCode.xcframework'
    if dest.exists(): shutil.rmtree(dest)
    shutil.copytree('artifacts/apple-package/CNanoCode.xcframework',dest)
with tarfile.open(f'artifacts/nanocode-flutter-{arch}.tar.gz','w:gz') as archive:
    for p in root.rglob('*'):
        relative=p.relative_to(root)
        if p.is_file() and not any(part.startswith('.') or part in ['build','pubspec.lock'] for part in relative.parts):
            archive.add(p,arcname=str(Path('flutter')/relative))
