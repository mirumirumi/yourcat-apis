from __future__ import annotations
from typing import Any, cast, Literal, TypedDict

import os
import json
import boto3
import random
from proxy_response import *
from aws_lambda_powertools.logging import Logger
from aws_lambda_powertools.utilities.typing import LambdaContext

logger = Logger()

s3 = boto3.resource("s3")
cache_bucket_name = os.environ["CACHE_BUCKET_NAME"]
key = "scanned_data.json"
obj = s3.Object(cache_bucket_name, key)

SHOW_IMAGE_COUNT = 100


@logger.inject_lambda_context
def lambda_handler(event: dict[str, Any], context: LambdaContext) -> ProxyResponse:
    logger.info(event)

    try:
        res = obj.get()
        images = res["Body"].read().decode()
    except Exception as e:
        logger.exception(e)
        return s500()
    else:
        images = json.loads(images)
        logger.debug(images)
        
    # random pick
    result = random.sample(images, SHOW_IMAGE_COUNT if SHOW_IMAGE_COUNT < len(images) else len(images))
    logger.debug(result)

    return s200(result)
