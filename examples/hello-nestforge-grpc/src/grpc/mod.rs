pub mod proto {
    pub mod hello {
        nestforge::tonic::include_proto!("hello");
    }
}

pub mod service;
