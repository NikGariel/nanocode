from pathlib import Path
import shutil
shutil.copy('packages/android/nanocode/build/outputs/aar/nanocode-release.aar','artifacts/nanocode-android.aar')
shutil.make_archive('artifacts/nanocode-maven','zip','artifacts/maven')
for abi,target in [('arm64-v8a','aarch64-linux-android'),('armeabi-v7a','armv7-linux-androideabi'),('x86_64','x86_64-linux-android'),('x86','i686-linux-android')]:
    dest=Path('artifacts/android')/abi; dest.mkdir(parents=True,exist_ok=True)
    shutil.copy(Path('target')/target/'release/libnanocode.so',dest)
