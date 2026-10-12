import AppKit
import CoreGraphics
import Foundation
import Darwin

let pid = Int32(CommandLine.arguments[1])!
let stopFile = CommandLine.arguments[2]
let limit = Double(CommandLine.arguments[3])!
let epoch = ProcessInfo.processInfo.systemUptime
func emit(_ value: [String: Any]) {
  let data = try! JSONSerialization.data(withJSONObject: value, options: [.sortedKeys])
  print(String(data: data, encoding: .utf8)!)
  fflush(stdout)
}
func snapshot() -> [String: Any] {
  let frontmost = NSWorkspace.shared.frontmostApplication?.processIdentifier == pid
  let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
  let visible = windows.contains {
    ($0[kCGWindowOwnerPID as String] as? Int32) == pid && ($0[kCGWindowLayer as String] as? Int) == 0
  }
  return [
    "kind": "sample", "target_pid": pid,
    "wall_s": Date().timeIntervalSince1970,
    "elapsed_s": ProcessInfo.processInfo.systemUptime - epoch,
    "target_frontmost": frontmost, "target_window_on_screen": visible
  ]
}
var armed = false
var last: Double? = nil
while true {
  let value = snapshot()
  let elapsed = value["elapsed_s"] as! Double
  let ready = value["target_frontmost"] as! Bool && value["target_window_on_screen"] as! Bool
  emit(value)
  if armed {
    if !ready { emit(["kind": "invalid", "reason": "foreground_or_visible_window_lost", "elapsed_s": elapsed]); exit(3) }
    if let prior = last, elapsed - prior > 1 {
      emit(["kind": "invalid", "reason": "sampling_gap_over_one_second", "gap_s": elapsed-prior]); exit(4)
    }
  } else if ready {
    armed = true
    emit(["kind": "ready", "elapsed_s": elapsed, "wall_s": Date().timeIntervalSince1970])
  } else if elapsed >= 10 {
    emit(["kind": "invalid", "reason": "not_ready_within_ten_seconds"]); exit(2)
  }
  last = elapsed
  if FileManager.default.fileExists(atPath: stopFile) {
    emit(["kind": "stopped", "armed": armed, "elapsed_s": elapsed]); exit(0)
  }
  if elapsed >= limit { emit(["kind": "invalid", "reason": "monitor_time_limit"]); exit(5) }
  RunLoop.current.run(until: Date(timeIntervalSinceNow: 0.1))
}
