from __future__ import annotations
from typing import Any, cast, Literal, TypedDict

import os
import img
import json
from boto3 import Session
from proxy_response import *
from aws_lambda_powertools.logging import Logger
from aws_lambda_powertools.utilities.typing import LambdaContext
from mypy_boto3_rekognition.type_defs import DetectLabelsResponseTypeDef

logger = Logger()

session = Session(region_name="ap-northeast-1")
rekognition_client = session.client("rekognition")

CONFIDENCE_THRESHOLD = 60.0


@logger.inject_lambda_context
def lambda_handler(event: dict[str, Any], context: LambdaContext) -> ProxyResponse:
    body = json.loads(event["body"])
    logger.info(body)

    key = img.to_jpg(body["image_data"]["base64"])

    # rekognition
    with open("/tmp/" + key, mode="rb") as f:
        file = f.read()
        try:
            res = rekognition_client.detect_labels(Image={
                "Bytes": file,
            })
        except Exception as e:
            logger.exception(f"failed detect labels: {e}")
            return s500()
        else:
            logger.debug(res)

    # delete file in this runtime
    os.remove("/tmp/" + key)

    result = {
        "cat": get_label_data(res, "Cat"),
        "dog": get_label_data(res, "Dog"),
    }

    logger.debug(result)
    return s200(result)


def get_label_data(label_data: DetectLabelsResponseTypeDef, name: str) -> dict[str, Any]:
    result: dict[str, Any] = {}

    for label in label_data["Labels"]:
        # because LabelTypeDef is total=False (from the documentation, it is safe to assume that this is not possible)
        if not "Name" in label \
           or not "Confidence" in label \
           or not "Instances" in label:
            break

        if label["Name"] == name:
            if CONFIDENCE_THRESHOLD < label["Confidence"]:
                result["judge"] = True
                result["bounding_box"] = []
                for bbs in label["Instances"]:
                    # same as above
                    if not "BoundingBox" in bbs \
                       or not "Width" in bbs["BoundingBox"] \
                       or not "Height" in bbs["BoundingBox"] \
                       or not "Left" in bbs["BoundingBox"] \
                       or not "Top" in bbs["BoundingBox"]:
                        break

                    result["bounding_box"].append({
                        "width": bbs["BoundingBox"]["Width"],
                        "height": bbs["BoundingBox"]["Height"],
                        "left": bbs["BoundingBox"]["Left"],
                        "top": bbs["BoundingBox"]["Top"],
                    })
                break
        else:
            result["judge"] = False

    return result
