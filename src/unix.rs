extern crate libc;
use std::os::raw::*;
use std::env;

use super::{Height, Width};

#[cfg(target_os = "macos")]
const TIOCGWINSZ: c_ulong = 0x40087468;
#[cfg(all(target_env = "musl", not(target_os = "macos")))]
const TIOCGWINSZ: c_int = 0x00005413;
#[cfg(all(not(target_env = "musl"), not(target_os = "macos")))]
const TIOCGWINSZ: c_ulong = 0x00005413;

#[derive(Debug)]
struct WinSize {
    ws_row: c_ushort,
    ws_col: c_ushort,
    ws_xpixel: c_ushort,
    ws_ypixel: c_ushort,
}

/// Returns the size of the terminal defaulting to STDOUT, if available.
///
/// If STDOUT is not a tty, returns `None`
/// If STDOUT is a tty, but both width and height is 0,
/// fallback to use system env: COLUMNS and LINES.
pub fn terminal_size() -> Option<(Width, Height)> {
    let size = terminal_size_using_fd();
    match size {
        Some((Width(0), Height(0))) => {
            terminal_size_using_env()
        },
        _ => size,
    }
}

/// Returns the size of the terminal using the given file descriptor, if available.
///
/// If the STDOUT file descriptor is not a tty, returns `None`
pub fn terminal_size_using_fd() -> Option<(Width, Height)> {
    use self::libc::STDOUT_FILENO;
    use self::libc::ioctl;
    use self::libc::isatty;

    let fd = STDOUT_FILENO;
    let is_tty: bool = unsafe { isatty(fd) == 1 };

    if !is_tty {
        return None;
    }

    let (rows, cols) = unsafe {
        let mut winsize = WinSize {
            ws_row: 0,
            ws_col: 0,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        ioctl(fd, TIOCGWINSZ, &mut winsize);
        let rows = if winsize.ws_row > 0 {
            winsize.ws_row
        } else {
            0
        };
        let cols = if winsize.ws_col > 0 {
            winsize.ws_col
        } else {
            0
        };
        (rows as u16, cols as u16)
    };

    Some((Width(cols), Height(rows)))
}

/// Returns the size of the terminal using system env:
/// COLUMNS and LINES
///
/// If both env are 0, returns `None`
fn terminal_size_using_env() -> Option<(Width, Height)> {
    let get_u16_from_env = |x: &str| -> u16 {
        if let Some(v) = env::var_os(x) {
            v.into_string().unwrap()
                .parse::<u16>().unwrap()
        } else {
            0
        }
    };
    let c = get_u16_from_env("COLUMNS");
    let r = get_u16_from_env("LINES");

    if r == 0 && c == 0 {
        None
    } else {
        return Some((Width(c), Height(r)))
    }
}

#[test]
/// Compare using_fd with the output of `stty size`
fn compare_using_fd_with_stty() {
    use std::process::Command;
    use std::process::Stdio;

    let output = if cfg!(target_os = "macos") {
        Command::new("stty")
            .arg("-f")
            .arg("/dev/stderr")
            .arg("size")
            .stderr(Stdio::inherit())
            .output()
            .unwrap()
    } else {
        Command::new("stty")
            .arg("size")
            .arg("-F")
            .arg("/dev/stderr")
            .stderr(Stdio::inherit())
            .output()
            .unwrap()
    };
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(output.status.success());
    // stdout is "rows cols"
    let mut data = stdout.split_whitespace();
    let rows = u16::from_str_radix(data.next().unwrap(), 10).unwrap();
    let cols = u16::from_str_radix(data.next().unwrap(), 10).unwrap();
    println!("{}", stdout);
    println!("{} {}", rows, cols);

    if let Some((Width(w), Height(h))) = terminal_size_using_fd() {
        assert_eq!(rows, h);
        assert_eq!(cols, w);
    }
}

#[test]
/// Compare using env result with the output of `tput cols` and `tput lines`
fn compare_using_env_with_tput() {
    use std::process::Command;
    use std::process::Stdio;

    let cols_output = Command::new("tput")
        .arg("cols")
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
    let rows_output = Command::new("tput")
        .arg("lines")
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
    assert!(cols_output.status.success());
    assert!(rows_output.status.success());

    let cols = String::from_utf8(cols_output.stdout).unwrap()
        .trim()
        .parse::<u16>()
        .unwrap();
    let rows = String::from_utf8(rows_output.stdout).unwrap()
        .trim()
        .parse::<u16>()
        .unwrap();

    println!("{} {}", rows, cols);

    if let Some((Width(w), Height(h))) = terminal_size_using_env() {
        assert_eq!(rows, h);
        assert_eq!(cols, w);
    }
}

#[test]
/// Compare result with the output of `tput cols` and `tput lines`
fn compare_with_tput() {
    use std::process::Command;
    use std::process::Stdio;

    let cols_output = Command::new("tput")
        .arg("cols")
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
    let rows_output = Command::new("tput")
        .arg("lines")
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
    assert!(cols_output.status.success());
    assert!(rows_output.status.success());

    let cols = String::from_utf8(cols_output.stdout).unwrap()
        .trim()
        .parse::<u16>()
        .unwrap();
    let rows = String::from_utf8(rows_output.stdout).unwrap()
        .trim()
        .parse::<u16>()
        .unwrap();

    println!("{} {}", rows, cols);

    if let Some((Width(w), Height(h))) = terminal_size() {
        assert_eq!(rows, h);
        assert_eq!(cols, w);
    }
}
