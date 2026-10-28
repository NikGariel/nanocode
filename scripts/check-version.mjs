import {readFile} from 'node:fs/promises';
const version=JSON.parse(await readFile('package.json')).version;
const files=['packages/js/package.json','crates/nanocode/Cargo.toml','packages/flutter/pubspec.yaml','packages/android/nanocode/build.gradle','packages/flutter/ios/nanocode.podspec','packages/flutter/macos/nanocode.podspec'];
for(const file of files)if(!(await readFile(file,'utf8')).includes(version))throw Error(`Version mismatch: ${file}`);
if(process.env.GITHUB_REF_TYPE==='tag'&&process.env.GITHUB_REF_NAME!==`v${version}`)throw Error('Tag does not match package version');
console.log(version);
