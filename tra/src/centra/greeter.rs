use super::{Greeter, HelloReply, HelloRequest};
use tonic::{Request, Response, Status};

#[derive(Default, Clone, Copy)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        let name = request.into_inner().name;
        let reply = HelloReply {
            message: format!("Received, hello, {}!", name),
        };
        Ok(Response::new(reply))
    }
}
