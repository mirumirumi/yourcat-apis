package main

import (
	"context"
	"errors"
	"os"

	"github.com/aws/aws-lambda-go/events"
	"github.com/aws/aws-lambda-go/lambda"
	"github.com/aws/aws-lambda-go/lambdacontext"
	"github.com/aws/aws-sdk-go-v2/aws"
	"github.com/aws/aws-sdk-go-v2/service/s3"

	"go.uber.org/zap"
)

var (
	cacheBucketName = os.Getenv("CACHE_BUCKET_NAME")
	fRI             string
	fN              = lambdacontext.FunctionName
)

var logger, _ = zap.NewDevelopment(zap.Fields(zap.String("functionName", fN)))
var log = logger.Sugar()

const SHOW_IMAGE_COUNT = 100

func handler(ctx context.Context, request events.APIGatewayProxyRequest) (events.APIGatewayProxyResponse, error) {
	lc, _ := lambdacontext.FromContext(ctx)
	fRI = lc.AwsRequestID
	log.Infow(fRI, "api request", request)

	key := "scanned_data.json"

	s3c := s3.NewFromConfig(aws.Config{
		Region: "ap-northeast-1",
	})

	res, err := s3c.GetObject(ctx, &s3.GetObjectInput{
		Bucket: &cacheBucketName,
		Key: &key,
	})
	if err != nil {
		log.Errorw(fRI, "failed get object from s3", err)
		return events.APIGatewayProxyResponse{}, errors.New("todo: check what number the status code will be")
	}

	log.Infow(fRI, "s3 object", res)












	result := ""



	return events.APIGatewayProxyResponse{
		Body:       result,
		StatusCode: 200,
	}, nil
}

func main() {
	defer log.Sync()
	lambda.Start(handler)
}
