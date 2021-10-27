import os
import img
import json
import boto3
import random
import proxy_response
from aws_lambda_powertools import Logger
from boto3.dynamodb.conditions import Key, Attr
from datetime import datetime, timezone, timedelta

# PowerTools
logger = Logger()

# S3
s3 = boto3.resource("s3")
cache_bucket_name = os.getenv("Cache_S3_BUCKET_NAME")
key = "scanned_data.json"
obj = s3.Object(cache_bucket_name, key)

# Constant
SHOW_IMAGE_COUNT = 100


@logger.inject_lambda_context()
def lambda_handler(event, context):
    print("🍊")
    logger.info(event)

    # get scanned data
    try:
        res = obj.get()
        images = res["Body"].read().decode()
    except Exception as e:
        logger.exception(e)
        return proxy_response._500()
    else:
        images = json.loads(images)
        logger.info(images)
        
    # random pick
    result = random.sample(images, SHOW_IMAGE_COUNT if SHOW_IMAGE_COUNT < len(images) else len(images))
    logger.info(result)

    print("🍊")
    return proxy_response._200(result)


