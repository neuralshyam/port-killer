use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};
use colored::Colorize;

#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub port: u16,
    pub is_open: bool,
    pub latency: Duration,
    pub http_status: Option<String>,
    pub server_header: Option<String>,
    pub html_title: Option<String>,
    pub error: Option<String>,
}

pub fn probe_port(port: u16) -> ProbeResult {
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
    let start = Instant::now();

    let mut stream = match TcpStream::connect_timeout(&addr, Duration::from_millis(800)) {
        Ok(s) => s,
        Err(e) => {
            return ProbeResult {
                port,
                is_open: false,
                latency: start.elapsed(),
                http_status: None,
                server_header: None,
                html_title: None,
                error: Some(e.to_string()),
            };
        }
    };

    let _ = stream.set_read_timeout(Some(Duration::from_millis(1000)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(500)));

    let request = format!(
        "GET / HTTP/1.1\r\nHost: localhost:{}\r\nUser-Agent: kport/{}\r\nAccept: */*\r\nConnection: close\r\n\r\n",
        port,
        env!("CARGO_PKG_VERSION")
    );

    if stream.write_all(request.as_bytes()).is_err() {
        return ProbeResult {
            port,
            is_open: true,
            latency: start.elapsed(),
            http_status: None,
            server_header: None,
            html_title: None,
            error: None,
        };
    }

    let mut buffer = [0u8; 4096];
    let mut response = Vec::new();
    if let Ok(n) = stream.read(&mut buffer) {
        if n > 0 {
            response.extend_from_slice(&buffer[..n]);
        }
    }

    let latency = start.elapsed();
    let resp_str = String::from_utf8_lossy(&response);

    let (http_status, server_header, html_title) = if resp_str.starts_with("HTTP/") {
        let status = resp_str.lines().next().map(|s| s.to_string());
        let mut server = None;
        for line in resp_str.lines() {
            let lower = line.to_lowercase();
            if lower.starts_with("server:") {
                server = Some(line["server:".len()..].trim().to_string());
                break;
            }
        }

        let title = extract_html_title(&resp_str);
        (status, server, title)
    } else {
        (None, None, None)
    };

    ProbeResult {
        port,
        is_open: true,
        latency,
        http_status,
        server_header,
        html_title,
        error: None,
    }
}

pub fn extract_html_title(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    if let Some(start) = lower.find("<title>") {
        let content_start = start + 7;
        if let Some(end) = lower[content_start..].find("</title>") {
            return Some(html[content_start..content_start + end].trim().to_string());
        }
    }
    None
}

pub fn print_probe_result(res: &ProbeResult) {
    println!("\n🔍 {} Port {}", "PROBE REPORT:".cyan().bold(), res.port.to_string().yellow().bold());
    println!("{}", "─".repeat(50).dimmed());

    if !res.is_open {
        println!("  Status:   {}", "CLOSED / UNREACHABLE".red().bold());
        if let Some(ref err) = res.error {
            println!("  Reason:   {}", err.dimmed());
        }
        println!();
        return;
    }

    println!("  TCP Sockets:  {}", "OPEN (Active Listener)".green().bold());
    println!("  Latency:      {}", format!("{:.2} ms", res.latency.as_secs_f64() * 1000.0).cyan());

    if let Some(ref status) = res.http_status {
        println!("  HTTP Status:  {}", status.green().bold());
    } else {
        println!("  Protocol:     {}", "Raw TCP / Non-HTTP service".yellow());
    }

    if let Some(ref server) = res.server_header {
        println!("  Server:       {}", server.white());
    }

    if let Some(ref title) = res.html_title {
        println!("  HTML Title:   \"{}\"", title.cyan().italic());
    }

    println!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn test_extract_html_title() {
        let html = "<html><head><title>My Awesome App</title></head><body></body></html>";
        assert_eq!(extract_html_title(html), Some("My Awesome App".to_string()));

        let no_title = "<html><body>Hello</body></html>";
        assert_eq!(extract_html_title(no_title), None);
    }

    #[test]
    fn test_live_probe_mock_server() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let response = "HTTP/1.1 200 OK\r\nServer: MockServer/1.0\r\nContent-Type: text/html\r\n\r\n<html><head><title>Test Service</title></head></html>";
                let _ = stream.write_all(response.as_bytes());
            }
        });

        let res = probe_port(port);
        assert!(res.is_open);
        assert_eq!(res.http_status, Some("HTTP/1.1 200 OK".to_string()));
        assert_eq!(res.server_header, Some("MockServer/1.0".to_string()));
        assert_eq!(res.html_title, Some("Test Service".to_string()));
    }
}
