import os
import io
import re
import json
import uuid
import base64
from PIL import Image
from aws_lambda_powertools import Logger

# PowerTools
logger = Logger()


def to_jpg(event: dict) -> str:
    # base64 to bytes
    body = json.loads(event)
    temp = re.sub("data:image\/.*?;base64,", "", body["img"])
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


