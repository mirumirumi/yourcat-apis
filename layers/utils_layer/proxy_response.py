from __future__ import annotations
from typing import TypedDict

import os
import json
import utils

ALLOWED_CLIENT_ORIGIN = os.getenv("ALLOWED_CLIENT_ORIGIN")


class ProxyResponse(TypedDict):
    statusCode: int
    headers: dict[str, str | None]
    body: str

def s200(val: object = None) -> ProxyResponse:
    return {
        "statusCode": 200,
        "headers": {
            "Access-Control-Allow-Origin": ALLOWED_CLIENT_ORIGIN,
        },
        "body": json.dumps(
            val if val is not None else "no response data",
            default=utils.decimal_to_float,
        ),
    }

def s400() -> ProxyResponse:
    return {
        "statusCode": 400,
        "headers": {
            "Access-Control-Allow-Origin": ALLOWED_CLIENT_ORIGIN,
        },
        "body": "required request data is missing or I/F schema is invalid",
    }

def s403() -> ProxyResponse:
    return {
        "statusCode": 403,
        "headers": {
            "Access-Control-Allow-Origin": ALLOWED_CLIENT_ORIGIN,
        },
        "body": "required api key is missing or invalid, alternatively, your referrer is not allowed",
    }

def s500(val: object = None) -> ProxyResponse:
    return {
        "statusCode": 500,
        "headers": {
            "Access-Control-Allow-Origin": ALLOWED_CLIENT_ORIGIN,
        },
        "body": json.dumps(
            val if val is not None else "no response data",
            default=utils.decimal_to_float,
        ),
    }
