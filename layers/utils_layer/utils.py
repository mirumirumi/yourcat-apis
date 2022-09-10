from __future__ import annotations
from typing import Literal, NoReturn


from decimal import Decimal

def decimal_to_float(obj: object) -> float | NoReturn:
    if isinstance(obj, Decimal): return float(obj)
    raise TypeError


import io
import re
import base64
from PIL import Image

def save_img_into_lambda(
    input_b64: str,
    file_name: str,
    want_ext: Literal["jpg", "png"],
    size: tuple[int, int] | None = None,
) -> None:
    regexp = re.compile(r"data:image/(.*?);base64,")

    input_ext = re.sub(regexp, r"\1", input_b64)
    _ = re.sub(regexp, "", input_b64).encode("ascii")
    byte = base64.b64decode(_)

    image = Image.open(io.BytesIO(byte))
    if input_ext == want_ext or (input_ext == "jpeg" and want_ext == "jpg"):  # jpg->jpg, png->png
        if size:
            image.resize(size=size)
    elif want_ext == "jpg":
        image = image.convert("RGB")
        if size:
            image.resize(size=size)
    elif want_ext == "png":
        if size:
            image.resize(size=size)
    else:
        # most likely filtered on the front-end
        raise Exception("it was an unexpected file extension")
    temp_path = "/tmp/" + file_name + "." + want_ext
    image.save(temp_path, format=want_ext.replace("jpg", "jpeg"))

    return
