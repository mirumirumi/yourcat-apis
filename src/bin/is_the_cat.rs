use anyhow::Result;
use aws_sdk_rekognition::{
    operation::detect_labels::DetectLabelsOutput, primitives::Blob, types::Image,
};
use lambda_http::{run, Body, Error, Request, RequestExt, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::{fs::File, io::AsyncReadExt};
use tracing::info;
use uuid::Uuid;

mod utils {
    pub mod lambda;
    pub mod responses;
}

use utils::{lambda, responses::*};

const CONFIDENCE_THRESHOLD: f32 = 60.0;

#[derive(Deserialize)]
struct ImageData {
    image_data: ImageDataInfo,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct ImageDataInfo {
    base64: String,
    width: i32,
    height: i32,
}

#[derive(Serialize)]
struct DetectResult {
    judge: bool,
    bounding_box: Option<Vec<BoundingBox>>,
}

#[derive(Serialize)]
struct BoundingBox {
    top: f32,
    left: f32,
    width: f32,
    height: f32,
}

impl DetectResult {
    fn judge_to(&mut self, to: bool) {
        self.judge = to;
    }

    fn set_b_box(&mut self, value: Vec<BoundingBox>) {
        self.bounding_box = Some(value);
    }
}

async fn lambda_handler(request: Request) -> Result<Response<Body>, Error> {
    let context = request.lambda_context();
    lambda::log_incoming_event(&request, context);

    let config = aws_config::load_from_env().await;
    let rekognition = aws_sdk_rekognition::Client::new(&config);

    let key = Uuid::new_v4().to_string();
    let ext = "jpg";
    let file_name = format!("{}.{}", key, ext);

    let payload = match request.body() {
        Body::Text(text) => text.clone(),
        _ => panic!("The image data is not included in request."),
    };

    let image_data: ImageData = serde_json::from_str(&payload)?;

    lambda::save_image_in_temp(
        &image_data.image_data.base64,
        &key,
        match ext {
            "jpg" => lambda::Extension::JPG,
            "jpeg" => lambda::Extension::JPG,
            "png" => lambda::Extension::PNG,
            _ => panic!("Invalid image file extension input."),
        },
        None,
    )?;

    let mut file = File::open(format!("/tmp/{}", file_name))
        .await
        .expect("Failed to open the image file in `/tmp/`.");

    let res = rekognition
        .detect_labels()
        .image(
            Image::builder()
                .bytes(file_to_blob(&mut file).await?)
                .build(),
        )
        .send()
        .await?;

    // Don't remove file in `/tmp/` because the file name is UUID which guarantees uniqueness enough.

    let result = json!({
        "cat": extract_label_data(&res, "Cat")?,
        "dog": extract_label_data(&res, "Dog")?,
    })
    .to_string();

    info!(result);
    _200(result)
}

async fn file_to_blob(file: &mut File) -> Result<Blob, std::io::Error> {
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;

    Ok(Blob::new(buffer))
}

fn extract_label_data(label_data: &DetectLabelsOutput, name: &str) -> Result<DetectResult> {
    let mut result = DetectResult {
        judge: false,
        bounding_box: None,
    };

    let labels = match label_data.labels() {
        Some(label) => label,
        None => return Ok(result),
    };

    for label in labels.iter() {
        if label.name().unwrap() == name {
            if CONFIDENCE_THRESHOLD < label.confidence().unwrap() {
                result.judge_to(true);

                let mut bounding_boxes = vec![];
                for instance in label.instances().unwrap().iter() {
                    bounding_boxes.push(BoundingBox {
                        top: instance.bounding_box().unwrap().top().unwrap(),
                        left: instance.bounding_box().unwrap().left().unwrap(),
                        width: instance.bounding_box().unwrap().width().unwrap(),
                        height: instance.bounding_box().unwrap().height().unwrap(),
                    })
                }

                result.set_b_box(bounding_boxes);
            }
        }
    }

    Ok(result)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(lambda::init_app(lambda_handler)).await
}
