use crate::{LoadMonitors, Monitor, Point, Rect};

use std::process::Command;

use std::io::{Error, ErrorKind};

/// This is an implementor for `LoadMonitors` which uses the `xrandr` command-line interface to
/// load the list of monitors.
/// Note that this will not work on Wayland.
pub struct XRandrMonitorLoader;

impl XRandrMonitorLoader {
    /// Creates an instance of `XRandrMonitorLoader` if `xrandr` is installed and usable;
    /// otherwise, yields an Error.
    pub fn new() -> Result<XRandrMonitorLoader, Error> {
        let mut child = Command::new("xrandr").arg("--current").spawn()?;

        let exit_status = child.wait()?;
        let code = exit_status.code();

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

impl LoadMonitors for XRandrMonitorLoader {
    fn load_monitors(&self) -> Vec<Monitor> {
        unimplemented!()
    }
}
