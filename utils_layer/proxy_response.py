import os
import json
from decimal import Decimal

API_ALLOW_ORIGIN = os.getenv("API_ALLOW_ORIGIN")


def _200(val=None):
    return {
        "statusCode": 200,
        "headers": {
            "Access-Control-Allow-Origin": API_ALLOW_ORIGIN
        },
        "body": json.dumps(
            val if val is not None else "no response data",
            default=decimal_to_float
        )
    }

def _500(val=None):
    return {
        "statusCode": 500,
        "headers": {
            "Access-Control-Allow-Origin": API_ALLOW_ORIGIN
        },
        "body": json.dumps(
            val if val is not None else "no response data",
            default=decimal_to_float
        )
    }

def decimal_to_float(obj):
    if isinstance(obj, Decimal):
        return float(obj)
    raise TypeError
