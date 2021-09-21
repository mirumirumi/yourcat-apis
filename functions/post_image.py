import os
import img
import json
import boto3
import proxy_response
from datetime import datetime, timezone, timedelta
from aws_lambda_powertools import Logger

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

# datetime
JST = timezone(timedelta(hours=+9), "JST")
now = datetime.now(JST).isoformat()


@logger.inject_lambda_context()
def lambda_handler(event, context):
    print("🍊")
    logger.info(event)

    key = img.to_jpg(event["body"])
    obj = s3.Object(bucket_name, key)

    # get image count
    try:
        res = image_count_table.get_item(Key={
            "type": "count"
        })
    except Exception as e:
        logger.exception(e)
    else:
        if not res.get("Item") is None:
            item = res["Item"]
            image_count = item["count"]
        else:
            put_initial()
            image_count = 0

    # write image data
    file_id, ext = os.path.splitext(key)
    try:
        res = image_table.put_item(Item={
            "file_id": file_id,
            "extension": ext,
            "increment_num": image_count + 1,
            "timestamp": now
        })
    except Exception as e:
        logger.exception(e)
    else:
        logger.info(res)

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

    # update image count
    try:
        res = image_count_table.put_item(Item={
            "type": "count",
            "count": image_count + 1
        })
    except Exception as e:
        logger.exception(e)
    else:
        logger.info(res)

    print("🍊")
    return proxy_response._200(key)


def put_initial():
    try:
        res = image_count_table.put_item(Item={
            "type": "count",
            "count": 0
        })
    except Exception as e:
        logger.exception(e)
    else:
        logger.info(res)


