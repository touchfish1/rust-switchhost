from PIL import Image, ImageDraw

# 创建 32x32 PNG 图标
img = Image.new('RGBA', (32, 32), (0, 0, 0, 0))
draw = ImageDraw.Draw(img)

# 绘制背景
background_color = (24, 144, 255, 255)  # #1890ff
draw.rectangle([0, 0, 31, 31], fill=background_color)

# 绘制圆角
for i in range(32):
    for j in range(32):
        distance = ((i - 4) ** 2 + (j - 4) ** 2) ** 0.5
        if distance > 4 and (i < 4 or j < 4):
            img.putpixel((i, j), (0, 0, 0, 0))
        distance = ((i - 28) ** 2 + (j - 4) ** 2) ** 0.5
        if distance > 4 and (i > 28 or j < 4):
            img.putpixel((i, j), (0, 0, 0, 0))
        distance = ((i - 4) ** 2 + (j - 28) ** 2) ** 0.5
        if distance > 4 and (i < 4 or j > 28):
            img.putpixel((i, j), (0, 0, 0, 0))
        distance = ((i - 28) ** 2 + (j - 28) ** 2) ** 0.5
        if distance > 4 and (i > 28 or j > 28):
            img.putpixel((i, j), (0, 0, 0, 0))

# 绘制网格线
line_color = (255, 255, 255, 255)

# 水平线
draw.line([(8, 12), (24, 12)], fill=line_color, width=2)
draw.line([(8, 16), (24, 16)], fill=line_color, width=2)
draw.line([(8, 20), (24, 20)], fill=line_color, width=2)

# 垂直线
draw.line([(12, 8), (12, 24)], fill=line_color, width=2)
draw.line([(16, 8), (16, 24)], fill=line_color, width=2)
draw.line([(20, 8), (20, 24)], fill=line_color, width=2)

# 绘制点
draw.ellipse([(22, 6), (26, 10)], fill=line_color)

# 保存图标
img.save('icon.png')
print('Icon created successfully!')