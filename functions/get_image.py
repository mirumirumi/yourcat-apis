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
bucket_name = os.getenv("Image_S3_BUCKET_NAME")

# DynamoDB
IMAGE_TABLE_NAME = os.getenv("IMAGE_TABLE_NAME")
IMAGE_COUNT_TABLE_NAME = os.getenv("IMAGE_COUNT_TABLE_NAME")
image_table = boto3.resource("dynamodb").Table(IMAGE_TABLE_NAME)
image_count_table = boto3.resource("dynamodb").Table(IMAGE_COUNT_TABLE_NAME)


@logger.inject_lambda_context()
def lambda_handler(event, context):
    print("🍊")
    logger.info(event)

    # get image count
    try:
        res = image_count_table.get_item(Key={
            "type": "count"
        })
    except Exception as e:
        logger.exception(e)
        return proxy_response._500()
    else:
        if not res.get("Item") is None:
            item = res["Item"]
            image_count = item["count"]
            image_count = int(str(image_count))
        else:
            pass  # it shouldn't be

    # random list
    if image_count <= 1000:
        num_list = random.sample(range(1, image_count + 1), image_count)
    else:
        num_list = random.sample(range(1, image_count + 1), 1000)

    # get image urls
    result = []
    i = image_count
    for num in num_list:
        try:
            res = image_table.get_item(Key={
                "increment_num": num
            })
        except Exception as e:
            logger.exception(f"failed get image url at {num} , error is: {e}")
        else:
            if res.get("Item") is None:
                logger.error(f"number: {num} was empty...")
                num_list.extend([i + 1])
                i += 1
            else:
                result.append(res["Item"])

    print("🍊")
    return proxy_response._200(result)


