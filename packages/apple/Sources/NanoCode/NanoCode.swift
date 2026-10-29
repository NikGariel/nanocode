import Foundation
import CNanoCode

public enum NanoCodeError: Error { case native(Int32), invalidFrame }
public struct NanoCodeDetection {
    public let bytes: Data
    public let format: UInt32
    public let corrected: UInt32
    public let corners: [Double]
    public var text: String? { String(data: bytes, encoding: .utf8) }
}
public struct PaletteAssessment {
    public let score: Int32
    public let flags: UInt32
    public var allowed: Bool { flags == 0 }
    public var recommended: Bool { allowed && score >= 55 }
}
/// Thread-safe codec. Call scan on a background queue; camera capture belongs to your app.
public enum NanoCode {
    public static func svg(_ data: Data, format: UInt32 = 0,
                           one: UInt32 = 0x00a6a1, zero: UInt32 = 0x77dddd,
                           background: UInt32 = 0xffffff) throws -> String {
        try data.withUnsafeBytes { raw in
            let ptr = raw.bindMemory(to: UInt8.self).baseAddress
            let count = nc_svg_ex(ptr, data.count, format, one, zero, background, nil, 0)
            guard count >= 0 else { throw NanoCodeError.native(count) }
            var output = [UInt8](repeating: 0, count: Int(count))
            let written = nc_svg_ex(ptr, data.count, format, one, zero, background, &output, output.count)
            guard written == count else { throw NanoCodeError.native(written) }
            return String(decoding: output, as: UTF8.self)
        }
    }
    public static func scan(_ gray: Data, width: Int, height: Int) throws -> NanoCodeDetection? {
        guard (32...2048).contains(width), (32...2048).contains(height), gray.count == width * height else {
            throw NanoCodeError.invalidFrame
        }
        return try gray.withUnsafeBytes { raw in
            var output = [UInt8](repeating: 0, count: 40)
            var corners = [Double](repeating: 0, count: 8)
            var metadata = [UInt32](repeating: 0, count: 2)
            let count = nc_scan_ex(raw.bindMemory(to: UInt8.self).baseAddress, width, height,
                                   &output, output.count, &corners, &metadata)
            if count == -3 { return nil }
            guard count >= 0 else { throw NanoCodeError.native(count) }
            return NanoCodeDetection(bytes: Data(output.prefix(Int(count))), format: metadata[0],
                                     corrected: metadata[1], corners: corners)
        }
    }
    public static func backgrounds(one: UInt32, zero: UInt32) throws -> [UInt32] {
        var colors = [UInt32](repeating: 0, count: 8)
        let count = nc_backgrounds(one, zero, &colors, colors.count)
        guard count >= 0 else { throw NanoCodeError.native(count) }
        return Array(colors.prefix(Int(count)))
    }
    public static func secondaryColors(one: UInt32, background: UInt32 = 0xffffff) throws -> [UInt32] {
        var colors = [UInt32](repeating: 0, count: 8)
        let count = nc_secondary_colors(one, background, &colors, colors.count)
        guard count >= 0 else { throw NanoCodeError.native(count) }
        return Array(colors.prefix(Int(count)))
    }
    public static func assess(one: UInt32, zero: UInt32, background: UInt32) -> PaletteAssessment {
        PaletteAssessment(score: nc_palette_score(one, zero, background), flags: nc_palette_flags(one, zero, background))
    }
    public static func paletteScore(one: UInt32, zero: UInt32, background: UInt32) -> Int32 {
        nc_palette_score(one, zero, background)
    }
}
