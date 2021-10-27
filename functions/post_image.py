import os
import img
import json
import boto3
import twitter
import proxy_response
from decimal import Decimal
from aws_lambda_powertools import Logger
from datetime import datetime, timezone, timedelta

# PowerTools
logger = Logger()

# S3
s3 = boto3.resource("s3")
image_bucket_name = os.getenv("Image_S3_BUCKET_NAME")
cache_bucket_name = os.getenv("Cache_S3_BUCKET_NAME")

# DynamoDB
IMAGE_TABLE_NAME = os.getenv("IMAGE_TABLE_NAME")
image_table = boto3.resource("dynamodb").Table(IMAGE_TABLE_NAME)

# timezone
JST = timezone(timedelta(hours=+9), "JST")


@logger.inject_lambda_context()
def lambda_handler(event, context):
    print("🍊")
    body = json.loads(event["body"])
    logger.info(body)

    key = img.to_jpg(body["image_data"]["base64"])
    obj = s3.Object(image_bucket_name, key)

    # write image data
    file_id, ext = os.path.splitext(key)
    now = datetime.now(JST).isoformat()
    width = body["image_data"]["width"]
    height = body["image_data"]["height"]
    try:
        res = image_table.put_item(Item={
            # "increment_num": image_count + 1,
            "file_id": file_id,
            "extension": ext,
            "size": {
                "width": width,
                "height": height
            },
            "timestamp": now
        })
    except Exception as e:
        logger.exception(e)
        return proxy_response._500()
    else:
        logger.info(res)

    # file save to S3
    with open("/tmp/" + key, mode="rb") as f:
        file = f.read()
        logger.info(file)
        try:
            res = obj.put(Body=file, ContentType="image/jpeg", CacheControl="no-store")
        except Exception as e:
            logger.exception(e)
            return proxy_response._500()
        else:
            logger.info(res)

        # Twitter bot
        twitter.tweet(file)

    # delete file in this runtime
    os.remove("/tmp/" + key)

    # all scan for update
    try:
        res = image_table.scan()
        items = res["Items"]
        item_count = res["Count"]
    except Exception as e:
        logger.exception(e)
    else:
        logger.info(res)

    # put scan file to s3
    key_scanned_data = "scanned_data.json"
    obj_scanned_data = s3.Object(cache_bucket_name, key_scanned_data)
    try:
        res = obj_scanned_data.put(Body=json.dumps(items, default=decimal_to_float), ContentType="application/json")
    except Exception as e:
        logger.exception(e)
    else:
        logger.info(res)

    print("🍊")
    return proxy_response._200({
        "file_id": key,
        "size": {
            "width": width,
            "height": height
        }
    })


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


def decimal_to_float(obj):
    if isinstance(obj, Decimal):
        return float(obj)
    raise TypeError


