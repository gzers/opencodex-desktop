import Foundation
import CoreGraphics
let ws=CGWindowListCopyWindowInfo([.optionOnScreenOnly,.excludeDesktopElements],kCGNullWindowID) as? [[String:Any]] ?? []
let own=ws.filter{($0[kCGWindowName as String] as? String ?? "").contains("ocxd023-dialog-glass-fixture") && ($0[kCGWindowLayer as String] as? Int ?? -1)==0}
print(String(data:try JSONSerialization.data(withJSONObject:own),encoding:.utf8)!)
