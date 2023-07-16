#!/usr/bin/env python3
from __future__ import annotations

import logging
import re
from pathlib import Path

COLOR_RGBA_LEN = 9
COLOR_RGB_LEN = 7

ASCII_ART_REGEX = re.compile(
    r"match=\s*?r?['\"]{1,3}(?P<name>.+?)['\"]{1,3},\s*color=['\"]{1,3}(?P<colors>.*?)['\"]{1,3},\s*ascii=r?['\"]{3}(?P<ascii>.*)['\"]{3}",
    re.MULTILINE | re.DOTALL,
)


def get_names(match: re.Match[str]) -> list[str]:
    return [
        x.strip()
        for x in match.group("name")
        .replace("'", "")
        .replace('"', "")
        .replace("*", "")
        .split("|")
    ]


def process_color(color_raw: str) -> str:
    color_raw = color_raw.strip('"').strip("'")
    # foreground
    if color_raw == "fg":
        color = "- Reset"
    elif color_raw == "bg":
        color = "- Bg"
    # Hex color
    elif len(color_raw) == COLOR_RGBA_LEN or len(color_raw) == COLOR_RGB_LEN:
        color_rgb = color_raw.strip('"')
        color = f"""- !Rgb
      r: {int(color_rgb[1:3], 16)}
      g: {int(color_rgb[3:5], 16)}
      b: {int(color_rgb[5:7], 16)}
"""
    # ANSI Color
    else:
        color = f"- !AnsiValue {int(color_raw)}"
    return "    " + color


def get_colors(match: re.Match[str]) -> list[str]:
    return list(map(process_color, match.group("colors").split()))


def convert_to_yaml(name: list[str], colors: list[str], art: str, width: int) -> str:
    return f"""- name: {name}
  width: {width}
  colors:
{chr(10).join(colors)}
  art: |-
{art}"""


def get_width(unprocessed_art: list[str]) -> int:
    return max(
        len(re.sub(r"\$\{c.*?\}", "", line.rstrip())) for line in unprocessed_art
    )


def process_art(unprocessed_art: list[str], width: int) -> str:
    return "\n".join(
        "    " + line.replace(":", ";").ljust(width) for line in unprocessed_art
    )


def parse_art(match: re.Match[str]) -> tuple[list[str], list[str], int, str]:
    name = get_names(match)
    colors = get_colors(match)
    unprocessed_art: list[str] = match.group("ascii").strip().split("\n")
    width = get_width(unprocessed_art)
    art = process_art(unprocessed_art, width)
    return name, colors, width, art


def parse_file(path: Path) -> str| None:
    art_file = path.open(encoding="utf-8").read()

    match = ASCII_ART_REGEX.search(art_file)
    if match is not None:
        names, colors, width, art = parse_art(match)
        logging.info("Parsed %s", "/".join(names))
        return convert_to_yaml(names, colors, art, width)
    logging.warning("Ignored %s", path.name)
    return None


def parse_ascii_art_files() -> list[str]:
    pathlist = (
        Path(__file__).parent.joinpath("hyfetch", "hyfetch", "distros").glob("**/*.py")
    )

    return list(filter(lambda x: x is not None, map(parse_file, pathlist)))


def parse_flags() -> dict[str, tuple[int, int, int]]:
    res: dict[str, tuple[int, int, int]] = {}
    return res


if __name__ == "__main__":
    ascii_art_data = parse_ascii_art_files()
    with Path("../data/icons.yaml").open("w", encoding="utf-8") as f:
        _ = f.write("\n".join(ascii_art_data))
