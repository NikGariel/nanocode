import pathlib, shutil, sys
source = pathlib.Path(sys.argv[1])
dest = pathlib.Path('packages/apple/CNanoCode.xcframework')
if dest.exists(): shutil.rmtree(dest)
shutil.copytree(source, dest)
shutil.copy('LICENSE', 'packages/apple/LICENSE')
shutil.make_archive('artifacts/nanocode-apple', 'zip', 'packages/apple')
