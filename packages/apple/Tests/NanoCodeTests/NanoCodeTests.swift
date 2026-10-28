import XCTest
@testable import NanoCode
final class NanoCodeTests: XCTestCase {
    func testGenerationAndValidation() throws {
        XCTAssertTrue(try NanoCode.svg(Data("BRAND:01".utf8)).contains("<svg"))
        XCTAssertThrowsError(try NanoCode.svg(Data(repeating: 1, count: 41)))
        XCTAssertThrowsError(try NanoCode.svg(Data(), one: 0, zero: 0, background: 0))
        XCTAssertFalse(try NanoCode.backgrounds(one: 0x00a6a1, zero: 0x77dddd).isEmpty)
        XCTAssertNil(try NanoCode.scan(Data(repeating: 255, count: 128*128), width: 128, height: 128))
        XCTAssertThrowsError(try NanoCode.scan(Data(), width: 128, height: 128))
    }
}
