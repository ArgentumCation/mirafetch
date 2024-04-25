#!/usr/bin/env python3
from __future__ import annotations

import ast
from concurrent.futures import ProcessPoolExecutor, ThreadPoolExecutor
from itertools import chain
from pathlib import Path
from typing import cast

from typing_extensions import override


def filter_presets(x: ast.AST) -> bool:
    return (
        hasattr(x, "target")
        and hasattr(x.target, "id")
        and cast(str, x.target.id) == "PRESETS"
    )


class Sanitize(ast.NodeTransformer):
    @override
    def visit_Call(self, node: ast.Call) -> ast.AST:
        if isinstance(node.func, ast.Attribute) and isinstance(
            node.func.value,
            ast.Call,
        ):
            node.func.value.args += node.args
            node = node.func.value
        _ = self.generic_visit(node)
        if isinstance(node.func, ast.Name) and len(node.args) > 0:
            if len(node.args) == 1:
                return node.args[0]
            return ast.Tuple(node.args)
        return node


def dict_transform(tup: tuple[str, list[str] | tuple[list[str], list[int]]]) -> tuple[str, list[str]]:
    key, value = tup
    if not isinstance(value, tuple):
        return key, value
    with ThreadPoolExecutor() as p:
        return key, list(
            chain.from_iterable(p.map(lambda a: [a[0]] * a[1], zip(*value))),
        )


# if __name__ == "__main__":
def main() -> None:
    with Path(__file__).parent.joinpath("hyfetch", "hyfetch", "presets.py").open() as f:
    # import os
    # with Path(os.getcwd()).joinpath("hyfetch", "hyfetch", "presets.py").open() as f:
        presets_file = f.read()
    presets_ast = ast.parse(presets_file)
    ast_dict_raw = next(filter(filter_presets, ast.walk(presets_ast)))
    flags_tree: ast.Dict = cast(ast.Dict, Sanitize().visit(ast_dict_raw).value)
    string = ast.unparse(flags_tree)
    string = string[string.index("{") :]
    preset_dict: dict[str, list[str] | tuple[list[str], list[int]]] = ast.literal_eval(
        string,
    )
    with ProcessPoolExecutor() as p:
        processed_presets = dict(p.map(dict_transform, preset_dict.items()))
    from tomlkit import dumps
    print(dumps(processed_presets))
main()
