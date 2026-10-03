// 用 Vision 对图片做文字识别，输出「[x,y,w,h] 文本」。
// 坐标是图片像素坐标、原点在左上角，便于按文字点击（像素 ÷ DPR = 点）。
import Foundation
import Vision
import AppKit

let args = CommandLine.arguments
guard args.count >= 2 else {
    FileHandle.standardError.write("usage: ocr <image.png>\n".data(using: .utf8)!)
    exit(1)
}
let path = args[1]
guard let data = FileManager.default.contents(atPath: path),
      let rep = NSBitmapImageRep(data: data),
      let cg = rep.cgImage else {
    FileHandle.standardError.write("cannot load image: \(path)\n".data(using: .utf8)!)
    exit(2)
}
let W = CGFloat(cg.width)
let H = CGFloat(cg.height)

let request = VNRecognizeTextRequest()
request.recognitionLevel = .accurate
request.usesLanguageCorrection = false
request.recognitionLanguages = ["zh-Hans", "en-US"]

let handler = VNImageRequestHandler(cgImage: cg, options: [:])
do {
    try handler.perform([request])
} catch {
    FileHandle.standardError.write("vision failed: \(error)\n".data(using: .utf8)!)
    exit(3)
}

let out = NSMutableString()
for observation in (request.results ?? []) {
    guard let candidate = observation.topCandidates(1).first else { continue }
    let box = observation.boundingBox          // 归一化，原点左下
    let x = Int((box.minX * W).rounded())
    let w = Int((box.width * W).rounded())
    let h = Int((box.height * H).rounded())
    let y = Int(((1 - box.maxY) * H).rounded()) // 换到原点左上
    out.append("[\(x),\(y),\(w),\(h)] \(candidate.string)\n")
}
print(out, terminator: "")
