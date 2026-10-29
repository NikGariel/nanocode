export const setup = `git clone https://github.com/NikGariel/nanocode.git
cd nanocode
rustup target add wasm32-unknown-unknown
npm ci
npm run build
npm run dev`;
export const examples = [
{ name:'Web / JavaScript', lang:'js', install:'npm install ./nikgariel-nanocode-0.2.3.tgz', code:`import {loadNanoCode} from '@nikgariel/nanocode';
const nc = await loadNanoCode();
const svg = nc.svg('ORBIT:EVT:042', {
  one: '#00a6a1', zero: '#77dddd', background: '#ffffff'
});
document.querySelector('#code').innerHTML = svg;
const backgrounds = nc.backgrounds('#00a6a1', '#77dddd');
// In a Worker: packed grayscale, dimensions 32..2048.
const result = nc.scan(gray, width, height);
if (result) console.log(result.bytes, result.formatName);`},
{ name:'Node.js', lang:'js', install:'npm install ./nikgariel-nanocode-0.2.3.tgz', code:`import {readFile, writeFile} from 'node:fs/promises';
import {loadNanoCode} from '@nikgariel/nanocode';
const nc = await loadNanoCode(await readFile(
  new URL(import.meta.resolve('@nikgariel/nanocode/nanocode.wasm'))
));
await writeFile('code.svg', nc.svg('ORBIT:EVT:042'));`},
{ name:'iOS / iPadOS / macOS · Swift', lang:'swift', install:'unzip nanocode-apple.zip -d NanoCode\n# Xcode → Add Package Dependencies → Add Local → NanoCode', code:`import Foundation
import NanoCode
let svg = try NanoCode.svg(Data("ORBIT:EVT:042".utf8))
let backgrounds = try NanoCode.backgrounds(one: 0x00a6a1, zero: 0x77dddd)
// Background queue; tightly packed camera Y-plane, without row padding.
if let result = try NanoCode.scan(gray, width: width, height: height) {
    print(result.bytes, result.format)
}`},
{ name:'Android · Kotlin', lang:'kotlin', install:'// Add the extracted release Maven repository to settings.gradle:\nmaven { url = uri("../nanocode-maven") }\n// app/build.gradle:\nimplementation("dev.nanocode:nanocode:0.2.3")', code:`import dev.nanocode.NanoCode
val svg = NanoCode.svg("ORBIT:EVT:042".toByteArray(Charsets.UTF_8))
val backgrounds = NanoCode.backgrounds(0x00a6a1, 0x77dddd)
// CameraX analysis executor: packed Y bytes, respect rowStride/pixelStride.
val result = NanoCode.scan(gray, width, height)
if (result != null) println(result.text())
// Always close the CameraX ImageProxy in a finally block.`},
{ name:'Flutter / Dart · native', lang:'dart', install:'dependencies:\n  nanocode:\n    path: ../flutter # extracted nanocode-flutter-x86_64.tar.gz', code:`import 'dart:convert';
import 'dart:typed_data';
import 'package:nanocode/nanocode.dart';
final nc = NanoCode();
final svg = nc.svg(Uint8List.fromList(utf8.encode('ORBIT:EVT:042')));
// Run camera analysis in an isolate. Not Flutter Web.
final result = nc.scan(gray, width, height);
if (result != null) print(result.bytes);`},
{ name:'Rust', lang:'rust', install:'[dependencies]\nnanocode = { path = "../nanocode/crates/nanocode" }', code:`use nanocode::{family, palette::Palette, vision};
fn main() -> Result<(), nanocode::Error> {
    let data = b"ORBIT:EVT:042";
    let svg = family::svg(data, 0, Palette::default())?;
    let gray = family::render_gray(data, 512, 0, Palette::default())?;
    let found = vision::scan(&gray, 512, 512)?;
    assert_eq!(found.payload, data);
    println!("{}", svg);
    Ok(())
}`},
{ name:'C / C++', lang:'c', install:'cargo build --release --lib\nclang -I include examples/family.c -L target/release -lnanocode -o nanocode-example', code:`#include "nanocode.h"
#include <stdlib.h>
int main(void) {
    const uint8_t id[] = "ORBIT:EVT:042";
    int32_t n = nc_svg_ex(id, sizeof(id)-1, 0,
        0x00a6a1, 0x77dddd, 0xffffff, NULL, 0);
    if (n < 0) return 1;
    uint8_t *svg = malloc((size_t)n);
    if (!svg) return 1;
    int32_t written = nc_svg_ex(id, sizeof(id)-1, 0,
        0x00a6a1, 0x77dddd, 0xffffff, svg, (size_t)n);
    /* svg contains written bytes, without a NUL terminator. */
    free(svg);
    return written == n ? 0 : 1;
}`},
{ name:'Python · ctypes', lang:'python', install:'# No Python dependencies. Use a matching native release library.', code:`import ctypes as c
lib = c.CDLL('./libnanocode.dylib')  # .so on Linux, nanocode.dll on Windows
lib.nc_svg_ex.argtypes = [c.c_void_p, c.c_size_t, c.c_uint32,
    c.c_uint32, c.c_uint32, c.c_uint32, c.c_void_p, c.c_size_t]
lib.nc_svg_ex.restype = c.c_int32
data = b'ORBIT:EVT:042'
args = (data, len(data), 0, 0x00a6a1, 0x77dddd, 0xffffff)
n = lib.nc_svg_ex(*args, None, 0)
if n < 0: raise ValueError(n)
out = c.create_string_buffer(n)
assert lib.nc_svg_ex(*args, out, n) == n
svg = out.raw.decode('utf-8')`},
{ name:'.NET · C#', lang:'csharp', install:'// Place the matching nanocode native library on the application library path.', code:`using System;
using System.Runtime.InteropServices;
using System.Text;
static class NanoCode {
    [DllImport("nanocode", CallingConvention = CallingConvention.Cdecl)]
    static extern int nc_svg_ex(byte[] data, nuint len, uint format,
        uint one, uint zero, uint background, [Out] byte[]? output, nuint capacity);
    public static string Svg(byte[] data) {
        int n = nc_svg_ex(data, (nuint)data.Length, 0, 0x00a6a1, 0x77dddd, 0xffffff, null, 0);
        if (n < 0) throw new ArgumentException($"NanoCode error {n}");
        var output = new byte[n];
        int written = nc_svg_ex(data, (nuint)data.Length, 0, 0x00a6a1, 0x77dddd, 0xffffff, output, (nuint)n);
        if (written != n) throw new InvalidOperationException();
        return Encoding.UTF8.GetString(output);
    }
}`}
];
