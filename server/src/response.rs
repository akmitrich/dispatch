pub struct Response;
impl Response {
    pub fn new(status_code: u16, status_text: &str, body: &str) -> Vec<u8> {
        format!(
            "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nContent-Type: text/plain\r\n\r\n{}",
            status_code,
            status_text,
            body.len(),
            body
        )
        .as_bytes()
        .to_vec()
    }
}
