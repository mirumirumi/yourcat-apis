import re
import json
import boto3
import base64
from boto3 import Session
from aws_lambda_powertools import Logger

# PowerTools
logger = Logger()



@logger.inject_lambda_context()
def lambda_handler(event, context):
    print("🍊")
    logger.info(event)

    session = Session(region_name="ap-northeast-1")
    rekognition_client = session.client("rekognition")

    body = json.loads(event["body"])
    temp = re.sub("data:image\/.*?;base64,", "", body["img"])
    temp = temp.encode("ascii")
    byte = base64.b64decode(temp)
    logger.info(byte)

    try:
        res = rekognition_client.detect_labels(Image={
            "Bytes": byte
        })
    except Exception as e:
        logger.exception(e)
        return {
            "statusCode": 500,
            "headers": {
                "Access-Control-Allow-Origin": "*"
            },
            "body": "failed detect labels"
        }
    else:
        logger.info(res)

    result = {
        "cat": get_label_data(res, "Cat"),
        "dog": get_label_data(res, "Dog")
    }

    logger.info(result)
    print("🍊")
    return  {
        "statusCode": 200,
        "headers": {
            "Access-Control-Allow-Origin": "*"
        },
        "body": json.dumps(result)
    }


def get_label_data(label_datas, name):
    result = {}
    for label in label_datas["Labels"]:
        if label["Name"] == name:
            if 80.0 < label["Confidence"]:
                result["judge"] = True
                result["bounding_box"] = []
                for bbs in label["Instances"]:
                    result["bounding_box"].append({
                        "width": bbs["BoundingBox"]["Width"],
                        "height": bbs["BoundingBox"]["Height"],
                        "left": bbs["BoundingBox"]["Left"],
                        "top": bbs["BoundingBox"]["Top"]
                    })
                break
        else:
            result["judge"] = False
    return result


