from __future__ import annotations
from typing import Any, cast

import os
import boto3
import secret
from TwitterAPI import TwitterAPI
from aws_lambda_powertools.logging import Logger
from aws_lambda_powertools.utilities.typing import LambdaContext

logger = Logger()

ENV_NAME = os.environ["ENV_NAME"]

CONSUMER_KEY = secret.CONSUMER_KEY
CONSUMER_SECRET = secret.CONSUMER_SECRET
ACCESS_TOKEN = secret.ACCESS_TOKEN
ACCESS_TOKEN_SECRET = secret.ACCESS_TOKEN_SECRET

s3 = boto3.resource("s3")
image_bucket_name = os.environ["IMAGE_BUCKET_NAME"]


@logger.inject_lambda_context
def lambda_handler(event: dict[str, Any], context: LambdaContext) -> None:
    logger.debug(event)
    file_id: str = event["file_id"]

    if ENV_NAME != "prd":
        logger.debug("function was exited without tweeting because this is not prd env")
        return

    obj = s3.Object(image_bucket_name, file_id)
    file = obj.get()["Body"].read()

    # https://github.com/geduldig/TwitterAPI
    api = TwitterAPI(CONSUMER_KEY, CONSUMER_SECRET, ACCESS_TOKEN, ACCESS_TOKEN_SECRET)

    try:
        res = api.request(
            "media/upload",
            None,
            {"media": file},
        )
        media_id = cast(str, res.json()["media_id"])
    except Exception as e:
        logger.exception(e)
        return

    tweet_text = "New Nyanko🐱 : "

    try:
        res = api.request(
            "statuses/update",
            {
                "status": tweet_text,
                "media_ids": media_id,
            },
        )
    except Exception as e:
        logger.exception(e)

    return
