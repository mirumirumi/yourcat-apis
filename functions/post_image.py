import os
import img
import json
import boto3
import proxy_response
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

    key = img.to_jpg(event["body"])
    obj = s3.Object(bucket_name, key)

    # file save to S3
    with open("/tmp/" + key, mode="rb") as f:
        file = f.read()
        logger.info(file)
        try:
            res = obj.put(Body=file)
        except Exception as e:
            logger.exception(e)
            return proxy_response._500()
        else:
            logger.info(res)

    # delete file in this runtime
    os.remove("/tmp/" + key)

    print("🍊")
    return proxy_response._200(key)


