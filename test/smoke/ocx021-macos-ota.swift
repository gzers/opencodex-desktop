// Read-only OTA process monitor. Match only the exact owned bundle executable.
import AppKit
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
guard expected.hasPrefix("/private/tmp/ocx021-mac-ota-"), expected.contains("/installed/OpenCodeX Desktop.app/Contents/MacOS/") else {
    emit(["error": "not an owned isolated OTA executable"]); exit(3)
}
func identity(_ app: NSRunningApplication) -> [String: Any] {
    ["pid": app.processIdentifier, "pgid": getpgid(app.processIdentifier),
     "executable": physical(app.executableURL!.path),
     "launchTime": app.launchDate?.timeIntervalSince1970 ?? 0,
     "bundleIdentifier": app.bundleIdentifier ?? "", "active": app.isActive]
}
if args[1] == "find" {
    let owned = NSWorkspace.shared.runningApplications.filter {
        $0.executableURL.map { physical($0.path) } == expected
    }
    emit(owned.map(identity)); exit(0)
}
guard args[1] == "inspect", args.count >= 4, let pid = pid_t(args[3]), pid > 1, pid != 80916,
      let app = NSRunningApplication(processIdentifier: pid),
      app.executableURL.map({ physical($0.path) }) == expected else {
    emit(["error": "PID/executable admission failed"]); exit(3)
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
