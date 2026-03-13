use chrono::{DateTime, Local, TimeZone};

pub fn format_size(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = 1024 * KIB;
    const GIB: u64 = 1024 * MIB;
    const TIB: u64 = 1024 * GIB;

    if bytes >= TIB {
        format!("{:.1} TiB", bytes as f64 / TIB as f64)
    } else if bytes >= GIB {
        format!("{:.1} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.1} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.1} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn epoch_to_string(epoch: u64) -> String {
    Local
        .timestamp_opt(epoch as i64, 0)
        .single()
        .map(|dt: DateTime<Local>| dt.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn epoch_to_iso(epoch: u64) -> String {
    Local
        .timestamp_opt(epoch as i64, 0)
        .single()
        .map(|dt: DateTime<Local>| dt.format("%Y-%m-%dT%H:%M:%S").to_string())
        .unwrap_or_else(|| "unknown".to_string())
}
