import json

def _200(val=None):
    return {
        "statusCode": 200,
        "headers": {
            "Access-Control-Allow-Origin": "*"
        },
        "body": json.dumps(
            val if val is not None else "no response data"
        )
    }

def _500(val=None):
    return {
        "statusCode": 500,
        "headers": {
            "Access-Control-Allow-Origin": "*"
        },
        "body": json.dumps(
            val if val is not None else "no response data"
        )
    }
