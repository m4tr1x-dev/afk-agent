//! A minimal HTTP client for one server on loopback.
//!
//! Written directly over a socket rather than pulled in as a dependency. The
//! server is on `127.0.0.1`, there is no transport security to get wrong, and
//! an HTTP client crate is a considerably larger surface than the hundred lines
//! it would replace. The project's supply-chain policy is strict enough that
//! adding a dependency is a decision; this one is not worth making.
//!
//! It speaks exactly as much HTTP as this probe needs: one POST, one GET,
//! `Content-Length` bodies, no chunked transfer, no keep-alive, no redirects.
//! Anything else from the server is an error rather than a silent partial read.

use std::fmt::Write as _;
use std::io::{Read as _, Write as _};
use std::net::TcpStream;
use std::time::Duration;

/// How long to wait for a response.
///
/// A first request loads the vision tower and can take minutes on a cold cache;
/// a request that has genuinely hung should still end the run rather than
/// leaving the probe waiting for a person who is not there.
const TIMEOUT: Duration = Duration::from_secs(600);

/// Send a POST with a JSON body and return the response body.
///
/// # Errors
///
/// Returns a message naming what failed: the connection, the write, the read,
/// or a status line that was not 200.
pub(crate) fn post_json(host: &str, path: &str, body: &str) -> Result<String, String> {
    request(host, "POST", path, Some(body))
}

/// Send a GET and return the response body.
///
/// # Errors
///
/// As [`post_json`].
pub(crate) fn get(host: &str, path: &str) -> Result<String, String> {
    request(host, "GET", path, None)
}

fn request(host: &str, method: &str, path: &str, body: Option<&str>) -> Result<String, String> {
    let mut stream = TcpStream::connect(host).map_err(|e| format!("connect {host}: {e}"))?;
    stream
        .set_read_timeout(Some(TIMEOUT))
        .map_err(|e| format!("read timeout: {e}"))?;
    stream
        .set_write_timeout(Some(TIMEOUT))
        .map_err(|e| format!("write timeout: {e}"))?;

    let mut head = format!(
        "{method} {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\nAccept: application/json\r\n"
    );
    if let Some(body) = body {
        head.push_str("Content-Type: application/json\r\n");
        let _ = write!(head, "Content-Length: {}\r\n", body.len());
    }
    head.push_str("\r\n");

    stream
        .write_all(head.as_bytes())
        .map_err(|e| format!("write head: {e}"))?;
    if let Some(body) = body {
        stream
            .write_all(body.as_bytes())
            .map_err(|e| format!("write body: {e}"))?;
    }
    stream.flush().map_err(|e| format!("flush: {e}"))?;

    let mut raw = Vec::new();
    stream
        .read_to_end(&mut raw)
        .map_err(|e| format!("read: {e}"))?;

    split(&raw)
}

/// Separate the status line and headers from the body.
fn split(raw: &[u8]) -> Result<String, String> {
    let separator = b"\r\n\r\n";
    let position = raw
        .windows(separator.len())
        .position(|window| window == separator)
        .ok_or_else(|| "response has no header terminator".to_owned())?;

    let head = String::from_utf8_lossy(&raw[..position]);
    let status = head
        .lines()
        .next()
        .ok_or_else(|| "response has no status line".to_owned())?;

    // `Connection: close` means the server closes the socket when it is done,
    // so the body is everything left. That is why this client does not have to
    // understand chunked transfer encoding.
    let body = String::from_utf8_lossy(&raw[position + separator.len()..]).into_owned();

    if status.contains(" 200") {
        Ok(body)
    } else {
        Err(format!("{status}: {}", body.trim()))
    }
}

/// Encode bytes as base64, for the image data URI.
///
/// The alphabet is fixed by the standard and the encoder is twenty lines, which
/// is smaller than the argument for adding a crate to do it.
#[must_use]
pub(crate) fn base64(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = u32::from(chunk[0]);
        let b1 = chunk.get(1).copied().map_or(0, u32::from);
        let b2 = chunk.get(2).copied().map_or(0, u32::from);
        let triple = (b0 << 16) | (b1 << 8) | b2;

        out.push(ALPHABET[(triple >> 18 & 63) as usize] as char);
        out.push(ALPHABET[(triple >> 12 & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[(triple >> 6 & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[(triple & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_the_standard_vectors() {
        // From RFC 4648. Padding is where hand-written encoders go wrong, and
        // a data URI with the wrong padding is rejected by the server with a
        // message about the image rather than about the encoding.
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foob"), "Zm9vYg==");
        assert_eq!(base64(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn base64_handles_the_high_bit() {
        assert_eq!(base64(&[0xff, 0xff, 0xff]), "////");
        assert_eq!(base64(&[0x00, 0x00, 0x00]), "AAAA");
    }

    #[test]
    fn a_200_response_yields_its_body() {
        let raw = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"ok\":true}";
        assert_eq!(split(raw).expect("200 parses"), "{\"ok\":true}");
    }

    #[test]
    fn a_non_200_response_is_an_error_carrying_the_body() {
        // The server reports a rejected image as a 400 with a message. Reading
        // that as success and parsing the body as a completion is how a probe
        // reports a confident wrong answer.
        let raw = b"HTTP/1.1 400 Bad Request\r\n\r\n{\"error\":\"bad image\"}";
        let error = split(raw).expect_err("400 is an error");
        assert!(error.contains("400"));
        assert!(error.contains("bad image"));
    }
}
