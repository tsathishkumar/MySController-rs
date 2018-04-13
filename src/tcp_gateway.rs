use std::net::TcpListener;

pub fn handle_connection() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    loop {
    match listener.accept() {
        Ok((_socket, addr)) => {
            println!("new client: {:?}", addr)
        },
        Err(e) => println!("couldn't get client: {:?}", e),
    }
    }
}