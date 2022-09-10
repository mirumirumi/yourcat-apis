from __future__ import annotations
from typing import Any, cast, Literal, TypedDict

import os
import io
import re
import uuid
import base64
from PIL import Image
from aws_lambda_powertools.logging import Logger

logger = Logger()


def to_jpg(input_b64: str) -> str:
    # base64 to bytes
    temp = re.sub("data:image\/.*?;base64,", "", input_b64)
    temp = temp.encode("ascii")
    byte = base64.b64decode(temp)
    logger.info(byte)
    
    # current status in tmp/
    logger.info(os.listdir("/tmp"))

    key = str(uuid.uuid4()) + ".jpg"

    # convert to jpg
    image = Image.open(io.BytesIO(byte)).convert("RGB")
    temp_path = "/tmp/" + key
    image.save(temp_path, format="jpeg")

    return key
