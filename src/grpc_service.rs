use tonic::{Request, Response, Status};

pub use self::find_me_pls::greeter_server::GreeterServer;
use self::find_me_pls::{greeter_server::Greeter, HelloReply, HelloRequest};

pub mod find_me_pls {
    #![allow(non_snake_case)]

    tonic::include_proto!("find_me_pls");
}

#[derive(Debug, Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>, // Accept request of type HelloRequest
    ) -> tonic::Result<Response<HelloReply>, Status> {
        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name).into(), // We must use .into_inner() as the fields of gRPC requests and responses are private
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}
