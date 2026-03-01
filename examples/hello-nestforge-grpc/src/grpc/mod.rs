pub mod proto {
    pub mod hello {
        nestforge::tonic::include_proto!("hello");
    }
}

pub mod patterns;
pub mod service;

pub use patterns::GrpcPatterns;
