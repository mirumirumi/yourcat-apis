import os
import img
import json
import boto3
import proxy_response
from boto3 import Session
from aws_lambda_powertools import Logger

# PowerTools
logger = Logger()

# Rekognition
session = Session(region_name="ap-northeast-1")
rekognition_client = session.client("rekognition")
CONFIDENCE_THRESHOLD = 60.0


@logger.inject_lambda_context()
def lambda_handler(event, context):
    print("🍊")
    body = json.loads(event["body"])
    logger.info(body)

    key = img.to_jpg(body["image_data"]["base64"])

    # rekognition
    with open("/tmp/" + key, mode="rb") as f:
        file = f.read()
        try:
            res = rekognition_client.detect_labels(Image={
                "Bytes": file
            })
        except Exception as e:
            logger.exception(f"failed detect labels: {e}")
            return proxy_response._500()
        else:
            logger.info(res)

    # delete file in this runtime
    os.remove("/tmp/" + key)

    result = {
        "cat": get_label_data(res, "Cat"),
        "dog": get_label_data(res, "Dog")
    }

    logger.info(result)
    print("🍊")
    return proxy_response._200(result)


def get_label_data(label_datas: dict, name: str) -> dict:
    result = {}
    for label in label_datas["Labels"]:
        if label["Name"] == name:
            if CONFIDENCE_THRESHOLD < label["Confidence"]:
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


