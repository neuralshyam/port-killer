use std::fs;
use std::path::Path;
use serde_json::Value;

pub struct ProjectMeta {
    pub cwd: Option<String>,
    pub project_name: Option<String>,
    pub framework: Option<String>,
}

pub fn detect_project_info(pid: u32, proc_name: &str, port: u16) -> ProjectMeta {
    let mut meta = ProjectMeta {
        cwd: None,
        project_name: None,
        framework: None,
    };

    // 1. Resolve CWD
    #[cfg(target_os = "linux")]
    {
        let cwd_link = format!("/proc/{}/cwd", pid);
        if let Ok(target) = fs::read_link(&cwd_link) {
            let path_str = target.to_string_lossy().to_string();
            meta.cwd = Some(path_str.clone());
            inspect_directory(&target, &mut meta);
        }
    }

    // 2. Fallback framework detection by port or process name
    if meta.framework.is_none() {
        meta.framework = infer_framework_from_port_and_name(port, proc_name);
    }

    meta
}

pub fn inspect_directory(dir: &Path, meta: &mut ProjectMeta) {
    // Check package.json (Node/Bun/TS ecosystem)
    let pkg_json = dir.join("package.json");
    if pkg_json.exists() {
        if let Ok(content) = fs::read_to_string(&pkg_json) {
            if let Ok(val) = serde_json::from_str::<Value>(&content) {
                if let Some(name) = val.get("name").and_then(|n| n.as_str()) {
                    meta.project_name = Some(name.to_string());
                }

                // Detect JS/TS framework from dependencies
                let mut fw = None;
                let deps = val.get("dependencies").and_then(|d| d.as_object());
                let dev_deps = val.get("devDependencies").and_then(|d| d.as_object());

                let has_dep = |key: &str| -> bool {
                    deps.map_or(false, |d| d.contains_key(key))
                        || dev_deps.map_or(false, |d| d.contains_key(key))
                };

                if has_dep("next") {
                    fw = Some("Next.js".to_string());
                } else if has_dep("vite") {
                    fw = Some("Vite".to_string());
                } else if has_dep("@nestjs/core") {
                    fw = Some("NestJS".to_string());
                } else if has_dep("astro") {
                    fw = Some("Astro".to_string());
                } else if has_dep("nuxt") || has_dep("nuxt3") {
                    fw = Some("Nuxt".to_string());
                } else if has_dep("@remix-run/node") {
                    fw = Some("Remix".to_string());
                } else if has_dep("express") {
                    fw = Some("Express".to_string());
                } else if has_dep("fastify") {
                    fw = Some("Fastify".to_string());
                } else if has_dep("hono") {
                    fw = Some("Hono".to_string());
                } else if has_dep("react") {
                    fw = Some("React".to_string());
                } else if has_dep("vue") {
                    fw = Some("Vue".to_string());
                } else if has_dep("svelte") || has_dep("@sveltejs/kit") {
                    fw = Some("Svelte".to_string());
                }

                meta.framework = fw;
            }
        }
    }

    // Check Cargo.toml (Rust)
    let cargo_toml = dir.join("Cargo.toml");
    if cargo_toml.exists() && meta.project_name.is_none() {
        if let Ok(content) = fs::read_to_string(&cargo_toml) {
            if let Ok(val) = content.parse::<toml::Value>() {
                if let Some(pkg) = val.get("package") {
                    if let Some(name) = pkg.get("name").and_then(|n| n.as_str()) {
                        meta.project_name = Some(name.to_string());
                        meta.framework = Some("Rust".to_string());
                    }
                }
            }
        }
    }

    // Check go.mod (Go)
    let go_mod = dir.join("go.mod");
    if go_mod.exists() && meta.project_name.is_none() {
        if let Ok(content) = fs::read_to_string(&go_mod) {
            if let Some(first_line) = content.lines().next() {
                if first_line.starts_with("module ") {
                    meta.project_name = Some(first_line.trim_start_matches("module ").to_string());
                    meta.framework = Some("Go".to_string());
                }
            }
        }
    }

    // Fallback: If no project name, use folder name
    if meta.project_name.is_none() {
        if let Some(folder_name) = dir.file_name().map(|n| n.to_string_lossy().to_string()) {
            meta.project_name = Some(folder_name);
        }
    }
}

pub fn infer_framework_from_port_and_name(port: u16, name: &str) -> Option<String> {
    let lower = name.to_lowercase();
    if lower.contains("postgres") || port == 5432 {
        return Some("PostgreSQL".to_string());
    }
    if lower.contains("mysqld") || lower.contains("mariadb") || port == 3306 {
        return Some("MySQL/MariaDB".to_string());
    }
    if lower.contains("redis") || port == 6379 {
        return Some("Redis".to_string());
    }
    if lower.contains("mongod") || port == 27017 {
        return Some("MongoDB".to_string());
    }
    if lower.contains("docker-proxy") {
        return Some("Docker Port Forward".to_string());
    }
    if lower.contains("nginx") {
        return Some("Nginx Reverse Proxy".to_string());
    }
    if lower.contains("caddy") {
        return Some("Caddy Web Server".to_string());
    }

    match port {
        3000 => Some("Dev Server (Next/React/Bun)".to_string()),
        5173 => Some("Vite Dev Server".to_string()),
        8080 => Some("HTTP Proxy / App Server".to_string()),
        8000 => Some("Backend API / Django / FastAPI".to_string()),
        4200 => Some("Angular Dev Server".to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framework_inference_by_port() {
        assert_eq!(infer_framework_from_port_and_name(5432, "postgres"), Some("PostgreSQL".to_string()));
        assert_eq!(infer_framework_from_port_and_name(6379, "redis-server"), Some("Redis".to_string()));
        assert_eq!(infer_framework_from_port_and_name(3000, "node"), Some("Dev Server (Next/React/Bun)".to_string()));
        assert_eq!(infer_framework_from_port_and_name(5173, "bun"), Some("Vite Dev Server".to_string()));
    }

    #[test]
    fn test_framework_inference_by_name() {
        assert_eq!(infer_framework_from_port_and_name(9999, "nginx"), Some("Nginx Reverse Proxy".to_string()));
        assert_eq!(infer_framework_from_port_and_name(9999, "docker-proxy"), Some("Docker Port Forward".to_string()));
    }
}
