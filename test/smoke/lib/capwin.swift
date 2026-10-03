// 按窗口编号截图（不依赖窗口是否在前台）。
// macOS 15 起 CGWindowListCreateImage 被标记为不可用，这里用 dlsym 取符号保持可用。
import Foundation
import CoreGraphics
import AppKit

typealias CaptureFn = @convention(c) (CGRect, CGWindowListOption, CGWindowID, CGWindowImageOption) -> Unmanaged<CGImage>?

let args = CommandLine.arguments
guard args.count >= 3, let rawID = UInt32(args[1]) else {
    FileHandle.standardError.write("usage: capwin <windowNumber> <out.png>\n".data(using: .utf8)!)
    exit(1)
}
let handle = dlopen("/System/Library/Frameworks/CoreGraphics.framework/CoreGraphics", RTLD_NOW)
guard let symbol = dlsym(handle, "CGWindowListCreateImage") else {
    FileHandle.standardError.write("CGWindowListCreateImage unavailable\n".data(using: .utf8)!)
    exit(2)
}
let capture = unsafeBitCast(symbol, to: CaptureFn.self)
guard let image = capture(.null, .optionIncludingWindow, CGWindowID(rawID), [.boundsIgnoreFraming, .bestResolution])?.takeRetainedValue() else {
    FileHandle.standardError.write("no image for window \(rawID)\n".data(using: .utf8)!)
    exit(3)
}
let rep = NSBitmapImageRep(cgImage: image)
guard let png = rep.representation(using: .png, properties: [:]) else { exit(4) }
try png.write(to: URL(fileURLWithPath: args[2]))
print("wrote \(image.width)x\(image.height) -> \(args[2])")
