use nestforge::{
    dispatch_grpc_message,
    tonic::{Request, Response, Status},
    GrpcContext, TransportMetadata,
};

use crate::{
    grpc::proto::hello::{
        greeter_server::Greeter, HelloReply, HelloRequest,
    },
    grpc::GrpcPatterns,
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

        let patterns = self.ctx.resolve::<GrpcPatterns>()?;
        let payload = dispatch_grpc_message(
            &self.ctx,
            patterns.registry(),
            "hello.say",
            name,
            TransportMetadata::new().insert("service", "greeter"),
        )
        .await?;
        let message = payload
            .get("message")
            .and_then(|value| value.as_str())
            .ok_or_else(|| Status::internal("missing `message` in microservice payload"))?;
        let reply = HelloReply {
            message: message.to_string(),
        };

        Ok(Response::new(reply))
    }
}
