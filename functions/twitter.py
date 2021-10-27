import os
import io
import re
import json
import base64
import secret
from TwitterAPI import TwitterAPI
from aws_lambda_powertools import Logger

# PowerTools
logger = Logger()

# Twitter API
CONSUMER_KEY = secret.CONSUMER_KEY
CONSUMER_SECRET = secret.CONSUMER_SECRET
ACCESS_TOKEN = secret.ACCESS_TOKEN
ACCESS_TOKEN_SECRET = secret.ACCESS_TOKEN_SECRET


def tweet(file):
    # https://github.com/geduldig/TwitterAPI
    api = TwitterAPI(CONSUMER_KEY, CONSUMER_SECRET, ACCESS_TOKEN, ACCESS_TOKEN_SECRET)

    try:
        res = api.request(
            "media/upload",
            None,
            {"media": file}
        )
        media_id = res.json()["media_id"]
    except Exception as e:
        logger.exception(e)
        return
    else:
        logger.info(res)

    tweet_text = "New Nyanko🐱 photo have been uploaded: "

    try:
        res = api.request(
            "statuses/update", {
                "status": tweet_text,
                "media_ids": media_id
            }
        )
    except Exception as e:
        logger.exception(e)
    else:
        logger.info(res.text)

    return


