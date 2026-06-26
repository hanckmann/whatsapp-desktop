#!/usr/bin/env python3
"""Generate placeholder app icons for WhatsApp Desktop (Tauri build requirement).

Run once before building:
    python3 gen_icons.py

Requires Pillow:
    pip install pillow
"""

import os
import subprocess
from PIL import Image, ImageDraw, ImageFont

ICONS_DIR = os.path.join(os.path.dirname(__file__), "src-tauri", "icons")
os.makedirs(ICONS_DIR, exist_ok=True)

# WhatsApp green palette
BG_COLOR = (18, 140, 126, 255)     # dark teal ring
FILL_COLOR = (37, 211, 102, 255)   # bright green body
TEXT_COLOR = (255, 255, 255, 255)

def make_icon(size: int) -> Image.Image:
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    radius = size // 6
    # Background rounded square
    draw.rounded_rectangle([0, 0, size - 1, size - 1], radius=radius, fill=BG_COLOR)
    # Inner fill
    pad = max(2, size // 16)
    draw.rounded_rectangle(
        [pad, pad, size - 1 - pad, size - 1 - pad],
        radius=max(2, radius - pad // 2),
        fill=FILL_COLOR,
    )
    # "W" letter centred
    font = None
    _font_candidates = [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",           # Debian/Ubuntu
        "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",                        # Arch
        "/usr/share/fonts/dejavu-sans-fonts/DejaVuSans-Bold.ttf",          # Fedora
        "/usr/share/fonts/liberation/LiberationSans-Bold.ttf",             # Fedora/RHEL
        "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",    # Debian
        "/usr/share/fonts/truetype/freefont/FreeSansBold.ttf",             # freefont
    ]
    try:
        _fc = subprocess.check_output(
            ["fc-match", "--format=%{file}", "sans:bold"], stderr=subprocess.DEVNULL
        ).decode().strip()
        if _fc:
            _font_candidates.insert(0, _fc)
    except (OSError, subprocess.CalledProcessError):
        pass
    for _path in _font_candidates:
        try:
            font = ImageFont.truetype(_path, size // 2)
            break
        except OSError:
            continue
    if font is None:
        font = ImageFont.load_default()
    text = "W"
    bbox = draw.textbbox((0, 0), text, font=font)
    tw, th = bbox[2] - bbox[0], bbox[3] - bbox[1]
    x = (size - tw) // 2 - bbox[0]
    y = (size - th) // 2 - bbox[1]
    draw.text((x, y), text, fill=TEXT_COLOR, font=font)
    return img


SIZES = {
    "32x32.png": 32,
    "128x128.png": 128,
    "128x128@2x.png": 256,   # high-DPI variant stored as 256 px
}

for filename, px in SIZES.items():
    path = os.path.join(ICONS_DIR, filename)
    make_icon(px).save(path)
    print(f"  created {path}")

print("Icons generated successfully.")
