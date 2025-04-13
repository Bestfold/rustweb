use axum::{
    routing::get,
    Router,
    response::{Html, Response, IntoResponse},
    http::{StatusCode, header, HeaderValue},
};
use std::fs::File;
use std::io::prelude::*;
use tower_http::services::ServeDir;

use aws_config::{self, BehaviorVersion};
use aws_sdk_s3;
use std::env;

use dotenvy::dotenv;


#[tokio::main]
async fn main() {
	dotenv().ok();

    let shared_config = aws_config::defaults(BehaviorVersion::latest())
        .load()
        .await;

    let config = aws_sdk_s3::config::Builder::from(&shared_config)
        .force_path_style(true)
        .build();

	let client = aws_sdk_s3::Client::from_conf(config);

    let buckets_to_set_up = vec![
        env::var("MINIO_IMAGE_BUCKET").expect("MINIO_IMAGE_BUCKET must be set"),
        env::var("MINIO_STYLE_BUCKET").expect("MINIO_STYLE_BUCKET must be set")
    ];

    let existing_buckets = client.list_buckets().send().await.unwrap()
        .buckets.unwrap_or_default();
    let existing_bucket_names: Vec<String> = existing_buckets
        .into_iter()
        .filter_map(|b| b.name)
        .collect();

    for bucket_to_set_up in buckets_to_set_up {
        if existing_bucket_names.contains(&bucket_to_set_up) {
            println!("The {} bucket exists!", &bucket_to_set_up);
        } else {
            println!("The {} bucket sadly does not exist... Creating it...", &bucket_to_set_up);
            let response = client.create_bucket().bucket(&bucket_to_set_up).send().await.unwrap();
            println!("I'll be damned. The bucket was created at {:?}", response.location.unwrap());
            let policy = format!(r#"{{
                "Version": "2012-10-17",
                "Statement": [
                  {{
                    "Effect": "Allow",
                    "Principal": "*",
                    "Action": "s3:GetObject",
                    "Resource": "arn:aws:s3:::{}/*"
                  }}
                ]
              }}"#, &bucket_to_set_up);
            client
                .put_bucket_policy()
                .bucket(&bucket_to_set_up)
                .policy(policy)
                .send().await.unwrap();
            println!("Set public read access on {}", &bucket_to_set_up);
        }
    }

    // build our application with a single route
    let app = Router::new()
        .route("/portfolio", get(|| async { Html("<img src='assets/imgs/crashPad.jpg'>") }))
        //.route("/some_endpoint", get(some_handler))
        //.route("/assets/css/main.css", get(get_css))
        .fallback_service(ServeDir::new("webrepo/DATA1200-Web-Development-and-Accessible-Design-Autumn-2024"));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}





/*
async fn some_handler() -> Html<String> {
    let mut file = File::open("webrepo/DATA1200-Web-Development-and-Accessible-Design-Autumn-2024/index.html").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    Html(contents)
}

async fn get_css() -> Response {
    let mut file = File::open("webrepo/DATA1200-Web-Development-and-Accessible-Design-Autumn-2024/assets/css/main.css").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();


    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, HeaderValue::from_static("text/css"))],
        contents
    ).into_response()
}
*/
// https://docs.rs/axum/latest/axum/response/trait.IntoResponse.html
// https://docs.rs/axum/latest/axum/response/index.html
// http://localhost:3000/assets/css/main.css
// https://tokio.rs/blog/2021-05-14-inventing-the-service-trait