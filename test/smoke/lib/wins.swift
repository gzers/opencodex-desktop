// 列出指定应用（默认匹配 OpenCodeX）的窗口：编号、层级、是否在屏、边界（点）。
import Foundation
import CoreGraphics

let needle = CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : "OpenCodeX"
let list = CGWindowListCopyWindowInfo([.optionAll], kCGNullWindowID) as? [[String: Any]] ?? []
for window in list {
    let owner = window[kCGWindowOwnerName as String] as? String ?? "?"
    guard owner.contains(needle) else { continue }
    let number = window[kCGWindowNumber as String] as? Int ?? -1
    let layer = window[kCGWindowLayer as String] as? Int ?? -1
    let onscreen = window[kCGWindowIsOnscreen as String] as? Bool ?? false
    let bounds = window[kCGWindowBounds as String] as? [String: Any] ?? [:]
    print("num=\(number) layer=\(layer) onscreen=\(onscreen) bounds=\(bounds)")
}
