#!/usr/bin/env python3
"""Generate ZeroApi app icons (PNG multi-size + .ico)"""
from PIL import Image, ImageDraw
import os
import io

PRIMARY_LIGHT = (159, 217, 187)
PRIMARY_MID = (127, 200, 169)
PRIMARY_DARK = (79, 156, 123)

OUT_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "src-tauri", "icons")
OUT_DIR = os.path.normpath(OUT_DIR)
os.makedirs(OUT_DIR, exist_ok=True)


def lerp_color(c1, c2, t):
    return tuple(int(c1[i] + (c2[i] - c1[i]) * t) for i in range(3))


def gradient_bg(size):
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    px = img.load()
    radius = int(size * 0.22)
    for y in range(size):
        for x in range(size):
            dx, dy = 0, 0
            if x < radius and y < radius:
                dx, dy = radius - x, radius - y
            elif x >= size - radius and y < radius:
                dx, dy = x - (size - radius - 1), radius - y
            elif x < radius and y >= size - radius:
                dx, dy = radius - x, y - (size - radius - 1)
            elif x >= size - radius and y >= size - radius:
                dx, dy = x - (size - radius - 1), y - (size - radius - 1)
            if dx * dx + dy * dy > radius * radius:
                continue
            t = (x + y) / (2 * size)
            t = max(0, min(1, t))
            c = lerp_color(PRIMARY_LIGHT, PRIMARY_DARK, t)
            px[x, y] = c + (255,)
    return img


def draw_logo(base, size):
    overlay = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(overlay)
    pad = int(size * 0.18)
    sw = max(int(size * 0.16), 2)
    h = int(size * 0.7)
    y_top = (size - h) // 2 - int(size * 0.05)
    y_bot = y_top + h
    draw.rectangle([pad, y_top, size - pad, y_top + sw], fill=(255, 255, 255, 255))
    draw.rectangle([pad, y_bot - sw, size - pad, y_bot], fill=(255, 255, 255, 255))
    poly = [
        (size - pad - sw, y_top),
        (size - pad, y_top),
        (pad + sw, y_bot),
        (pad, y_bot),
    ]
    draw.polygon(poly, fill=(255, 255, 255, 255))
    api_cx = int(size * 0.85)
    api_cy = int(size * 0.5)
    r = max(int(size * 0.06), 2)
    draw.ellipse([api_cx - r, api_cy - r, api_cx + r, api_cy + r], fill=(255, 255, 255, 255))
    line_len = int(size * 0.10)
    draw.line([(api_cx - line_len, api_cy), (api_cx - r, api_cy)], fill=(255, 255, 255, 255), width=max(2, int(size * 0.025)))
    base.alpha_composite(overlay)


def gen_png(size, name):
    img = gradient_bg(size)
    draw_logo(img, size)
    path = os.path.join(OUT_DIR, name)
    img.save(path, "PNG")
    print(f"  · {name} ({size}x{size})")


def gen_ico():
    """Generate proper multi-size .ico file"""
    sizes = [16, 24, 32, 48, 64, 128, 256]
    # Build ICO file manually for guaranteed correctness
    ico_path = os.path.join(OUT_DIR, "icon.ico")
    png_data = []
    for s in sizes:
        img = gradient_bg(s)
        draw_logo(img, s)
        buf = io.BytesIO()
        img.save(buf, "PNG")
        png_data.append((s, buf.getvalue()))

    # ICO header: 6 bytes (reserved, type=1, count)
    # Each entry: 16 bytes (width, height, ncolors, reserved, planes, bpp, size, offset)
    header = b"\x00\x00\x01\x00" + struct.pack("<H", len(sizes))
    offset = 6 + 16 * len(sizes)
    entries = b""
    images = b""
    for s, data in png_data:
        w = 0 if s == 256 else s  # ICO uses 0 for 256
        h = 0 if s == 256 else s
        entries += struct.pack(
            "<BBBBHHII",
            w, h, 0, 0, 1, 32,
            len(data), offset,
        )
        images += data
        offset += len(data)
    with open(ico_path, "wb") as f:
        f.write(header + entries + images)
    print(f"  · icon.ico ({len(sizes)} sizes: {sizes})")


def gen_icns():
    """Generate .icns for macOS (Pillow supports basic ICNS)"""
    sizes = [(16, "icp4"), (32, "icp5"), (64, "icp6"), (128, "ic07"), (256, "ic08"), (512, "ic09"), (1024, "ic10")]
    icns_path = os.path.join(OUT_DIR, "icon.icns")
    blocks = b""
    for s, code in sizes:
        if code in ("ic09", "ic10"):  # 512+ -> use PNG
            img = gradient_bg(s)
            draw_logo(img, s)
            buf = io.BytesIO()
            img.save(buf, "PNG")
            data = buf.getvalue()
        else:
            img = gradient_bg(s)
            draw_logo(img, s)
            buf = io.BytesIO()
            img.save(buf, "PNG")
            data = buf.getvalue()
        block = code.encode("ascii") + struct.pack(">I", len(data) + 8) + data
        blocks += block
    total_len = 8 + len(blocks)
    with open(icns_path, "wb") as f:
        f.write(b"icns" + struct.pack(">I", total_len) + blocks)
    print(f"  · icon.icns (multi-size)")


import struct

def main():
    print(f"→ 生成图标到 {OUT_DIR}")
    gen_png(32, "32x32.png")
    gen_png(128, "128x128.png")
    gen_png(256, "128x128@2x.png")
    gen_png(512, "icon.png")
    gen_ico()
    gen_icns()
    print("✓ 完成")


if __name__ == "__main__":
    main()
