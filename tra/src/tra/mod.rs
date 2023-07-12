use std::thread;

use std::io::Error as IoError;
use std::io::Result as IoResult;

use tonic::transport::Server;
use tonic::{Request, Response, Status};

use crate::hello::greeter_server::{Greeter, GreeterServer};
use crate::hello::{HelloReply, HelloRequest};

type MacThread = thread::JoinHandle<Result<(), IoError>>;

pub struct MacInfo {
    pub thread_handle: MacThread,
    pub port: u16,
}

#[derive(Default)]
struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        let name = request.into_inner().name;
        let reply = HelloReply {
            message: format!("Fuck you, {}!", name),
        };
        Ok(Response::new(reply))
    }
}

pub async fn start_tra() -> IoResult<()> {
    let server = MyGreeter::default();

    Server::builder()
        .add_service(GreeterServer::new(server))
        .serve("[::]:8080".parse().unwrap())
        .await
        .unwrap();

    Ok(())
}
