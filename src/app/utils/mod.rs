pub mod config_parser;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
pub mod fs;
pub mod i18n;
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

pub fn permissions_to_string(permissions: &Permissions) -> String {
    let mode = permissions.mode() & 0o777;
    let mut result = String::with_capacity(9);

    result.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    result.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    result.push(if mode & 0o100 != 0 { 'x' } else { '-' });

    result.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    result.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    result.push(if mode & 0o010 != 0 { 'x' } else { '-' });

    result.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    result.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    result.push(if mode & 0o001 != 0 { 'x' } else { '-' });

    result
}
