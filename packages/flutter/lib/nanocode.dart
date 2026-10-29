library nanocode;

import 'dart:convert';
import 'dart:ffi';
import 'dart:io';
import 'dart:typed_data';
import 'package:ffi/ffi.dart';

typedef _SvgN = Int32 Function(Pointer<Uint8>, Size, Uint32, Uint32, Uint32, Uint32, Pointer<Uint8>, Size);
typedef _Svg = int Function(Pointer<Uint8>, int, int, int, int, int, Pointer<Uint8>, int);
typedef _ScanN = Int32 Function(Pointer<Uint8>, Size, Size, Pointer<Uint8>, Size, Pointer<Double>, Pointer<Uint32>);
typedef _Scan = int Function(Pointer<Uint8>, int, int, Pointer<Uint8>, int, Pointer<Double>, Pointer<Uint32>);
typedef _ScoreN = Int32 Function(Uint32, Uint32, Uint32);
typedef _Score = int Function(int, int, int);
typedef _FlagsN = Uint32 Function(Uint32, Uint32, Uint32);
typedef _BgN = Int32 Function(Uint32, Uint32, Pointer<Uint32>, Size);
typedef _Bg = int Function(int, int, Pointer<Uint32>, int);

class PaletteAssessment {
  final int score, flags;
  PaletteAssessment(this.score, this.flags);
  bool get allowed => flags == 0;
  bool get recommended => allowed && score >= 55;
}
class NanoCodeException implements Exception {
  final int code;
  NanoCodeException(this.code);
  @override
  String toString() => 'NanoCode native error: $code';
}
class Detection {
  final Uint8List bytes;
  final int format, corrected;
  final List<double> corners;
  Detection(this.bytes, this.format, this.corrected, this.corners);
  String get text => utf8.decode(bytes, allowMalformed: true);
}
/// Synchronous FFI. Run scanning in an isolate; supply packed grayscale camera data.
/// Not available on Flutter Web: use the JavaScript/WASM package there.
class NanoCode {
  final DynamicLibrary library;
  late final _Svg _svg = library.lookupFunction<_SvgN, _Svg>('nc_svg_ex');
  late final _Scan _scan = library.lookupFunction<_ScanN, _Scan>('nc_scan_ex');
  late final _Score _score = library.lookupFunction<_ScoreN, _Score>('nc_palette_score');
  late final _Bg _backgrounds = library.lookupFunction<_BgN, _Bg>('nc_backgrounds');
  late final _Bg _secondaryColors = library.lookupFunction<_BgN, _Bg>('nc_secondary_colors');
  late final _Score _flags = library.lookupFunction<_FlagsN, _Score>('nc_palette_flags');
  NanoCode({DynamicLibrary? library}) : library = library ?? _open();
  static DynamicLibrary _open() {
    if (Platform.isIOS || Platform.isMacOS) return DynamicLibrary.process();
    return DynamicLibrary.open(Platform.isWindows ? 'nanocode.dll' : 'libnanocode.so');
  }
  String svg(Uint8List data, {int format = 0, int one = 0x00a6a1, int zero = 0x77dddd, int background = 0xffffff}) {
    if (data.length > 40 || format < 0 || format > 4) throw ArgumentError('Payload/format out of range');
    if ([one, zero, background].any((c) => c < 0 || c > 0xffffff)) throw ArgumentError('Invalid RGB');
    return using((arena) {
      final input = arena<Uint8>(data.isEmpty ? 1 : data.length);
      input.asTypedList(data.length).setAll(0, data);
      final n = _svg(input, data.length, format, one, zero, background, nullptr, 0);
      if (n < 0) throw NanoCodeException(n);
      final output = arena<Uint8>(n);
      final written = _svg(input, data.length, format, one, zero, background, output, n);
      if (written != n) throw NanoCodeException(written);
      return utf8.decode(output.asTypedList(n));
    });
  }
  Detection? scan(Uint8List gray, int width, int height) {
    if (width < 32 || height < 32 || width > 2048 || height > 2048 || gray.length != width * height) {
      throw ArgumentError('Expected packed grayscale, dimensions 32..2048');
    }
    return using((arena) {
      final input = arena<Uint8>(gray.length);
      input.asTypedList(gray.length).setAll(0, gray);
      final output = arena<Uint8>(40);
      final corners = arena<Double>(8);
      final meta = arena<Uint32>(2);
      final n = _scan(input, width, height, output, 40, corners, meta);
      if (n == -3) return null;
      if (n < 0) throw NanoCodeException(n);
      return Detection(Uint8List.fromList(output.asTypedList(n)), meta[0], meta[1], List<double>.from(corners.asTypedList(8)));
    });
  }
  PaletteAssessment assess(int one, int zero, int background) {
    if ([one, zero, background].any((c) => c < 0 || c > 0xffffff)) return PaletteAssessment(0, 1);
    return PaletteAssessment(_score(one, zero, background), _flags(one, zero, background));
  }
  List<int> secondaryColors(int one, [int background = 0xffffff]) {
    if ([one, background].any((c) => c < 0 || c > 0xffffff)) throw ArgumentError('Invalid RGB');
    return using((arena) {
      final output = arena<Uint32>(8);
      final n = _secondaryColors(one, background, output, 8);
      if (n < 0) throw NanoCodeException(n);
      return List<int>.from(output.asTypedList(n));
    });
  }
  int paletteScore(int one, int zero, int background) => _score(one, zero, background);
  List<int> backgrounds(int one, int zero) => using((arena) {
    final output = arena<Uint32>(8);
    final n = _backgrounds(one, zero, output, 8);
    if (n < 0) throw NanoCodeException(n);
    return List<int>.from(output.asTypedList(n));
  });
}
