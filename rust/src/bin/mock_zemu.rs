//! Minimal JSON HTTP stub for zkool's ZEMU transport.
//!
//! This only mocks the transport API. It does not implement any Ledger APDU.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn handle(mut stream: TcpStream, response_data: &str) {
    let mut request = Vec::new();
    let mut buffer = [0; 8192];
    let Ok(size) = stream.read(&mut buffer) else {
        return;
    };
    request.extend_from_slice(&buffer[..size]);
    let request = String::from_utf8_lossy(&request);
    let Some(body) = request.split("\r\n\r\n").nth(1) else {
        return;
    };
    let valid = serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|value| value.get("apduHex")?.as_str().map(str::to_owned))
        .is_some_and(|apdu| !apdu.is_empty() && hex::decode(apdu).is_ok());
    let (status, body) = if valid {
        ("200 OK", format!(r#"{{"data":"{response_data}","error":null}}"#))
    } else {
        ("400 Bad Request", r#"{"error":"expected hexadecimal apduHex"}"#.into())
    };
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
}

fn main() -> std::io::Result<()> {
    let mut args = std::env::args().skip(1);
    let bind = args.next().unwrap_or_else(|| "127.0.0.1:9999".into());
    let response_data = args.next().unwrap_or_else(|| "9000".into());
    if hex::decode(&response_data).is_err() {
        eprintln!("response data must be hexadecimal");
        std::process::exit(2);
    }
    let listener = TcpListener::bind(&bind)?;
    println!("ZEMU mock listening on http://{bind}");
    for stream in listener.incoming().flatten() {
        handle(stream, &response_data);
    }
    Ok(())
}
