use std::fs::{self, read_to_string};
use std::io::{prelude::*, BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;
use web_server::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let thread_pool = ThreadPool::new(4);
    let mut cnt = 0;
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        cnt += 1;
        println!("{cnt}");
        thread_pool.execute(|| {
            handler_connection(stream);
        });
    }
}

fn handler_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);

    let request_line = buf_reader.lines().next().unwrap().unwrap();
    let (status_line, file_name) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    let content = fs::read_to_string(file_name).unwrap();
    let length = content.len();
    let response = format!("{status_line}\r\n Content-length:{length}\r\n\r\n{content}");
    stream.write_all(response.as_bytes()).unwrap();
}
