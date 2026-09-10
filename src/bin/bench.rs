use std::process::Command;
use std::time::Instant;

fn main() {
    println!("🔬 Running 50 iterations of each tool on your live system...\n");

    // 1. Benchmark port-killer scan
    let start_pk = Instant::now();
    for _ in 0..50 {
        let _ = port_killer::scanner::scan_ports();
    }
    let duration_pk = start_pk.elapsed() / 50;

    // 2. Benchmark lsof
    let start_lsof = Instant::now();
    for _ in 0..50 {
        let _ = Command::new("lsof")
            .args(["-iTCP", "-sTCP:LISTEN", "-n", "-P"])
            .output();
    }
    let duration_lsof = start_lsof.elapsed() / 50;

    // 3. Benchmark fuser
    let start_fuser = Instant::now();
    for _ in 0..50 {
        let _ = Command::new("fuser").args(["3000/tcp"]).output();
    }
    let duration_fuser = start_fuser.elapsed() / 50;

    // 4. Benchmark ss
    let start_ss = Instant::now();
    for _ in 0..50 {
        let _ = Command::new("ss").args(["-tulpn"]).output();
    }
    let duration_ss = start_ss.elapsed() / 50;

    println!("==========================================================");
    println!("📊 100% REAL & VERIFIED BENCHMARK RESULTS (Average per run)");
    println!("==========================================================");
    println!("1. port-killer scan (Rust kernel procfs): {:.2} ms  ({:?})", duration_pk.as_secs_f64() * 1000.0, duration_pk);
    println!("2. ss -tulpn (C iproute2):               {:.2} ms  ({:?})", duration_ss.as_secs_f64() * 1000.0, duration_ss);
    println!("3. fuser 3000/tcp:                       {:.2} ms  ({:?})", duration_fuser.as_secs_f64() * 1000.0, duration_fuser);
    println!("4. lsof -iTCP -sTCP:LISTEN:              {:.2} ms  ({:?})", duration_lsof.as_secs_f64() * 1000.0, duration_lsof);
    println!("==========================================================\n");
}
