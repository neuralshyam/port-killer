use port_killer::model::{PortProcess, Protocol};
use port_killer::pattern::resolve_port_patterns;
use port_killer::probe::probe_port;
use std::net::TcpListener;

#[test]
fn test_integration_port_scanning() {
    let ports = port_killer::scanner::scan_ports();
    // Verify scan returns a valid list without panicking
    for p in &ports {
        assert!(p.port > 0);
    }
}

#[test]
fn test_integration_live_probe() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind ephemeral test port");
    let port = listener.local_addr().unwrap().port();

    let res = probe_port(port);
    assert!(res.is_open);
    assert_eq!(res.port, port);
}

#[test]
fn test_integration_pattern_matching() {
    let dummy_ports = vec![
        PortProcess {
            port: 3000,
            protocol: Protocol::Tcp,
            pid: 1234,
            ppid: None,
            name: "node".to_string(),
            cmdline: "".to_string(),
            cwd: None,
            project_name: Some("web".to_string()),
            framework: Some("Next.js".to_string()),
            memory_bytes: 1048576,
            cpu_usage: 0.0,
            user: Some("dev".to_string()),
            is_system_protected: false,
            is_orphan: false,
        },
        PortProcess {
            port: 5432,
            protocol: Protocol::Tcp,
            pid: 5678,
            ppid: None,
            name: "postgres".to_string(),
            cmdline: "".to_string(),
            cwd: None,
            project_name: None,
            framework: Some("PostgreSQL".to_string()),
            memory_bytes: 2097152,
            cpu_usage: 0.0,
            user: Some("postgres".to_string()),
            is_system_protected: false,
            is_orphan: false,
        },
    ];

    let patterns = vec!["3000".to_string(), "5430..5435".to_string()];
    let resolved = resolve_port_patterns(&patterns, &dummy_ports);

    assert!(resolved.contains(&3000));
    assert!(resolved.contains(&5432));
    assert!(resolved.contains(&5435));
}
