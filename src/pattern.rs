use std::collections::HashSet;
use crate::model::PortProcess;

pub fn resolve_port_patterns(patterns: &[String], all_ports: &[PortProcess]) -> Vec<u16> {
    let mut resolved_ports: HashSet<u16> = HashSet::new();

    for pat in patterns {
        let trimmed = pat.trim();

        // 1. Presets
        if trimmed == ":dev" || trimmed == "dev" {
            for &p in &[3000, 3001, 5173, 8000, 8080, 4200, 8888, 5000, 8008] {
                resolved_ports.insert(p);
            }
            continue;
        }
        if trimmed == ":db" || trimmed == "db" {
            for &p in &[5432, 3306, 6379, 27017, 9200, 9092] {
                resolved_ports.insert(p);
            }
            continue;
        }

        // 2. Ranges (e.g. 3000..3010 or 3000-3010)
        if let Some((start_str, end_str)) = trimmed.split_once("..").or_else(|| trimmed.split_once('-')) {
            if let (Ok(start), Ok(end)) = (start_str.parse::<u16>(), end_str.parse::<u16>()) {
                let (min, max) = if start <= end { (start, end) } else { (end, start) };
                for p in min..=max {
                    resolved_ports.insert(p);
                }
                continue;
            }
        }

        // 3. Wildcards (e.g. 80* or 30*)
        if trimmed.ends_with('*') {
            let prefix = trimmed.trim_end_matches('*');
            for p in all_ports {
                if p.port.to_string().starts_with(prefix) {
                    resolved_ports.insert(p.port);
                }
            }
            continue;
        }

        // 4. Exact number
        if let Ok(single_port) = trimmed.parse::<u16>() {
            resolved_ports.insert(single_port);
        }
    }

    let mut list: Vec<u16> = resolved_ports.into_iter().collect();
    list.sort();
    list
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Protocol;

    fn mock_port(port: u16) -> PortProcess {
        PortProcess {
            port,
            protocol: Protocol::Tcp,
            pid: 1000,
            ppid: None,
            name: "test".to_string(),
            cmdline: String::new(),
            cwd: None,
            project_name: None,
            framework: None,
            memory_bytes: 1024,
            cpu_usage: 0.0,
            user: None,
            is_system_protected: false,
            is_orphan: false,
        }
    }

    #[test]
    fn test_single_port_resolution() {
        let ports = vec![mock_port(3000), mock_port(8080)];
        let patterns = vec!["3000".to_string()];
        let res = resolve_port_patterns(&patterns, &ports);
        assert_eq!(res, vec![3000]);
    }

    #[test]
    fn test_range_dot_resolution() {
        let ports = vec![];
        let patterns = vec!["3000..3003".to_string()];
        let res = resolve_port_patterns(&patterns, &ports);
        assert_eq!(res, vec![3000, 3001, 3002, 3003]);
    }

    #[test]
    fn test_range_dash_resolution() {
        let ports = vec![];
        let patterns = vec!["8080-8082".to_string()];
        let res = resolve_port_patterns(&patterns, &ports);
        assert_eq!(res, vec![8080, 8081, 8082]);
    }

    #[test]
    fn test_wildcard_resolution() {
        let ports = vec![mock_port(80), mock_port(8080), mock_port(8000), mock_port(3000)];
        let patterns = vec!["80*".to_string()];
        let res = resolve_port_patterns(&patterns, &ports);
        assert_eq!(res, vec![80, 8000, 8080]);
    }

    #[test]
    fn test_dev_preset_resolution() {
        let ports = vec![];
        let patterns = vec![":dev".to_string()];
        let res = resolve_port_patterns(&patterns, &ports);
        assert!(res.contains(&3000));
        assert!(res.contains(&5173));
        assert!(res.contains(&8080));
    }

    #[test]
    fn test_db_preset_resolution() {
        let ports = vec![];
        let patterns = vec![":db".to_string()];
        let res = resolve_port_patterns(&patterns, &ports);
        assert!(res.contains(&5432));
        assert!(res.contains(&6379));
        assert!(res.contains(&27017));
    }
}
