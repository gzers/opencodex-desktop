import AppKit
import CoreGraphics
import Foundation
let pid = Int32(CommandLine.arguments[1])!
let phase = CommandLine.arguments[2]
let target = CommandLine.arguments[3]
let requiredSeconds = CommandLine.arguments.count > 4 ? Double(CommandLine.arguments[4])! : 46.0
let started = Date().timeIntervalSince1970
let uptime = ProcessInfo.processInfo.systemUptime
var samples: [[String: Any]] = []
while true {
  let elapsed = ProcessInfo.processInfo.systemUptime - uptime
  let app = NSWorkspace.shared.frontmostApplication
  let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
  let visible = windows.contains { ($0[kCGWindowOwnerPID as String] as? Int32) == pid && ($0[kCGWindowLayer as String] as? Int) == 0 }
  samples.append(["wall_s": Date().timeIntervalSince1970, "elapsed_s": elapsed, "target_frontmost": app?.processIdentifier == pid, "target_window_on_screen": visible])
  if elapsed >= requiredSeconds { break }
  RunLoop.current.run(until: Date(timeIntervalSinceNow: 1))
}
let value: [String: Any] = ["phase": phase, "pid": pid, "started_at": started, "ended_at": Date().timeIntervalSince1970, "duration_monotonic_s": ProcessInfo.processInfo.systemUptime - uptime, "method": "NSWorkspace frontmost PID plus CoreGraphics on-screen layer-zero window; one-second sampling, runloop serviced; no UI actions", "samples": samples, "all_samples_frontmost_visible": samples.allSatisfy { ($0["target_frontmost"] as? Bool == true) && ($0["target_window_on_screen"] as? Bool == true) }]
let data = try JSONSerialization.data(withJSONObject: value, options: [.prettyPrinted, .sortedKeys])
try data.write(to: URL(fileURLWithPath: target), options: .withoutOverwriting)
print(phase, pid, samples.count, value["all_samples_frontmost_visible"]!)
