// swift-tools-version: 5.9
import PackageDescription
let package = Package(
    name: "NanoCode", platforms: [.iOS(.v13), .macOS(.v11)],
    products: [.library(name: "NanoCode", targets: ["NanoCode"])],
    targets: [.binaryTarget(name: "CNanoCode", path: "CNanoCode.xcframework"),
              .target(name: "NanoCode", dependencies: ["CNanoCode"]),
              .testTarget(name: "NanoCodeTests", dependencies: ["NanoCode"])])
