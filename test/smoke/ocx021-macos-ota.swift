// OTA process monitor and single scoped native restart click. Exact owned app only.
import AppKit
import CoreGraphics
import Darwin
import Foundation

func emit(_ object: Any) {
    let bytes = try! JSONSerialization.data(withJSONObject: object, options: [.sortedKeys])
    print(String(data: bytes, encoding: .utf8)!)
}
let args = CommandLine.arguments
guard args.count >= 3 else { emit(["error": "find EXECUTABLE | inspect EXECUTABLE PID"]); exit(2) }
func physical(_ path: String) -> String {
    guard let resolved = realpath(path, nil) else { return path }
    defer { free(resolved) }
    return String(cString: resolved)
}
let expected = physical(args[2])
let ownedPattern = #"^/private/tmp/ocx021-mac-ota-[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}/(?:installed/(?:OpenCodeX Desktop/)?|rollback/)OpenCodeX Desktop\.app/Contents/MacOS/opencodex-desktop$"#
guard expected.range(of: ownedPattern, options: .regularExpression) != nil else {
    emit(["error": "not an owned isolated OTA executable"]); exit(3)
}
func processStart(_ app: NSRunningApplication) -> (Double, String) {
    if let date = app.launchDate { return (date.timeIntervalSince1970, "NSRunningApplication.launchDate") }
    // Directly executed original apps can have no LaunchServices launchDate.
    // Read the exact admitted PID's real OS start time; never use observation
    // time, PID order, or a manually launched replacement as a substitute.
    let process = Process(), output = Pipe()
    process.executableURL = URL(fileURLWithPath: "/bin/ps")
    process.arguments = ["-p", String(app.processIdentifier), "-o", "lstart="]
    process.environment = ["PATH": "/usr/bin:/bin", "LC_ALL": "C", "LANG": "C"]
    process.standardOutput = output
    process.standardError = FileHandle.nullDevice
    do { try process.run() } catch { return (0, "OS start-time read failed") }
    let data = output.fileHandleForReading.readDataToEndOfFile()
    process.waitUntilExit()
    guard process.terminationStatus == 0 else { return (0, "OS start-time read failed") }
    let value = String(decoding: data, as: UTF8.self).trimmingCharacters(in: .whitespacesAndNewlines)
    let format = DateFormatter()
    format.locale = Locale(identifier: "en_US_POSIX")
    format.timeZone = TimeZone.current
    format.dateFormat = "EEE MMM d HH:mm:ss yyyy"
    guard let date = format.date(from: value) else { return (0, "OS start-time parse failed") }
    return (date.timeIntervalSince1970, "ps lstart exact owned PID; seconds resolution")
}
func identity(_ app: NSRunningApplication) -> [String: Any] {
    let (started, source) = processStart(app)
    return ["pid": app.processIdentifier, "pgid": getpgid(app.processIdentifier),
     "executable": physical(app.executableURL!.path),
     "launchTime": started, "launchTimeSource": source,
     "bundleIdentifier": app.bundleIdentifier ?? "", "active": app.isActive]
}
if args[1] == "find" {
    let owned = NSWorkspace.shared.runningApplications.filter {
        $0.executableURL.map { physical($0.path) } == expected
    }
    emit(owned.map(identity)); exit(0)
}
guard ["inspect", "click-restart"].contains(args[1]), args.count >= 4, let pid = pid_t(args[3]), pid > 1, pid != 80916,
      let app = NSRunningApplication(processIdentifier: pid),
      app.executableURL.map({ physical($0.path) }) == expected else {
    emit(["error": "PID/executable admission failed"]); exit(3)
}
if args[1] == "click-restart" {
    guard args.count == 7, let number = UInt32(args[4]), let x = Double(args[5]), let y = Double(args[6]), app.isActive else {
        emit(["error": "exact restart window/point and active owned app required"]); exit(5)
    }
    let session = CGSessionCopyCurrentDictionary() as? [String: Any] ?? [:]
    guard !(session["CGSSessionScreenIsLocked"] as? Bool ?? false) else { emit(["error": "locked"]); exit(5) }
    let point = CGPoint(x: x, y: y)
    let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
    // Admit only this exact foreground PID/window and OCR-verified point.
    // WindowServer must route a mouse event after the old executable's bundle
    // is replaced; posting directly to the PID did not trigger the control.
    let front = windows.first { window in
        guard (window[kCGWindowLayer as String] as? Int) == 0,
              let bounds = window[kCGWindowBounds as String] as? [String: Any],
              let rect = CGRect(dictionaryRepresentation: bounds as CFDictionary) else { return false }
        return rect.contains(point)
    }
    guard let front = front, (front[kCGWindowOwnerPID as String] as? Int) == Int(pid),
          (front[kCGWindowNumber as String] as? NSNumber)?.uint32Value == number else {
        emit(["error": "restart point is occluded or belongs to another window"]); exit(5)
    }
    var posts = [[String: Any]]()
    for type in [CGEventType.mouseMoved, .leftMouseDown, .leftMouseUp] {
        if type == .leftMouseUp { Thread.sleep(forTimeInterval: 0.050) }
        guard let event = CGEvent(mouseEventSource: nil, mouseType: type, mouseCursorPosition: point, mouseButton: .left) else { exit(5) }
        event.setIntegerValueField(.mouseEventClickState, value: 1)
        let before = DispatchTime.now().uptimeNanoseconds
        event.timestamp = before
        event.post(tap: .cghidEventTap)
        posts.append(["type": type.rawValue, "admittedPid": pid, "admittedWindowId": number, "monotonicBeforePostNanoseconds": before,
                      "monotonicAfterPostNanoseconds": DispatchTime.now().uptimeNanoseconds])
    }
    Thread.sleep(forTimeInterval: 0.020)
    emit(["ok": true, "pid": pid, "windowId": number, "point": ["x": x, "y": y], "posts": posts,
          "method": "one WindowServer-routed native mouse click at the exact foreground owned window and OCR-verified restart control; no AX/DOM action"])
    exit(0)
}
// KERN_PROCARGS2 is read only for this exact owned PID. Export only the fixed
// isolation keys; never serialize the whole process environment or other apps.
var mib: [Int32] = [CTL_KERN, KERN_PROCARGS2, pid]
var size = 0
guard sysctl(&mib, u_int(mib.count), nil, &size, nil, 0) == 0, size > 4, size <= 2_097_152 else {
    emit(["error": "owned process environment unavailable"]); exit(4)
}
var bytes = [UInt8](repeating: 0, count: size)
guard sysctl(&mib, u_int(mib.count), &bytes, &size, nil, 0) == 0 else { exit(4) }
let argc = bytes.withUnsafeBytes { $0.loadUnaligned(as: Int32.self) }
var cursor = MemoryLayout<Int32>.size
while cursor < size && bytes[cursor] != 0 { cursor += 1 }
while cursor < size && bytes[cursor] == 0 { cursor += 1 }
for _ in 0..<max(0, argc) {
    while cursor < size && bytes[cursor] != 0 { cursor += 1 }
    cursor += 1
}
let allowed: Set<String> = ["HOME", "CFFIXED_USER_HOME", "TMP", "TMPDIR", "TEMP", "CODEX_HOME", "OPENCODEX_HOME",
    "XDG_CONFIG_HOME", "XDG_CACHE_HOME", "OPENCODEX_SANDBOX", "OPENCODEX_SANDBOX_ROOT", "OPENCODEX_SANDBOX_BOUNDARY"]
var environment: [String: String] = [:]
while cursor < size {
    let start = cursor
    while cursor < size && bytes[cursor] != 0 { cursor += 1 }
    let value = String(decoding: bytes[start..<cursor], as: UTF8.self)
    if let split = value.firstIndex(of: "=") {
        let key = String(value[..<split])
        if allowed.contains(key) { environment[key] = String(value[value.index(after: split)...]) }
    }
    cursor += 1
}
var result = identity(app)
result["environment"] = environment
emit(result)
