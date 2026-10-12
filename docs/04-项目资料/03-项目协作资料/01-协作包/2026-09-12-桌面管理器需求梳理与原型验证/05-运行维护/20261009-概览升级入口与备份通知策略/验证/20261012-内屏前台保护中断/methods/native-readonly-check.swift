import AppKit
import ApplicationServices
let pid: pid_t = 31501
let target = NSRunningApplication(processIdentifier: pid)
let front = NSWorkspace.shared.frontmostApplication
var focused: CFTypeRef?
let status = AXUIElementCopyAttributeValue(AXUIElementCreateSystemWide(), kAXFocusedApplicationAttribute as CFString, &focused)
var focusedPid: pid_t = 0
if status == .success, let focused { AXUIElementGetPid(unsafeBitCast(focused, to: AXUIElement.self), &focusedPid) }
let windows = (CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []).filter { ($0[kCGWindowOwnerPID as String] as? Int) == Int(pid) && ($0[kCGWindowLayer as String] as? Int) == 0 }.map { entry -> [String: Any] in ["bounds": entry[kCGWindowBounds as String] ?? [:], "window_number": entry[kCGWindowNumber as String] ?? 0] }
let displays = NSScreen.screens.map { screen -> [String: Any] in let id = (screen.deviceDescription[NSDeviceDescriptionKey("NSScreenNumber")] as? NSNumber)?.uint32Value ?? 0; return ["id": id, "builtin": CGDisplayIsBuiltin(id) != 0, "cg_bounds": ["x": CGDisplayBounds(id).origin.x, "y": CGDisplayBounds(id).origin.y, "width": CGDisplayBounds(id).width, "height": CGDisplayBounds(id).height]] }
let result: [String: Any] = ["timestamp": ISO8601DateFormatter().string(from: Date()), "target_pid": pid, "target_running": target != nil, "target_active": target?.isActive ?? false, "workspace_frontmost_is_target": front?.processIdentifier == pid, "ax_focused_is_target": focusedPid == pid, "ax_status": status.rawValue, "target_windows": windows, "displays": displays]
print(String(data: try JSONSerialization.data(withJSONObject: result, options: [.prettyPrinted, .sortedKeys]), encoding: .utf8)!)
