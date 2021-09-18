import os
import io
import re
import json
import uuid
import boto3
import base64
import proxy_response
from PIL import Image
from aws_lambda_powertools import Logger

# PowerTools
logger = Logger()

# S3
s3 = boto3.resource("s3")
bucket_name = os.getenv("Image_S3_BUCKET_NAME")



@logger.inject_lambda_context()
def lambda_handler(event, context):
    print("🍊")
    logger.info(event)

    key = str(uuid.uuid4()) + ".jpg"
    obj = s3.Object(bucket_name, key)

    body = json.loads(event["body"])
    temp = re.sub("data:image\/.*?;base64,", "", body["img"])
    temp = temp.encode("ascii")
    byte = base64.b64decode(temp)
    logger.info(byte)
    
    # current status in tmp/
    logger.info(os.listdir("/tmp"))

    # convert to jpg
    image = Image.open(io.BytesIO(byte)).convert("RGB")
    temp_path = "/tmp/" + key
    image.save(temp_path, format="jpeg")

    # file save to S3
    with open(temp_path, mode="rb") as file:
        file = file.read()
        logger.info(file)
        try:
            res = obj.put(Body=file)
        except Exception as e:
            logger.exception(e)
            return proxy_response._500()
        else:
            logger.info(res)

    # delete file in this runtime
    os.remove(temp_path)

    print("🍊")
    return proxy_response._200(key)


