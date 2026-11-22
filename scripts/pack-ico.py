#!/usr/bin/env python3
"""Packs the PNG frames scripts/render-icons.swift writes to target/icon-frames
into resources/windows/utter.ico, the icon of utter.exe, its taskbar button,
the Start menu shortcut and the installer.

    swift scripts/render-icons.swift && python3 scripts/pack-ico.py
"""
import os
import struct

root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
frames_dir = os.path.join(root, "target", "icon-frames")
out = os.path.join(root, "resources", "windows", "utter.ico")
sizes = [16, 24, 32, 48, 64, 128, 256]

frames = []
for size in sizes:
    with open(os.path.join(frames_dir, f"{size}.png"), "rb") as f:
        frames.append(f.read())

offset = 6 + 16 * len(sizes)
entries = b""
body = b""
for size, frame in zip(sizes, frames):
    # Width and height 0 mean 256; PNG frames carry their own pixel format.
    entries += struct.pack("<BBBBHHII", size % 256, size % 256, 0, 0, 1, 32, len(frame), offset + len(body))
    body += frame

with open(out, "wb") as f:
    f.write(struct.pack("<HHH", 0, 1, len(sizes)) + entries + body)
print(f"wrote {os.path.relpath(out, root)}")
