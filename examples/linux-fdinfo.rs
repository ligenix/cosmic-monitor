use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

fn main() {
    // Do we have to collect more than one per process?
    let mut fdinfos = Vec::new();

    let instant = Instant::now();
    for pid in std::env::args().skip(1) {
        let proc_path = Path::new("/proc").join(&pid);
        let proc_fd_path = proc_path.join("fd");
        let proc_fdinfo_path = proc_path.join("fdinfo");
        let Ok(entries) = fs::read_dir(&proc_fd_path) else {
            continue;
        };
        for entry_res in entries {
            let Ok(entry) = entry_res else { continue };
            let path = entry.path();
            let Ok(link) = fs::read_link(&path) else {
                continue;
            };
            if link.starts_with("/dev/dri") {
                let name = entry.file_name();
                if let Ok(data) = fs::read_to_string(proc_fdinfo_path.join(&name)) {
                    fdinfos.push((pid.clone(), name, link, data));
                }
            }
        }
    }
    let elapsed = instant.elapsed();

    let instant = Instant::now();
    for (pid, name, link, data) in fdinfos {
        println!("{}: {}: {}", pid, name.display(), link.display());
        for line in data.lines() {
            let Some((key, value)) = line.split_once(":") else {
                continue;
            };
            // https://docs.kernel.org/gpu/drm-usage-stats.html
            if let Some(key) = key.strip_prefix("drm-") {
                let value = value.trim_start();
                if let Some(key) = key.strip_prefix("total-") {
                    let mut parts = value.splitn(2, ' ');
                    let Ok(mut bytes) = parts.next().unwrap_or_default().parse::<u64>() else {
                        continue;
                    };
                    match parts.next().unwrap_or_default() {
                        "KiB" => {
                            // Kilobytes
                            bytes *= 1024;
                        }
                        "MiB" => {
                            // Megabytes
                            bytes *= 1024 * 1024;
                        }
                        // Other suffixes not defined
                        _ => {
                            continue;
                        }
                    }
                    println!(
                        "  total {}: {}",
                        key,
                        humansize::format_size(bytes, humansize::BINARY)
                    );
                }
                if let Some(key) = key.strip_prefix("engine-") {
                    if key.starts_with("capacity-") {
                        continue;
                    }
                    let mut parts = value.splitn(2, ' ');
                    let Ok(nanos) = parts.next().unwrap_or_default().parse::<u64>() else {
                        continue;
                    };
                    match parts.next().unwrap_or_default() {
                        "ns" => {
                            // Nanoseconds
                        }
                        // Other suffixes not defined
                        _ => {
                            continue;
                        }
                    }
                    println!("  engine {}: {:?}", key, Duration::from_nanos(nanos));
                }
            }
        }
    }

    eprintln!(
        "Collected in {:?}, printed in {:?}",
        elapsed,
        instant.elapsed()
    );
}
