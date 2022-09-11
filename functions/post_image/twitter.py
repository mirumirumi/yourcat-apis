from __future__ import annotations
from typing import Any, cast, Literal, TypedDict

import os
import secret
from TwitterAPI import TwitterAPI
from aws_lambda_powertools.logging import Logger

logger = Logger()

ENV = os.environ["ENV_NAME"]

CONSUMER_KEY = secret.CONSUMER_KEY
CONSUMER_SECRET = secret.CONSUMER_SECRET
ACCESS_TOKEN = secret.ACCESS_TOKEN
ACCESS_TOKEN_SECRET = secret.ACCESS_TOKEN_SECRET


def tweet(file: bytes) -> None:
    if ENV != "prd":
        logger.info("function was exited without tweeting because this is not prd env")
        return

    # https://github.com/geduldig/TwitterAPI
    api = TwitterAPI(CONSUMER_KEY, CONSUMER_SECRET, ACCESS_TOKEN, ACCESS_TOKEN_SECRET)

    try:
        res = api.request(
            "media/upload",
            None,
            {"media": file},
        )
        media_id: str = res.json()["media_id"]
    except Exception as e:
        logger.exception(e)
        return

    tweet_text = "New Nyanko🐱 : "

    try:
        res = api.request(
            "statuses/update", {
                "status": tweet_text,
                "media_ids": media_id,
            }
        )
    except Exception as e:
        logger.exception(e)
    else:
        logger.debug(cast(str, res.text))

    return
