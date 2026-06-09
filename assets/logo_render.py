from PIL import Image, ImageDraw, ImageFont

BG_TOP = (11, 16, 32)
BG_BOT = (22, 27, 46)
RUST_L = (222, 165, 132)
RUST_R = (206, 66, 43)
NODE = (90, 107, 140)
GLOW = (255, 217, 160)
CORE = (255, 231, 194)
T_WHITE = (245, 247, 250)
T_RUST = (222, 165, 132)
T_GREY = (139, 151, 176)
T_DIM = (90, 107, 140)

FONT_DIR = "C:/Windows/Fonts/"


def lerp(a, b, t):
    return tuple(round(a[i] + (b[i] - a[i]) * t) for i in range(3))


def font(bold, size):
    name = "segoeuib.ttf" if bold else "segoeui.ttf"
    return ImageFont.truetype(FONT_DIR + name, size)


def vgrad(w, h):
    img = Image.new("RGB", (w, h))
    px = img.load()
    for y in range(h):
        c = lerp(BG_TOP, BG_BOT, y / (h - 1))
        for x in range(w):
            px[x, y] = c
    return img


def span(d, y, segs, r, x0, x1):
    for sx, ex in segs:
        x = sx
        while x <= ex:
            c = lerp(RUST_L, RUST_R, (x - x0) / (x1 - x0))
            d.ellipse([x - r, y - r, x + r, y + r], fill=c)
            x += 2


def gaps(d, y, xs):
    for x in xs:
        d.ellipse([x - 7, y - 7, x + 7, y + 7], fill=NODE)


def pulse(d, x, y, rad, core):
    steps = 60
    for i in range(steps, 0, -1):
        rr = rad * i / steps
        t = i / steps
        c = lerp(GLOW, BG_BOT, t)
        d.ellipse([x - rr, y - rr, x + rr, y + rr], fill=c)
    d.ellipse([x - core, y - core, x + core, y + core], fill=CORE)


def render(path, w, h, ty, segs, r, nx, py, prad, pcore, title_sz, t1, t2, t3, lines):
    img = vgrad(w, h)
    d = ImageDraw.Draw(img)
    x0, x1 = segs[0][0], segs[-1][1]
    span(d, ty, segs, r, x0, x1)
    gaps(d, ty, nx)
    pulse(d, py[0], ty, prad, pcore)
    d.text((lines[0][0], t1), "Segovia", font=font(True, title_sz), fill=T_WHITE, anchor="ls")
    d.text((lines[0][0] + 4, t2[0]), lines[1], font=font(False, t2[1]), fill=T_RUST, anchor="ls")
    d.text((lines[0][0] + 4, t3[0]), lines[2], font=font(False, t3[1]), fill=T_GREY, anchor="ls")
    d.text((lines[0][0] + 4, lines[3][0]), lines[4], font=font(False, lines[3][1]), fill=T_DIM, anchor="ls")
    img.save(path)
    print("wrote", path, img.size)


render(
    "C:/Projects/cyber/assets/segovia-cover.png", 1200, 630, 300,
    [(120, 300), (345, 525), (570, 750), (795, 975), (1020, 1090)], 11,
    [322, 547, 772, 997], (547,), 60, 13, 120,
    240, (440, 34), (486, 26),
    [(120,), "A Rust engine for neural data", "Chunked - bounded memory - open source",
     (556, 20), "ELECTROPHYSIOLOGY  -  PyO3  -  SpikeInterface"],
)

render(
    "C:/Projects/cyber/assets/segovia-feed.png", 1080, 1080, 540,
    [(90, 290), (335, 535), (580, 780), (825, 990)], 13,
    [312, 557, 802], (557,), 75, 15, 150,
    380, (730, 42), (790, 32),
    [(90,), "A Rust engine for neural data", "Chunked - bounded memory - open source",
     (980, 24), "ELECTROPHYSIOLOGY  -  PyO3  -  SpikeInterface"],
)

render(
    "C:/Projects/cyber/assets/segovia-social.png", 1280, 640, 305,
    [(128, 320), (368, 560), (608, 800), (848, 1040), (1088, 1163)], 12,
    [344, 583, 824, 1063], (583,), 64, 14, 128,
    244, (447, 36), (494, 28),
    [(128,), "A Rust engine for neural data", "Chunked - bounded memory - open source",
     (565, 21), "ELECTROPHYSIOLOGY  -  PyO3  -  SpikeInterface"],
)
