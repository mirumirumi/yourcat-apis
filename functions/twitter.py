import os
import io
import re
import json
import base64
import secret
from aws_lambda_powertools import Logger
from requests_oauthlib import OAuth1Session

# PowerTools
logger = Logger()

# Twitter API
CONSUMER_KEY = secret.CONSUMER_KEY
CONSUMER_SECRET = secret.CONSUMER_SECRET
ACCESS_TOKEN = secret.ACCESS_TOKEN
ACCESS_TOKEN_SECRET = secret.ACCESS_TOKEN_SECRET
ENDPOINT_UPLOAD = "https://upload.twitter.com/1.1/media/upload.json"
ENDPOINT_TWEET = "https://api.twitter.com/1.1/statuses/update.json"


def tweet(file):
    session = OAuth1Session(CONSUMER_KEY, CONSUMER_SECRET, ACCESS_TOKEN, ACCESS_TOKEN_SECRET)

    try:
        res = sesstion.post(ENDPOINT_UPLOAD, media=file)
        media_id = res["media_id_string"]
    except Exception as e:
        logger.exception(e)
    else:
        logger.info(res)

    tweet_text = f"New Nyanko🐱 photo have been uploaded: \n{media_id}"

    try:
        res = session.post(ENDPOINT_TWEET, params={
            "status": tweet_text
        })
    except Exception as e:
        logger.exception(e)
    else:
        logger.info(res)

    return


