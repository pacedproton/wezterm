// swift-tools-version:5.5
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "WezTermMacOSNative",
    platforms: [
        .macOS(.v11)
    ],
    products: [
        .library(
            name: "WezTermBridge",
            targets: ["WezTermBridge"]),
        .library(
            name: "WezTermPreferences",
            targets: ["WezTermPreferences"]),
        .library(
            name: "WezTermServices",
            targets: ["WezTermServices"]),
    ],
    dependencies: [],
    targets: [
        .target(
            name: "WezTermBridge",
            dependencies: [],
            path: "Sources/WezTermBridge",
            publicHeadersPath: ".",
            cSettings: [
                .headerSearchPath("."),
            ]),
        .target(
            name: "WezTermPreferences",
            dependencies: ["WezTermBridge"],
            path: "Sources/Preferences"),
        .target(
            name: "WezTermServices",
            dependencies: ["WezTermBridge"],
            path: "Sources/Services"),
        .testTarget(
            name: "WezTermBridgeTests",
            dependencies: ["WezTermBridge"]),
    ]
)
