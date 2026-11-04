// PID-scoped AX/CG helper. No app-name lookup, global events, quit or hide.
import AppKit
import ApplicationServices
import CoreGraphics
import Foundation

func emit(_ value: Any) {
    let data = try! JSONSerialization.data(withJSONObject: value, options: [.sortedKeys])
    print(String(data: data, encoding: .utf8)!)
}
func attr(_ e: AXUIElement, _ name: String) -> CFTypeRef? {
    var value: CFTypeRef?
    return AXUIElementCopyAttributeValue(e, name as CFString, &value) == .success ? value : nil
}
func frame(_ e: AXUIElement) -> [String: Double]? {
    guard let p = attr(e, kAXPositionAttribute), let s = attr(e, kAXSizeAttribute),
          CFGetTypeID(p) == AXValueGetTypeID(), CFGetTypeID(s) == AXValueGetTypeID() else { return nil }
    var point = CGPoint.zero, size = CGSize.zero
    guard AXValueGetValue(p as! AXValue, .cgPoint, &point), AXValueGetValue(s as! AXValue, .cgSize, &size) else { return nil }
    return ["x": point.x, "y": point.y, "width": size.width, "height": size.height]
}
func children(_ e: AXUIElement) -> [AXUIElement] { attr(e, kAXChildrenAttribute) as? [AXUIElement] ?? [] }
func element(_ root: AXUIElement, _ path: String) -> AXUIElement? {
    var e = root
    for part in path.split(separator: ".") {
        guard let i = Int(part), children(e).indices.contains(i) else { return nil }
        e = children(e)[i]
    }
    return e
}
func isWebViewDescendant(_ root: AXUIElement, _ path: String) -> Bool {
    var e = root, inWebView = false
    for part in path.split(separator: ".") {
        guard let i = Int(part), children(e).indices.contains(i) else { return false }
        e = children(e)[i]
        inWebView = inWebView || (attr(e, kAXRoleAttribute) as? String == "AXWebArea")
    }
    return inWebView
}
let args = CommandLine.arguments
guard args.count >= 2 else { emit(["error": "status | snapshot/press/set/key/focus/move/click PID EXPECTED_EXECUTABLE [path/value]"]); exit(2) }
let verb = args[1]
let session = CGSessionCopyCurrentDictionary() as? [String: Any] ?? [:]
if verb == "status" {
    emit(["axTrusted": AXIsProcessTrusted(), "screenCaptureAllowed": CGPreflightScreenCaptureAccess(),
          "locked": session["CGSSessionScreenIsLocked"] ?? false, "onConsole": session["kCGSSessionOnConsoleKey"] ?? false])
    exit(0)
}
guard args.count >= 4, let pid = pid_t(args[2]), pid > 1,
      let app = NSRunningApplication(processIdentifier: pid),
      app.executableURL?.resolvingSymlinksInPath().path == URL(fileURLWithPath: args[3]).resolvingSymlinksInPath().path else {
    emit(["error": "PID/executable admission failed"]); exit(3)
}
let root = AXUIElementCreateApplication(pid)
AXUIElementSetMessagingTimeout(root, 3)
if verb == "snapshot" {
    var nodes = [[String: Any]](), seen = Set<CFHashCode>()
    func walk(_ e: AXUIElement, _ path: String, _ depth: Int) {
        guard depth < 45, nodes.count < 7000, seen.insert(CFHash(e)).inserted else { return }
        // System menus can expose the user's recent documents. Never traverse them.
        if ["AXMenuBar", "AXMenuBarItem", "AXMenu", "AXMenuItem"].contains(attr(e, kAXRoleAttribute) as? String ?? "") { return }
        var n: [String: Any] = ["path": path]
        for (key, name) in [("role", kAXRoleAttribute), ("subrole", kAXSubroleAttribute), ("title", kAXTitleAttribute),
                            ("description", kAXDescriptionAttribute), ("help", kAXHelpAttribute), ("identifier", kAXIdentifierAttribute),
                            ("domId", "AXDOMIdentifier"), ("value", kAXValueAttribute), ("enabled", kAXEnabledAttribute),
                            ("focused", kAXFocusedAttribute), ("hidden", "AXHidden")] {
            if let v = attr(e, name), v is String || v is NSNumber { n[key] = v }
        }
        if let f = frame(e) { n["frame"] = f }
        var actions: CFArray?
        if AXUIElementCopyActionNames(e, &actions) == .success { n["actions"] = actions as? [String] ?? [] }
        nodes.append(n)
        for (i, c) in children(e).enumerated() { walk(c, path.isEmpty ? String(i) : path + "." + String(i), depth + 1) }
    }
    walk(root, "", 0)
    let windows = (CGWindowListCopyWindowInfo([.optionAll], kCGNullWindowID) as? [[String: Any]] ?? []).filter { ($0[kCGWindowOwnerPID as String] as? Int) == Int(pid) }.map { w -> [String: Any] in
        ["id": w[kCGWindowNumber as String] ?? -1, "layer": w[kCGWindowLayer as String] ?? -1,
         "onscreen": w[kCGWindowIsOnscreen as String] ?? false, "bounds": w[kCGWindowBounds as String] ?? [:]]
    }
    emit(["pid": pid, "executable": app.executableURL!.path, "nodes": nodes, "windows": windows,
          "axTrusted": AXIsProcessTrusted(), "active": app.isActive, "hidden": app.isHidden,
          "locked": session["CGSSessionScreenIsLocked"] ?? false])
    exit(0)
}
guard !(session["CGSSessionScreenIsLocked"] as? Bool ?? false), AXIsProcessTrusted() else { emit(["error": "locked or AX permission missing"]); exit(4) }
if verb == "focus" { emit(["ok": app.activate(options: [])]); exit(0) }
if verb == "key" {
    guard args.count >= 5, let code = CGKeyCode(args[4]) else { exit(2) }
    let flags = args.count > 5 && args[5] == "cmd" ? CGEventFlags.maskCommand : []
    var posts = [[String: Any]]()
    for down in [true, false] {
        // Real held key, same target/code/flags. Exactly one down and one up;
        // no event retry, DOM event, focus change or product timeout change.
        if !down { Thread.sleep(forTimeInterval: 0.050) }
        let event = CGEvent(keyboardEventSource: nil, virtualKey: code, keyDown: down)!
        event.flags = flags
        let before = DispatchTime.now().uptimeNanoseconds
        event.postToPid(pid)
        let after = DispatchTime.now().uptimeNanoseconds
        posts.append(["keyDown": down, "targetPid": pid, "keyCode": code, "flags": flags.rawValue,
                      "monotonicBeforePostNanoseconds": before, "monotonicAfterPostNanoseconds": after,
                      "eventTimestampNanoseconds": event.timestamp])
    }
    // Keep the sender alive after release, allowing the native dispatch to drain.
    RunLoop.current.run(until: Date(timeIntervalSinceNow: 0.020))
    Thread.sleep(forTimeInterval: 0.020)
    emit(["ok": true, "posts": posts, "requestedHoldMilliseconds": 50, "postCount": 2]); exit(0)
}
guard args.count >= 5, let e = element(root, args[4]) else { emit(["error": "AX path missing"]); exit(5) }
guard isWebViewDescendant(root, args[4]), attr(e, kAXSubroleAttribute) as? String != "AXCloseButton",
      attr(e, kAXSubroleAttribute) as? String != "AXFullScreenButton",
      attr(e, kAXSubroleAttribute) as? String != "AXMinimizeButton" else {
    emit(["error": "only candidate WebView controls may be mutated"]); exit(7)
}
let labelIndex = (verb == "set" || verb == "type") ? 6 : 5
if args.count > labelIndex {
    let label = args[labelIndex]
    guard attr(e, kAXTitleAttribute) as? String == label || attr(e, kAXDescriptionAttribute) as? String == label else {
        emit(["error": "AX tree changed; expected control label no longer matches"]); exit(7)
    }
}
if verb == "press" {
    let result = AXUIElementPerformAction(e, kAXPressAction as CFString)
    emit(["ok": result == .success, "axError": result.rawValue]); exit(result == .success ? 0 : 6)
}
if verb == "type" {
    guard args.count > 5 else { exit(2) }
    let focused = AXUIElementSetAttributeValue(e, kAXFocusedAttribute as CFString, kCFBooleanTrue)
    guard focused == .success else { emit(["error": "cannot focus text field"]); exit(6) }
    RunLoop.current.run(until: Date(timeIntervalSinceNow: 0.08))
    for down in [true, false] {
        let event = CGEvent(keyboardEventSource: nil, virtualKey: 0, keyDown: down)!
        event.flags = .maskCommand; event.postToPid(pid)
    }
    let units = Array(args[5].utf16)
    for down in [true, false] {
        let event = CGEvent(keyboardEventSource: nil, virtualKey: 0, keyDown: down)!
        event.flags = []
        event.keyboardSetUnicodeString(stringLength: units.count, unicodeString: units)
        event.postToPid(pid)
    }
    RunLoop.current.run(until: Date(timeIntervalSinceNow: 0.08))
    emit(["ok": true]); exit(0)
}
if verb == "set" {
    guard args.count > 5 else { exit(2) }
    _ = AXUIElementSetAttributeValue(e, kAXFocusedAttribute as CFString, kCFBooleanTrue)
    let result = AXUIElementSetAttributeValue(e, kAXValueAttribute as CFString, args[5] as CFString)
    emit(["ok": result == .success, "axError": result.rawValue]); exit(result == .success ? 0 : 6)
}
if verb == "click" || verb == "move" {
    guard let f = frame(e), f["width"]! > 0, f["height"]! > 0 else { exit(5) }
    let point = CGPoint(x: f["x"]! + f["width"]! / 2, y: f["y"]! + f["height"]! / 2)
    // AX hit test must resolve to this candidate before posting even to its PID.
    var hit: AXUIElement?
    guard AXUIElementCopyElementAtPosition(AXUIElementCreateSystemWide(), Float(point.x), Float(point.y), &hit) == .success, let hit = hit else { exit(5) }
    var hitPid: pid_t = 0; AXUIElementGetPid(hit, &hitPid)
    guard hitPid == pid else { emit(["error": "candidate is occluded at target"]); exit(7) }
    let types: [CGEventType] = verb == "click" ? [.mouseMoved, .leftMouseDown, .leftMouseUp] : [.mouseMoved]
    for type in types {
        let event = CGEvent(mouseEventSource: nil, mouseType: type, mouseCursorPosition: point, mouseButton: .left)!
        event.setIntegerValueField(.mouseEventClickState, value: 1); event.postToPid(pid)
    }
    emit(["ok": true]); exit(0)
}
emit(["error": "unknown verb"]); exit(2)
