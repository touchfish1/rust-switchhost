from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter


ROOT = Path(__file__).resolve().parent
MASTER_SIZE = 512
RADIUS = 112


def lerp(a: int, b: int, t: float) -> int:
    return int(a + (b - a) * t)


def blend(c1: tuple[int, int, int], c2: tuple[int, int, int], t: float) -> tuple[int, int, int]:
    return tuple(lerp(a, b, t) for a, b in zip(c1, c2))


def rounded_mask(size: int, radius: int) -> Image.Image:
    mask = Image.new("L", (size, size), 0)
    draw = ImageDraw.Draw(mask)
    draw.rounded_rectangle((0, 0, size - 1, size - 1), radius=radius, fill=255)
    return mask


def build_gradient(size: int) -> Image.Image:
    image = Image.new("RGBA", (size, size))
    px = image.load()

    top = (15, 23, 42)
    bottom = (13, 148, 136)
    edge = (8, 47, 73)

    for y in range(size):
        v = y / (size - 1)
        vertical = blend(top, bottom, v)
        for x in range(size):
            h = x / (size - 1)
            color = blend(vertical, edge, h * 0.28)
            px[x, y] = (*color, 255)

    return image


def draw_card(draw: ImageDraw.ImageDraw, x0: int, y0: int, x1: int, y1: int, accent: tuple[int, int, int]) -> None:
    draw.rounded_rectangle((x0, y0, x1, y1), radius=42, fill=(248, 250, 252, 238))
    draw.rounded_rectangle((x0, y0, x1, y0 + 28), radius=42, fill=(*accent, 255))
    draw.rectangle((x0, y0 + 14, x1, y0 + 28), fill=(*accent, 255))

    line_left = x0 + 32
    line_right = x1 - 32
    line_y = y0 + 92
    colors = [(148, 163, 184), (100, 116, 139), accent]

    for i, color in enumerate(colors):
        width = 22 if i < 2 else 18
        draw.rounded_rectangle(
            (line_left, line_y + i * 42, line_right - i * 18, line_y + i * 42 + width),
            radius=width // 2,
            fill=(*color, 255),
        )


def draw_switch(draw: ImageDraw.ImageDraw) -> None:
    mint = (45, 212, 191)
    amber = (251, 146, 60)

    # Top arrow: left to right
    draw.rounded_rectangle((194, 196, 336, 236), radius=20, fill=(*mint, 255))
    draw.polygon([(336, 176), (396, 216), (336, 256)], fill=(*mint, 255))

    # Bottom arrow: right to left
    draw.rounded_rectangle((176, 278, 318, 318), radius=20, fill=(*amber, 255))
    draw.polygon([(176, 258), (116, 298), (176, 338)], fill=(*amber, 255))

    # Center badge to keep the symbol crisp against the cards
    draw.rounded_rectangle((176, 156, 336, 356), radius=52, fill=(15, 23, 42, 235))

    draw.rounded_rectangle((196, 196, 316, 232), radius=18, fill=(*mint, 255))
    draw.polygon([(316, 174), (372, 214), (316, 254)], fill=(*mint, 255))

    draw.rounded_rectangle((196, 280, 316, 316), radius=18, fill=(*amber, 255))
    draw.polygon([(196, 258), (140, 298), (196, 338)], fill=(*amber, 255))


def build_master_icon() -> Image.Image:
    base = build_gradient(MASTER_SIZE)
    mask = rounded_mask(MASTER_SIZE, RADIUS)
    icon = Image.new("RGBA", (MASTER_SIZE, MASTER_SIZE), (0, 0, 0, 0))
    icon.paste(base, (0, 0), mask)

    highlight = Image.new("RGBA", (MASTER_SIZE, MASTER_SIZE), (0, 0, 0, 0))
    hdraw = ImageDraw.Draw(highlight)
    hdraw.ellipse((-64, -84, 360, 300), fill=(255, 255, 255, 26))
    hdraw.ellipse((260, 320, 570, 620), fill=(255, 255, 255, 18))
    highlight = highlight.filter(ImageFilter.GaussianBlur(18))
    icon.alpha_composite(highlight)

    shadow = Image.new("RGBA", (MASTER_SIZE, MASTER_SIZE), (0, 0, 0, 0))
    sdraw = ImageDraw.Draw(shadow)
    sdraw.rounded_rectangle((72, 104, 440, 408), radius=56, fill=(0, 0, 0, 70))
    shadow = shadow.filter(ImageFilter.GaussianBlur(18))
    icon.alpha_composite(shadow)

    draw = ImageDraw.Draw(icon)
    draw_card(draw, 72, 96, 242, 392, (34, 197, 94))
    draw_card(draw, 270, 120, 440, 416, (59, 130, 246))
    draw_switch(draw)

    border = Image.new("RGBA", (MASTER_SIZE, MASTER_SIZE), (0, 0, 0, 0))
    bdraw = ImageDraw.Draw(border)
    bdraw.rounded_rectangle(
        (4, 4, MASTER_SIZE - 5, MASTER_SIZE - 5),
        radius=RADIUS,
        outline=(255, 255, 255, 42),
        width=8,
    )
    border.putalpha(mask.filter(ImageFilter.GaussianBlur(1)))
    icon.alpha_composite(border)

    return icon


def save_outputs(icon: Image.Image) -> None:
    icon_32 = icon.resize((32, 32), Image.Resampling.LANCZOS)
    icon_128 = icon.resize((128, 128), Image.Resampling.LANCZOS)
    icon_256 = icon.resize((256, 256), Image.Resampling.LANCZOS)

    icon_32.save(ROOT / "32x32.png")
    icon_128.save(ROOT / "128x128.png")
    icon_256.save(ROOT / "128x128@2x.png")
    icon_256.save(ROOT / "icon.png")
    icon.save(
        ROOT / "icon.ico",
        sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)],
    )


def main() -> None:
    icon = build_master_icon()
    save_outputs(icon)
    print("Updated icon assets in", ROOT)


if __name__ == "__main__":
    main()
