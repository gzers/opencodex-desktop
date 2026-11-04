import Cocoa
import WebKit
class Delegate: NSObject, WKNavigationDelegate {
 func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {print("FINISH",webView.url?.absoluteString ?? "nil");fflush(stdout);exit(0)}
 func webView(_ webView: WKWebView, didFailProvisionalNavigation navigation: WKNavigation!, withError error: Error) {print("FAIL",error as NSError);fflush(stdout);exit(1)}
}
let app=NSApplication.shared
app.setActivationPolicy(.accessory)
let window=NSWindow(contentRect:NSRect(x:0,y:0,width:700,height:500),styleMask:[.titled],backing:.buffered,defer:false)
let config=WKWebViewConfiguration()
let view=WKWebView(frame:window.contentView!.bounds,configuration:config)
let delegate=Delegate();view.navigationDelegate=delegate
window.contentView=view;window.orderFront(nil)
view.load(URLRequest(url:URL(string:CommandLine.arguments[1])!))
DispatchQueue.main.asyncAfter(deadline:.now()+12) {print("TIMEOUT");fflush(stdout);exit(2)}
app.run()
