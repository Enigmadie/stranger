pub mod fs;
const KB: f64 = 1024.0;
const MB: f64 = KB * 1024.0;
const GB: f64 = MB * 1024.0;

pub fn format_bytes(bytes: u64) -> String {
    let b = bytes as f64;

    if b >= GB {
        format!("{} G", format_float_clean(bytes as f64 / GB))
    } else if b >= MB {
        format!("{} M", format_float_clean(bytes as f64 / MB))
    } else if b >= KB {
        format!("{} K", format_float_clean(bytes as f64 / KB))
    } else {
        format!("{} B", b)
    }
}

fn format_float_clean(n: f64) -> String {
    if n.fract() == 0.0 {
        format!("{:.0}", n)
    } else if (n * 10.0).fract() == 0.0 {
        format!("{:.1}", n)
    } else {
        format!("{:.2}", n)
    }
}
