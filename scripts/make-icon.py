"""Иконка Larpscord v2: жирная «L» с сине-фиолетовым градиентом и свечением.
Заметная на рабочем столе, современный стеклянный стиль.
"""
from PIL import Image, ImageDraw, ImageFilter
import math, os

OUT = os.path.join(os.path.dirname(__file__), "..", "src-tauri", "icons")
os.makedirs(OUT, exist_ok=True)

def lerp(a, b, t):
    return tuple(int(a[i] + (b[i] - a[i]) * t) for i in range(3))

def base_icon(size: int) -> Image.Image:
    q = 4
    S = size * q
    img = Image.new("RGBA", (S, S), (0, 0, 0, 0))

    # ---- Фон: глубокий тёмный градиент (диагональ) ----
    bg = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    grad = Image.new("L", (S, S))
    px = grad.load()
    top = (26, 24, 46)     # тёмно-фиолетовый
    bot = (10, 12, 20)     # почти чёрный
    for y in range(S):
        for x in range(S):
            t = (x / S * 0.35) + (y / S * 0.65)
            r, g, b = lerp(top, bot, t)
            px[x, y] = int(255 * t)  # маска: t=0 -> top, t=1 -> bot
    dark = Image.new("RGBA", (S, S), bot + (255,))
    light = Image.new("RGBA", (S, S), top + (255,))
    # Композиция: light вверху-слева, dark внизу-справа
    bg = Image.composite(dark, light, grad)

    # ---- Скруглённый квадрат ----
    mask = Image.new("L", (S, S), 0)
    d = ImageDraw.Draw(mask)
    radius = int(S * 0.23)
    d.rounded_rectangle([0, 0, S - 1, S - 1], radius=radius, fill=255)
    img.paste(bg, (0, 0), mask)

    # ---- Жирная буква «L» ----
    # Толстая геометрия: вертикаль + широкая полка снизу.
    stroke = S * 0.185
    x0 = S * 0.30
    y0 = S * 0.22
    y1 = S * 0.78
    poly = [
        (x0, y0),
        (x0 + stroke, y0),
        (x0 + stroke, y1 - stroke * 1.15),
        (x0 + stroke * 2.55, y1 - stroke * 1.15),
        (x0 + stroke * 2.55, y1),
        (x0, y1),
    ]
    # Градиент буквы: яркий сине-фиолетовый (сверху #7EA6FF → снизу #A78BFA).
    letter_mask = Image.new("L", (S, S), 0)
    ld = ImageDraw.Draw(letter_mask)
    ld.polygon(poly, fill=255)

    lgrad = Image.new("L", (S, S))
    lp = lgrad.load()
    c_top, c_bot = (126, 166, 255), (167, 139, 250)
    for y in range(S):
        t = y / S
        lp_col = int(255 * t)
        for x in range(S):
            lp[x, y] = lp_col
    top_img = Image.new("RGBA", (S, S), c_top + (255,))
    bot_img = Image.new("RGBA", (S, S), c_bot + (255,))
    letter = Image.composite(bot_img, top_img, lgrad)

    # Свечение буквы (glow) — сначала светящееся пятно под ней.
    glow_mask = letter_mask.filter(ImageFilter.GaussianBlur(int(S * 0.035)))
    glow = Image.new("RGBA", (S, S), (140, 170, 255, 0))
    glow.putalpha(glow_mask.point(lambda v: int(v * 0.55)))
    img = Image.alpha_composite(img, glow)

    # Сама буква + тонкая белая окантовка для читаемости.
    img.paste(letter, (0, 0), letter_mask)
    d2 = ImageDraw.Draw(img)
    d2.polygon(poly, outline=(255, 255, 255, 200), width=max(1, int(S * 0.004)))

    # ---- Тонкая внешняя рамка ----
    d3 = ImageDraw.Draw(img)
    d3.rounded_rectangle(
        [int(S*0.008), int(S*0.008), int(S*0.992), int(S*0.992)],
        radius=int(radius * 0.99),
        outline=(255, 255, 255, 46),
        width=max(1, int(S * 0.005)),
    )

    return img.resize((size, size), Image.LANCZOS)

# PNG всех размеров
sizes = {
    "32x32.png": 32,
    "128x128.png": 128,
    "128x128@2x.png": 256,
    "icon.png": 512,
    "Square30x30Logo.png": 30,
    "Square44x44Logo.png": 44,
    "Square71x71Logo.png": 71,
    "Square89x89Logo.png": 89,
    "Square107x107Logo.png": 107,
    "Square142x142Logo.png": 142,
    "Square150x150Logo.png": 150,
    "Square284x284Logo.png": 284,
    "Square310x310Logo.png": 310,
    "StoreLogo.png": 50,
}
for name, s in sizes.items():
    base_icon(s).save(os.path.join(OUT, name))

base_icon(256).save(
    os.path.join(OUT, "icon.ico"),
    format="ICO",
    sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)],
)
# Иконка для фронтенда/README
base_icon(256).save(os.path.join(OUT, "..", "..", "public", "icon.png"))
print("OK: v2 icons generated")
