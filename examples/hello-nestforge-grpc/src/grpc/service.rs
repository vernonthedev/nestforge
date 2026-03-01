use nestforge::{
    tonic::{Request, Response, Status},
    GrpcContext,
};

use crate::{
    app_config::AppConfig,
    grpc::proto::{
        greeter_server::Greeter, HelloReply, HelloRequest,
    },
};

#[derive(Clone)]
pub struct GreeterGrpcService {
    ctx: GrpcContext,
}

impl GreeterGrpcService {
    pub fn new(ctx: GrpcContext) -> Self {
        Self { ctx }
    }
}

#[nestforge::tonic::async_trait]
impl Greeter for GreeterGrpcService {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        let name = request.into_inner().name.trim().to_string();
        if name.is_empty() {
            return Err(Status::invalid_argument("name is required"));
        }

        let config = self.ctx.resolve::<AppConfig>()?;
        let reply = HelloReply {
            message: format!("Hello, {name}! Welcome to {}.", config.app_name),
        };

        Ok(Response::new(reply))
    }
}
