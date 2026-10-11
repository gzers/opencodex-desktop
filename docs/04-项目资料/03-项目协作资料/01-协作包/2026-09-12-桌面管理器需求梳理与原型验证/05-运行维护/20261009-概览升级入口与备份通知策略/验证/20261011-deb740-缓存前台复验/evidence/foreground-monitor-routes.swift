import AppKit
import CoreGraphics
import Foundation
let pid = Int32(CommandLine.arguments[1])!
let phase = CommandLine.arguments[2]
let target = CommandLine.arguments[3]
let stopFile = CommandLine.arguments[4]
func sample(_ elapsed: Double) -> [String: Any] {
  let app = NSWorkspace.shared.frontmostApplication
  let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
  let visible = windows.contains { ($0[kCGWindowOwnerPID as String] as? Int32) == pid && ($0[kCGWindowLayer as String] as? Int) == 0 }
  return ["wall_s": Date().timeIntervalSince1970, "elapsed_s": elapsed, "target_frontmost": app?.processIdentifier == pid, "target_window_on_screen": visible]
}
func ready(_ value: [String: Any]) -> Bool {
  (value["target_frontmost"] as? Bool == true) && (value["target_window_on_screen"] as? Bool == true)
}
let observedFrom = Date().timeIntervalSince1970
let waitingUptime = ProcessInfo.processInfo.systemUptime
var readiness: [[String: Any]] = []
while true {
  let s = sample(ProcessInfo.processInfo.systemUptime - waitingUptime)
  readiness.append(s)
  if ready(s) { break }
  if ProcessInfo.processInfo.systemUptime - waitingUptime >= 10 {
    let data = try JSONSerialization.data(withJSONObject: ["phase":phase, "pid":pid, "readiness_samples":readiness, "error":"foreground visible window not ready within 10s"], options: [.prettyPrinted, .sortedKeys])
    try data.write(to: URL(fileURLWithPath:target), options: .withoutOverwriting)
    exit(2)
  }
  RunLoop.current.run(until: Date(timeIntervalSinceNow: 0.1))
}
let started = Date().timeIntervalSince1970
let uptime = ProcessInfo.processInfo.systemUptime
var samples: [[String: Any]] = []
while true {
  let elapsed = ProcessInfo.processInfo.systemUptime - uptime
  samples.append(sample(elapsed))
  if FileManager.default.fileExists(atPath: stopFile) { break }; if elapsed >= 600 { break }
  RunLoop.current.run(until: Date(timeIntervalSinceNow: 0.1))
}
let value: [String: Any] = ["phase":phase, "pid":pid, "observed_from":observedFrom, "readiness_samples":readiness, "started_at":started, "ended_at":Date().timeIntervalSince1970, "duration_monotonic_s":ProcessInfo.processInfo.systemUptime-uptime, "method":"NSWorkspace frontmost PID and on-screen layer-zero window; readiness attempts preserved, then sampling until explicit stop file or 600s limit, serviced RunLoop; no UI actions", "samples":samples, "all_samples_frontmost_visible":samples.allSatisfy(ready)]
let data = try JSONSerialization.data(withJSONObject: value, options: [.prettyPrinted, .sortedKeys])
try data.write(to: URL(fileURLWithPath:target), options: .withoutOverwriting)
print(phase, pid, samples.count, value["all_samples_frontmost_visible"]!)
