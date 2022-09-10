from __future__ import annotations
from typing import Any, Literal


def decimal_to_float(obj: object) -> float | Any:
    ...

def save_img_into_lambda(
    input_b64: str,
    file_name: str,
    want_ext: Literal["jpg", "png"],
    size: tuple[int, int] | None = ...,
) -> None:
    ...
