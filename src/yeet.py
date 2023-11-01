#!/usr/bin/env python3
import re
import io
from pathlib import Path


pathlist = Path("../hyfetch/hyfetch/distros").glob("**/*.py")

art_file_regex = re.compile(
    r"match=.*?['\"](?P<name>.+)['\"].*color='(?P<colors>.*)'.*?r\"{3}(?P<ascii>.*)\"{3}",
    re.MULTILINE | re.DOTALL,
)


def get_names(match) -> list[str]:
    return [
        x.strip()
        for x in match.group("name")
        .replace("'", "")
        .replace('"', "")
        .replace("*", "")
        .split("|")
    ]


def get_colors(match):
    colors = []
    for color_raw in match.group("colors").split():
        # foreground
        if color_raw == "fg":
            color = "- Reset"
        # elif k == "bg":
        #     color = "- Bg"
        # Hex color
        elif len(color_raw) == 9 or len(color_raw) == 7:
            color_raw = color_raw.strip('"')
            color = f"""- !Rgb
      r: {int(color_raw[1:3], 16)}
      g: {int(color_raw[3:5], 16)}
      b: {int(color_raw[5:7], 16)}
"""
        # ANSI Color
        else:
            color = f"- !AnsiValue {int(color_raw)}"
        color = "    " + color
        colors += [color]
    return colors


def convert_to_yaml(name, colors, art, width):
    return f"""- name: {name}
  width: {width}
  colors:
{'\n'.join(colors)}
  art: |-
{art}"""


def get_width(unprocessed_art):
    return max(
        [len(re.sub(r"\$\{c.*?\}", "", line.rstrip())) for line in unprocessed_art]
    )


def process_art(unprocessed_art, width):
    return "\n".join(
        ["    " + line.replace(":", ";").ljust(width) for line in unprocessed_art]
    )


def parse_art(match):
    name = get_names(match)
    colors = get_colors(match)
    unprocessed_art = match.group("ascii").strip().split("\n")
    width = get_width(unprocessed_art)
    art = process_art(unprocessed_art, width)
    return name, colors, width, art


def parse_files():
    data = []
    for path in pathlist:
        # print(path)
        art_file = io.open(path, mode="r", encoding="utf-8").read()

        match = art_file_regex.search(art_file)
        if match is not None:
            names, colors, width, art = parse_art(match)
            print("Parsed", '/'.join(names))
            data += [convert_to_yaml(names, colors, art, width)]
        else:
            print("Ignored", path.name)
    return data


if __name__ == "__main__":
    data = parse_files()

    with open("../data/icons.yaml", "w", encoding="utf-8") as f:
        f.write("\n".join(data))
