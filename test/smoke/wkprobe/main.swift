import Cocoa
import WebKit

final class Handler: NSObject, WKURLSchemeHandler {
  let root: URL
  init(root: URL) { self.root = root }
  func webView(_ webView: WKWebView, start task: WKURLSchemeTask) {
    guard let url = task.request.url else { return }
    var path = url.path
    if path.isEmpty || path == "/" { path = "/index.html" }
    let file = root.appendingPathComponent(path)
    guard let data = try? Data(contentsOf: file) else {
      task.didFailWithError(NSError(domain: "wkprobe", code: 404)); return
    }
    let mime: String
    switch file.pathExtension {
    case "html": mime = "text/html"
    case "js", "mjs": mime = "text/javascript"
    case "css": mime = "text/css"
    case "json": mime = "application/json"
    case "svg": mime = "image/svg+xml"
    default: mime = "application/octet-stream"
    }
    let resp = HTTPURLResponse(url: url, statusCode: 200, httpVersion: "HTTP/1.1", headerFields: ["Content-Type": mime, "Access-Control-Allow-Origin": "*"])!
    task.didReceive(resp); task.didReceive(data); task.didFinish()
  }
  func webView(_ webView: WKWebView, stop task: WKURLSchemeTask) {}
}

let args = CommandLine.arguments
let root = URL(fileURLWithPath: args[1])
let hash = args.count > 2 ? args[2] : "#overview"
let captionText = args.count > 3 ? args[3] : ""
let width = args.count > 4 ? Double(args[4]) ?? 1180 : 1180
let height = args.count > 5 ? Double(args[5]) ?? 760 : 760

let app = NSApplication.shared
app.setActivationPolicy(.accessory)
let cfg = WKWebViewConfiguration()
cfg.setURLSchemeHandler(Handler(root: root), forURLScheme: "app")
let web = WKWebView(frame: NSRect(x: 0, y: 0, width: width, height: height), configuration: cfg)
let win = NSWindow(contentRect: web.frame, styleMask: [.borderless], backing: .buffered, defer: false)
win.contentView = web
win.orderFrontRegardless()
web.load(URLRequest(url: URL(string: "app://local/index.html\(hash)")!))

DispatchQueue.main.asyncAfter(deadline: .now() + 4.0) {
  let cap = captionText.replacingOccurrences(of: "'", with: "\\'")
  let js = """
  (() => {
    const q = s => document.querySelector(s);
    const c = q('.motion-caption');
    if (c && '\(cap)'.length) c.textContent = '\(cap)';
    const cx = el => { if (!el) return null; const b = el.getBoundingClientRect(); return {x:+b.x.toFixed(1), w:+b.width.toFixed(1), h:+b.height.toFixed(1), cx:+(b.x+b.width/2).toFixed(1)}; };
    const amb = q('.motion-ambient'); const as = amb ? getComputedStyle(amb) : null;
    const en = q('.motion-ambient .energy'); const es = en ? getComputedStyle(en) : null;
    const glowFields = {
      glow: cx(amb), cloudA: cx(q('.motion-ambient .cloud-a')), cloudB: cx(q('.motion-ambient .cloud-b')), cloudC: cx(q('.motion-ambient .cloud-c')),
      energy: cx(en), floor: cx(q('.motion-ambient .light-floor')),
      glowStyle: as ? { w: as.width, h: as.height, filter: as.filter, mask: as.maskImage || as.webkitMaskImage, opacity: as.opacity } : null,
      energyStyle: es ? { w: es.width, mask: es.maskImage || es.webkitMaskImage } : null,
    };
    return JSON.stringify({ mode: q('.ov')?.dataset.mode, caption: q('.motion-caption')?.textContent,
      fixed: cx(q('.motion-fixed')), stage: cx(q('.motion-stage')), mark: cx(q('.motion-mark')),
      ident: cx(q('.motion-identity')), main: cx(q('.motion-mainline')), capbox: cx(q('.motion-caption')),
      ...glowFields, identStyle: q('.motion-identity') ? { width: getComputedStyle(q('.motion-identity')).width } : null });
  })()
  """
  web.evaluateJavaScript(js) { result, error in
    if let error = error { print("ERR \(error)") }
    print(result as? String ?? "nil")
    exit(0)
  }
}
app.run()
