use std::thread;

mod client;
mod debugger;
mod server;

fn main() {
    let server_thread = thread::spawn(|| {
        server::start_server().expect("Server failed");
    });

    // sleep for a second to give the server time to start
    thread::sleep(std::time::Duration::from_secs(1));

    let client_thread = thread::spawn(|| {
        client::start_client().expect("Client failed");
    });

    server_thread.join().expect("Server thread panicked");
    client_thread.join().expect("Client thread panicked");
}
