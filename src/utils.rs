use std::fmt::Write;

pub fn size_to_string(size: f64) -> String {
    if size <= 0.0 {
        return "-".to_string();
    }

    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];

    let i = if size < 1.0 {
        0
    } else {
        size.log(1024.0).floor() as usize
    };

    let i = i.min(UNITS.len().saturating_sub(1));

    let p = 1024_f64.powf(i as f64);
    let s = size / p;

    let mut buffer = String::with_capacity(10);
    write!(&mut buffer, "{:.2} {}", s, UNITS[i]).unwrap();

    buffer
}
