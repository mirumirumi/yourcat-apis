from __future__ import annotations
from typing import Any, cast, Literal, TypedDict

import os
import json
import uuid
import boto3
import twitter
from utils import *
from proxy_response import *
from datetime import datetime, timezone, timedelta
from aws_lambda_typing import events
from aws_lambda_powertools.logging import Logger
from aws_lambda_powertools.utilities.typing import LambdaContext

logger = Logger()

s3 = boto3.resource("s3")
image_bucket_name = os.environ["IMAGE_BUCKET_NAME"]
cache_bucket_name = os.environ["CACHE_BUCKET_NAME"]

image_table = boto3.resource("dynamodb").Table(os.environ["IMAGE_TABLE_NAME"])

JST = timezone(timedelta(hours=+9), "JST")


@logger.inject_lambda_context  # type: ignore
def lambda_handler(event: events.APIGatewayProxyEventV1, context: LambdaContext) -> ProxyResponse:
    logger.debug(event)

    body = json.loads(event["body"])
    key = str(uuid.uuid4())
    ext = "jpg"

    save_img_into_lambda(
        input_b64=body["image_data"]["base64"],
        file_name=key,
        want_ext=ext,
    )

    key = key + "." + ext
    obj = s3.Object(image_bucket_name, key)

    # write image data
    file_id, ext = os.path.splitext(key)
    now = datetime.now(JST).isoformat()
    width = body["image_data"]["width"]
    height = body["image_data"]["height"]
    try:
        res = image_table.put_item(Item={
            "file_id": file_id,
            "extension": ext,
            "size": {
                "width": width,
                "height": height,
            },
            "timestamp": now,
        })
    except Exception as e:
        logger.exception(e)
        return s500()

    # file save to S3
    with open("/tmp/" + key, mode="rb") as f:
        file = f.read()
        try:
            res = obj.put(Body=file, ContentType="image/jpeg", CacheControl="no-store")
        except Exception as e:
            logger.exception(e)
            return s500()

        # Twitter bot
        twitter.tweet(file)

    # delete file in this runtime
    os.remove("/tmp/" + key)

    # all scan for update
    try:
        res = image_table.scan()
        items = res["Items"]
    except Exception as e:
        logger.exception(e)
        return s500()
    else:
        logger.debug(res)

    # put scan file to s3
    key_scanned_data = "scanned_data.json"
    obj_scanned_data = s3.Object(cache_bucket_name, key_scanned_data)
    try:
        res = obj_scanned_data.put(
            Body=json.dumps(items, default=decimal_to_float),
            ContentType="application/json",
        )
    except Exception as e:
        logger.exception(e)
        return s500()

    return s200({
        "file_id": key,
        "size": {
            "width": width,
            "height": height,
        },
    })
