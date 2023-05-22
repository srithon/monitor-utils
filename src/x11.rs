use crate::{LoadMonitors, Monitor, Point, Rect};

use std::process::Command;

use once_cell::sync::OnceCell;
use regex::Regex;

use std::io::{Error, ErrorKind};

fn xrandr_display_information_regex() -> &'static Regex {
    static XRANDR_DISPLAY_INFORMATION_REGEX: OnceCell<Regex> = OnceCell::new();
    XRANDR_DISPLAY_INFORMATION_REGEX.get_or_init(|| {
        Regex::new(
            r"(?x) # ignore whitespace
            # [[:alpha:]] represents ascii letters
            ^([[:alpha:]]+-[[:digit:]]+) # 0 : the adapter name
            \ # space
            # 1 : 'disconnected' or 'connected ...'
            (
                disconnected
                |
                connected
                \ # space
                .*? # optional other words
                ([[:digit:]]+) # 2 : width
                x
                ([[:digit:]]+) # 3 : height
                \+
                ([[:digit:]]+) # 4 : x_offset
                \+
                ([[:digit:]]+) # 5 : y_offset
            )
            ",
        )
        .unwrap()
    })
}

fn xrandr_crtc_regex() -> &'static Regex {
    static XRANDR_CRTC_REGEX: OnceCell<Regex> = OnceCell::new();
    XRANDR_CRTC_REGEX.get_or_init(|| {
        Regex::new(
            r"(?x) # ignore whitespace
        # NOTE: for some reason the [:digit:] needs to be enclosed in more
        ^(\ |\t)+CRTC: (\ |\t)+([[:digit:]]) # 3 : the crtc number
        ",
        )
        .unwrap()
    })
}

/// This is an implementor for `LoadMonitors` which uses the `xrandr` command-line interface to
/// load the list of monitors.
/// Note that this will not work on Wayland.
pub struct XRandrMonitorLoader;

impl XRandrMonitorLoader {
    /// Creates an instance of `XRandrMonitorLoader` if `xrandr` is installed and usable;
    /// otherwise, yields an Error.
    pub fn new() -> Result<XRandrMonitorLoader, Error> {
        let output = Command::new("xrandr").arg("--current").output()?;
        let code = output.status.code();

        match code {
            Some(0) => Ok(XRandrMonitorLoader {}),
            _ => {
                let exit_message = if let Some(code) = code {
                    format!("exit code {}", code)
                } else {
                    format!("no exit code")
                };

                Err(Error::new(
                    ErrorKind::Other,
                    format!("xrandr returned with {}", exit_message),
                ))
            }
        }
    }
}

/// Given a line from the output of `xrandr --query`, attempts to extract a `Monitor` specification
/// from it.
fn try_monitor_from_xrandr_line(xrandr_line: &str) -> Option<Monitor> {
    // eDP-1 connected primary 1920x1080+0+0 (normal left inverted right x axis y axis) 344mm x 193mm
    // HDMI-1 connected 1280x1024+1920+28 (normal left inverted right x axis y axis) 338mm x 270mm
    // <adapter> connected [primary] <width>x<height>+<x offset>+<y offset> (<flags>) <something>mm x <something else>mm
    let captures = xrandr_display_information_regex().captures(xrandr_line);

    if let Some(captures) = captures {
        // 0 points to the entire match, so skip
        let adapter_name = captures.get(1).unwrap().as_str().to_owned();

        let parse_int = |num: regex::Match| num.as_str().parse::<u32>().map_err(|_| ());

        (|| {
            match captures.get(2).map(|capture| capture.as_str()) {
                Some("disconnected") | None => return Err(()),
                _ => (),
            };

            let monitor_rectangle = {
                let width = parse_int(captures.get(3).unwrap())?;
                let height = parse_int(captures.get(4).unwrap())?;
                let x_offset = parse_int(captures.get(5).unwrap())?;
                let y_offset = parse_int(captures.get(6).unwrap())?;

                let offset = Point::new(x_offset, y_offset);

                Rect {
                    width,
                    height,
                    offset,
                }
            };

            // set CRTC to 0 to begin with
            Ok(Monitor::new(adapter_name, 0, monitor_rectangle))
        })()
        .ok()
    } else {
        None
    }
}

/// Given a line from the output of `xrandr --query --verbose`, attempts to extract a `CRTC`
/// specification from it.
fn try_crtc_from_xrandr_line(xrandr_line: &str) -> Option<u32> {
    xrandr_crtc_regex().captures(xrandr_line).map(|captures| {
        let crtc_number_string = captures.get(3).expect("Capture must have a 3rd item");
        crtc_number_string
            .as_str()
            .parse()
            .expect("CRTC number must be parsable")
    })
}

impl LoadMonitors<Error> for XRandrMonitorLoader {
    /// Parses `xrandr --current` output and returns a list of connected monitors
    fn load_monitors(&self) -> Result<Vec<Monitor>, Error> {
        let mut xrandr_current = Command::new("xrandr");
        xrandr_current.arg("--current");
        xrandr_current.arg("--verbose");
        let command_output = xrandr_current.output()?;

        // the '&' operator dereferences ascii_code so that it can be compared with a regular u8
        // its original type is &u8
        let output_lines = command_output
            .stdout
            .split(|&ascii_code| ascii_code == b'\n');

        let mut monitors: Vec<Monitor> = Vec::new();

        for line in output_lines {
            // if valid UTF-8, pass to Monitor
            if let Ok(line) = std::str::from_utf8(line) {
                if let Some(monitor) = try_monitor_from_xrandr_line(line) {
                    monitors.push(monitor)
                } else if let Some(crtc) = try_crtc_from_xrandr_line(line) {
                    // assign crtc number to the latest display
                    monitors.last_mut().expect("Vector must not be empty").crtc = crtc;
                }
            }
        }

        Ok(monitors)
    }
}
