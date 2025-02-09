use std::sync::Mutex;

use proto::{
    drawing_server::{Drawing, DrawingServer},
    DrawingCanvas, HealthCheckRequest,
};
use tonic::{server::NamedService, transport::Server, Request, Response, Status};

use crate::proto::HealthCheckResponse;

mod proto {
    tonic::include_proto!("drawing");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("drawing_descriptor");
}

mod canvas;

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:7878".parse().unwrap();

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    let service = TestService {
        canon: Mutex::new(canvas::blank()),
    };

    let drawing_server = DrawingServer::new(service);

    Server::builder()
        .add_service(reflection_service)
        .add_service(drawing_server)
        .serve(addr)
        .await
        .unwrap();
}

pub struct TestService {
    canon: Mutex<DrawingCanvas>,
}

#[tonic::async_trait]
impl Drawing for TestService {
    async fn upload_canvas(
        &self,
        uploaded: Request<DrawingCanvas>,
    ) -> Result<Response<DrawingCanvas>, Status> {
        let canvas = uploaded.into_inner();

        let (_, new) = {
            let lock = self.canon.lock().unwrap();

            canvas::merge(&lock, &canvas)
        };

        let clamped = canvas::clamp(&new);

        Ok(Response::new(clamped))
    }

    async fn health_check(
        &self,
        request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        println!("Got a request: {:?}", request.metadata());
        Ok(Response::new(HealthCheckResponse {
            status: "OK".to_string(),
        }))
    }
}

impl NamedService for TestService {
    const NAME: &'static str = "test";
}
