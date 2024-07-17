use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

fn main() {
    let http_listener = TcpListener::bind("127.0.0.1:8080").expect("Binding to Port Failed...");
    let pool = rust_http_server::ThreadPool::new(5);

    for stream in http_listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(move || handle_connection(stream))
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect::<Vec<_>>();

    let (requested_file, status) = {
        let mut requested_file = String::from("./public");
        requested_file.push_str(
            http_request[0]
                .split(' ')
                .nth(1)
                .expect("Invalid HTTP Request"),
        );
        if requested_file.ends_with('/') {
            requested_file.push_str("index.html");
        }
        if fs::metadata(&requested_file).is_ok() {
            (requested_file, "HTTP/1.1 200 Ok")
        } else {
            (String::from("./public/404.html"), "HTTP/1.1 404 NOT FOUND")
        }
    };

    match fs::read_to_string(requested_file) {
        Ok(content) => {
            let length = content.len();
            let response = format!("{status}\r\nContent-Length: {length}\r\n\r\n{content}");
            stream.write_all(response.as_bytes()).unwrap();
        }
        Err(_) => {
            let response = String::from("HTTP/1.1 400 INTERNAL SERVER ERROR\r\n\r\r");
            stream.write_all(response.as_bytes()).unwrap();
        }
    };
}
