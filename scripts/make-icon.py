"""Генерация иконки Larpscord: все размеры PNG + ICO.

Дизайн: чёрный glass-квадрат со скруглением, тонкая светлая рамка,
монограмма «L» с градиентом от белого к серому.
"""
from PIL import Image, ImageDraw, ImageFilter
import os

OUT = os.path.join(os.path.dirname(__file__), "..", "src-tauri", "icons")
os.makedirs(OUT, exist_ok=True)

def base_icon(size: int) -> Image.Image:
    s = size
    # Рендерим в 4x и уменьшаем для сглаживания.
    q = 4
    S = s * q
    img = Image.new("RGBA", (S, S), (0, 0, 0, 0))

    # Фон: тёмный градиент (диагональный, от #101114 к #08090B).
    bg = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    grad = Image.new("L", (S, S))
    px = grad.load()
    for y in range(S):
        for x in range(S):
            t = (x + y) / (2 * S)
            px[x, y] = int(255 * t)
    dark = Image.new("RGBA", (S, S), (8, 9, 11, 255))
    lighter = Image.new("RGBA", (S, S), (18, 20, 24, 255))
    bg = Image.composite(lighter, dark, grad)

    # Скругление квадрата (squircle-подобное).
    mask = Image.new("L", (S, S), 0)
    d = ImageDraw.Draw(mask)
    radius = int(S * 0.24)
    d.rounded_rectangle([0, 0, S - 1, S - 1], radius=radius, fill=255)
    img.paste(bg, (0, 0), mask)

    # Тонкая светлая рамка (стеклянный кант).
    d = ImageDraw.Draw(img)
    d.rounded_rectangle(
        [int(S*0.012), int(S*0.012), int(S*0.988), int(S*0.988)],
        radius=int(radius * 0.98),
        outline=(245, 245, 245, 36),
        width=max(1, int(S * 0.006)),
    )

    # Монограмма «L» — геометричная, с лёгким вертикальным градиентом.
    # Задаём полигоном: толстая вертикаль + короткая горизонталь снизу.
    pad = S * 0.30
    w = S * 0.115          # толщина штриха
    x0 = S * 0.40
    y0 = pad
    y1 = S - pad
    poly = [
        (x0, y0),
        (x0 + w, y0),
        (x0 + w, y1 - w * 1.35),
        (x0 + w * 2.9, y1 - w * 1.35),
        (x0 + w * 2.9, y1),
        (x0, y1),
    ]
    # Градиент для буквы: белый сверху, чуть серее снизу.
    letter_mask = Image.new("L", (S, S), 0)
    ld = ImageDraw.Draw(letter_mask)
    ld.polygon(poly, fill=255)
    letter_grad = Image.new("L", (S, S))
    lp = letter_grad.load()
    for y in range(S):
        v = int(255 - 40 * (y / S))
        for x in range(S):
            lp[x, y] = v
    white = Image.new("RGBA", (S, S), (255, 255, 255, 255))
    gray = Image.new("RGBA", (S, S), (200, 200, 208, 255))
    letter = Image.composite(white, gray, letter_grad)
    img.paste(letter, (0, 0), letter_mask)

    # Лёгкое внутреннее свечение сверху (стеклянный блик).
    glow = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    gd = ImageDraw.Draw(glow)
    gd.rounded_rectangle([0, 0, S - 1, int(S * 0.5)], radius=radius, fill=(255, 255, 255, 14))
    glow = glow.filter(ImageFilter.GaussianBlur(int(S * 0.05)))
    img = Image.alpha_composite(img, Image.composite(glow, Image.new("RGBA", (S, S), (0, 0, 0, 0)), mask))

    return img.resize((s, s), Image.LANCZOS)

# PNG всех размеров.
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
icons = {}
for name, s in sizes.items():
    ic = base_icon(s)
    ic.save(os.path.join(OUT, name))
    icons[s] = ic

# ICO со всеми размерами.
base_icon(256).save(
    os.path.join(OUT, "icon.ico"),
    format="ICO",
    sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)],
)
print("OK: иконки сгенерированы в", os.path.abspath(OUT))
