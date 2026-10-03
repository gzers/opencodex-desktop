// 打印当前图形会话是否锁屏。
// 锁屏时 macOS 不为 WKWebView 合成内容，截图会接近纯白，所有像素断言都不可信。
import Foundation
import CoreGraphics

if let session = CGSessionCopyCurrentDictionary() as? [String: Any] {
    print("CGSSessionScreenIsLocked: \(session["CGSSessionScreenIsLocked"] ?? "nil")")
    print("kCGSSessionOnConsoleKey: \(session["kCGSSessionOnConsoleKey"] ?? "nil")")
    print("kCGSSessionUserNameKey: \(session["kCGSSessionUserNameKey"] ?? "nil")")
} else {
    print("no session dict")
}
