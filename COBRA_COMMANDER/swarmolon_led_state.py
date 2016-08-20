
combos = [
    tuple(),
    ("red"),
    ("green"),
    ("blue"),
    ("amber"),
    ("white"),
    ("white", "red"),
    ("red", "green"),
    ("green", "blue"),
    ("blue", "amber"),
    ("amber", "white"),
    ("white", "green"),
    ("green", "amber"),
    ("amber", "red"),
    ("red", "blue"),
    ("blue", "white"),
    ("red", "green", "blue"),
    ("red", "green", "amber"),
    ("red", "green", "white"),
    ("red", "amber", "blue"),
    ("red", "white", "blue"),
    ("red", "amber", "white"),
    ("amber", "green", "blue"),
    ("blue", "green", "white"),
    ("amber", "green", "white"),
    ("amber", "white", "blue"),
    ("red", "green", "blue", "amber"),
    ("red", "green", "blue", "white"),
    ("green", "blue", "amber", "white"),
    ("red", "green", "amber", "white"),
    ("red", "blue", "amber", "white"),
    ("red", "green", "blue", "amber", "white")]

mapping = {}
for i, line in enumerate(combos):
    contents = frozenset(item.strip() for item in line.split(', '))
    dmx_value = 14+(i*5)
    mapping[contents] = dmx_value

