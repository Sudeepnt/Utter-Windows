// Generates full-colour app icons from assets/icons/utter-logo.png.
// Menu bar and tray symbols use a monochrome microphone mark.
// Run with `swift scripts/render-icons.swift` from the repository root.

import AppKit
import CoreGraphics
import Foundation
import ImageIO
import UniformTypeIdentifiers

let root = URL(fileURLWithPath: CommandLine.arguments.count > 1
    ? CommandLine.arguments[1]
    : FileManager.default.currentDirectoryPath)

/// Draws the mark in a 100×100 box (y down) scaled into `rect`.
func drawMark(_ ctx: CGContext, in rect: CGRect, color: CGColor, dot: Bool = false) {
    ctx.saveGState()
    ctx.translateBy(x: rect.minX, y: rect.minY)
    ctx.scaleBy(x: rect.width / 100, y: rect.height / 100)
    ctx.setStrokeColor(color)
    ctx.setLineCap(.round)
    ctx.setLineJoin(.round)

    // The u: two stems joined by a half circle.
    let u = CGMutablePath()
    u.move(to: CGPoint(x: 24, y: 18))
    u.addLine(to: CGPoint(x: 24, y: 54))
    u.addArc(center: CGPoint(x: 50, y: 54), radius: 26,
             startAngle: .pi, endAngle: 0, clockwise: true)
    u.addLine(to: CGPoint(x: 76, y: 18))
    ctx.setLineWidth(13)
    ctx.addPath(u)
    ctx.strokePath()

    // The level bar inside the bowl.
    ctx.setLineWidth(13)
    ctx.move(to: CGPoint(x: 50, y: 24))
    ctx.addLine(to: CGPoint(x: 50, y: 56))
    ctx.strokePath()

    if dot {
        // A recording dot, cut clear of the mark.
        ctx.setBlendMode(.clear)
        ctx.fillEllipse(in: CGRect(x: 62, y: 62, width: 40, height: 40))
        ctx.setBlendMode(.normal)
        ctx.setFillColor(color)
        ctx.fillEllipse(in: CGRect(x: 69, y: 69, width: 26, height: 26))
    }
    ctx.restoreGState()
}

/// A bitmap context with y pointing down, like the mark's coordinates.
func canvas(_ w: Int, _ h: Int) -> CGContext {
    let ctx = CGContext(data: nil, width: w, height: h, bitsPerComponent: 8, bytesPerRow: w * 4,
                        space: CGColorSpace(name: CGColorSpace.sRGB)!,
                        bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)!
    ctx.translateBy(x: 0, y: CGFloat(h))
    ctx.scaleBy(x: 1, y: -1)
    ctx.interpolationQuality = .high
    ctx.setShouldAntialias(true)
    return ctx
}

func pngData(_ ctx: CGContext) -> Data {
    let data = NSMutableData()
    let dest = CGImageDestinationCreateWithData(data, UTType.png.identifier as CFString, 1, nil)!
    CGImageDestinationAddImage(dest, ctx.makeImage()!, nil)
    CGImageDestinationFinalize(dest)
    return data as Data
}

func write(_ data: Data, _ path: String) {
    let url = root.appendingPathComponent(path)
    try! FileManager.default.createDirectory(at: url.deletingLastPathComponent(),
                                             withIntermediateDirectories: true)
    try! data.write(to: url)
    print("wrote \(path)")
}

/// Use the checked-in brand artwork for every full-colour app icon.
func appIcon(_ size: Int) -> CGContext {
    let ctx = canvas(size, size)
    let url = root.appendingPathComponent("assets/icons/utter-logo.png")
    guard let source = CGImageSourceCreateWithURL(url as CFURL, nil),
          let image = CGImageSourceCreateImageAtIndex(source, 0, nil) else {
        fatalError("Missing brand artwork at \(url.path)")
    }
    ctx.saveGState()
    ctx.translateBy(x: 0, y: CGFloat(size))
    ctx.scaleBy(x: 1, y: -1)
    ctx.draw(image, in: CGRect(x: 0, y: 0, width: size, height: size))
    ctx.restoreGState()
    return ctx
}

func markOnly(_ size: Int, color: CGColor, dot: Bool = false, pad: CGFloat = 0.08) -> CGContext {
    let ctx = canvas(size, size)
    let s = CGFloat(size)
    drawMark(ctx, in: CGRect(x: s * pad, y: s * pad, width: s * (1 - 2 * pad), height: s * (1 - 2 * pad)),
             color: color, dot: dot)
    return ctx
}

/// Straight (not premultiplied) RGBA bytes, as the Windows tray reads them.
func straightRGBA(_ ctx: CGContext) -> Data {
    let bytes = ctx.data!.assumingMemoryBound(to: UInt8.self)
    var out = Data(count: ctx.width * ctx.height * 4)
    for i in 0..<(ctx.width * ctx.height) {
        let a = bytes[i * 4 + 3]
        out[i * 4 + 3] = a
        for c in 0..<3 {
            out[i * 4 + c] = a == 0 ? 0 : UInt8(min(255, Int(bytes[i * 4 + c]) * 255 / Int(a)))
        }
    }
    return out
}

// macOS app artwork and menu bar template images.
write(pngData(appIcon(1024)), "app/AppIcon.icon/Assets/Image.png")
write(pngData(markOnly(36, color: CGColor(gray: 0, alpha: 1), pad: 0.1)), "assets/icons/menubar-idle@2x.png")
write(pngData(markOnly(36, color: CGColor(gray: 0, alpha: 1), dot: true, pad: 0.1)), "assets/icons/menubar-recording@2x.png")

// Windows: window icons, the tray's raw pixels, and the .ico's frames.
for size in [32, 64, 128] {
    write(pngData(appIcon(size)), "resources/windows/icon-\(size).png")
}
write(straightRGBA(markOnly(32, color: CGColor(gray: 1, alpha: 1), pad: 0.06)), "resources/windows/tray-32.rgba")
for size in [16, 24, 32, 48, 64, 128, 256] {
    write(pngData(appIcon(size)), "target/icon-frames/\(size).png")
}

// Linux: the hicolor app icons.
for size in [16, 24, 32, 48, 64, 128, 256, 512] {
    write(pngData(appIcon(size)), "packaging/linux/icons/hicolor/\(size)x\(size)/apps/app.utter.png")
}

// Windows accepts PNG-compressed ICO frames on supported Windows versions.
let sizes = [16, 24, 32, 48, 64, 128, 256]
let frames = sizes.map { pngData(appIcon($0)) }
var ico = Data()
func word(_ value: UInt16) { var v = value.littleEndian; withUnsafeBytes(of: &v) { ico.append(contentsOf: $0) } }
func dword(_ value: UInt32) { var v = value.littleEndian; withUnsafeBytes(of: &v) { ico.append(contentsOf: $0) } }
word(0); word(1); word(UInt16(sizes.count))
var offset = UInt32(6 + 16 * sizes.count)
for (size, frame) in zip(sizes, frames) {
    ico.append(contentsOf: [UInt8(size % 256), UInt8(size % 256), 0, 0])
    word(1); word(32); dword(UInt32(frame.count)); dword(offset)
    offset += UInt32(frame.count)
}
for frame in frames { ico.append(frame) }
write(ico, "resources/windows/utter.ico")
