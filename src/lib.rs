#[cfg(feature = "x11")]
pub mod x11;

#[cfg(feature = "serialize")]
use miniserde::{Deserialize, Serialize};

#[cfg(feature = "global-cache")]
use directories::ProjectDirs;

#[cfg(feature = "global-cache")]
use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MonitorUtilsError {
    #[error("point out of bounds: {:?}", .0)]
    PointOutOfBounds(Point),

    #[error("monitor setup is invalid")]
    InvalidMonitorSetup,

    #[cfg(feature = "global-cache")]
    #[error("failed to read/write cache file")]
    ReadWriteCache {
        #[from]
        source: std::io::Error,
    },

    #[cfg(feature = "serialize")]
    #[error("cache cannot be parsed")]
    ParseCache {
        #[from]
        source: miniserde::Error,
    },
}

type LibResult<R> = std::result::Result<R, MonitorUtilsError>;

/// A Point represents an x, y coordinate relative to the top-left corner of the virtual screen.
/// This means that (100, 100) is the point 100 pixels down and 100 pixels to the right of the top
/// left corner of the virtual screen.
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Point {
    x: u32,
    y: u32,
}

impl Point {
    pub fn new(x: u32, y: u32) -> Point {
        Point { x, y }
    }

    pub fn x(&self) -> u32 {
        self.x
    }

    pub fn y(&self) -> u32 {
        self.y
    }
}

impl std::ops::Add for Point {
    type Output = Point;

    fn add(self, other: Self) -> Self::Output {
        Point::new(self.x() + other.x(), self.y() + other.y())
    }
}

/// Represents a Rectangle lying on a virtual screen.
/// The distinction between the Rectangle and the Monitor allows us to describe objects which do
/// not correspond to monitors.
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Rect {
    /// Width in pixels.
    width: u32,
    /// Height in pixels.
    height: u32,

    /// Offset of the top-left point of the Rectangle, relative to the top-left corner of the virtual
    /// screen.
    offset: Point,
}

impl Rect {
    /// Returns `true` if the point lies on the Rectangle, otherwise false.
    fn contains_point(&self, point: &Point) -> bool {
        let offset = self.offset;

        let x_min = offset.x();
        let x_max = x_min + self.width;

        let y_min = offset.y();
        let y_max = y_min + self.height;

        (point.x() >= x_min && point.x() < x_max) && (point.y() >= y_min && point.y() < y_max)
    }

    /// Returns the point at the center of the Rectangle.
    pub fn center(&self) -> Point {
        let raw_midpoint = Point::new(self.width / 2, self.height / 2);
        self.offset + raw_midpoint
    }

    /// Returns the width of the Rectangle in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height of the Rectangle in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Returns the offset of the Rectangle's top-left point relative to the top-left corner of the
    /// virtual screen.
    pub fn offset(&self) -> Point {
        self.offset
    }

    /// Returns `true` if the Rectangle is "empty", otherwise `false`.
    /// The definition of `empty` still has to be defined.
    fn is_empty(&self) -> bool {
        todo!()
    }

    /// Yields a Rectangle representing the intersection between the two input Rectangles.
    fn intersection(&self, other: &Self) -> Self {
        todo!()
    }

    /// Yields the (unsigned) area of the Rectangle.
    fn area(&self) -> u32 {
        self.width * self.height
    }
}

/// A `Monitor` represents a rectangular graphical display, positioned within a virtual Screen.
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Monitor {
    /// The index of the Monitor in a clock-wise ordering of its parent `MonitorSetup`
    order: u32,

    /// The name of the adapter corresponding to the monitor.
    name: String,
    /// CRTC index, used internally by graphics cards.
    crtc: u32,

    /// Rectangle representing the Monitor within the virtual screen.
    pub rect: Rect,
}

impl Monitor {
    /// Creates a new `Monitor`.
    pub fn new(name: String, crtc: u32, rect: Rect) -> Monitor {
        Monitor {
            // The `order` is default-initialized to 0 since we are not in a MonitorSetup yet.
            order: 0,

            name,
            crtc,
            rect,
        }
    }

    /// Yields the name of the adapter associated with the Monitor.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Yields the CRTC index of the monitor.
    pub fn crtc(&self) -> u32 {
        self.crtc
    }

    /// Yields the rectangle representing the Monitor within the virtual screen.
    pub fn rect(&self) -> &Rect {
        &self.rect
    }
}

/// A `MonitorSetup` represents a group of monitors used in conjunction with one another.
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct MonitorSetup {
    monitors: Vec<Monitor>,
}

impl MonitorSetup {
    /// Given an implementor of `LoadMonitors`, yields a `MonitorSetup`.
    pub fn with_loader<E>(loader: impl LoadMonitors<E>) -> Result<MonitorSetup, E> {
        let mut setup = MonitorSetup { monitors: vec![] };
        setup.reload(loader)?;

        Ok(setup)
    }

    #[cfg(feature = "serialize")]
    pub fn from_json(json_string: &str) -> LibResult<Self> {
        miniserde::json::from_str(json_string).map_err(|e| e.into())
    }

    #[cfg(feature = "global-cache")]
    fn get_cache_file() -> PathBuf {
        ProjectDirs::from("com.github", "srithon", "monitor-utils")
            .expect("Home directory must be valid")
            .cache_dir()
            .to_path_buf()
    }

    #[cfg(feature = "global-cache")]
    pub fn from_global_cache() -> LibResult<Self> {
        let cache_file = Self::get_cache_file();
        let string = std::fs::read_to_string(cache_file)?;

        Self::from_json(&string).map_err(|e| e.into())
    }

    #[cfg(feature = "global-cache")]
    pub fn to_global_cache(&self) -> LibResult<()> {
        let cache_file = Self::get_cache_file();
        std::fs::write(cache_file, miniserde::json::to_string(self))?;

        Ok(())
    }

    /// Reloads the list of monitors from the source.
    pub fn reload<E>(&mut self, loader: impl LoadMonitors<E>) -> Result<(), E> {
        self.monitors = loader.load_monitors()?;

        // now, sort them in clockwise order
        self.sort_clockwise();
        Ok(())
    }

    /// Sorts the internal list of monitors in a clockwise order, with further monitors coming
    /// before closer ones to break diagonal ties.
    /// "Clockwise" in this implementation refers to the top-left corners of the monitors.
    fn sort_clockwise(&mut self) {
        // compute angle from origin, distance from origin for top left corner
        self.monitors.sort_by(|m1, m2| {
            let to_angle_distance = |monitor: &Monitor| {
                let top_left = &monitor.rect.offset;

                // https://stackoverflow.com/questions/17530169/get-angle-between-point-and-origin
                let angle = f32::atan2(top_left.y() as f32, top_left.x() as f32);
                let distance = ((top_left.x().pow(2) + top_left.y().pow(2)) as f32).sqrt();

                (angle, distance)
            };

            // can't compare normally because NaN is unordered
            to_angle_distance(m1)
                .partial_cmp(&to_angle_distance(m2))
                .expect("Should not have NaN values.")
        });

        self.update_monitor_ordering();
    }

    /// Updates the internal ordering for the monitors, such that each Monitor contains the correct
    /// index for itself.
    fn update_monitor_ordering(&mut self) {
        for (index, monitor) in self.monitors.iter_mut().enumerate() {
            monitor.order = index as u32
        }
    }

    /// Yields the monitor which contains the given point.
    pub fn monitor_containing_point(&self, point: &Point) -> LibResult<&Monitor> {
        self.monitors
            .iter()
            .filter(|m| m.rect.contains_point(&point))
            .next()
            .ok_or(MonitorUtilsError::PointOutOfBounds(*point))
    }

    /// Given a monitor index and an offset, returns the monitor at the offset index, such that
    /// overflows loop back to the beginning, and underflows loop back from the end.
    fn monitor_at_offset_index(&self, index: u32, offset: i32) -> LibResult<&Monitor> {
        let num_monitors = self.monitors.len() as u32;

        // get rid of any redundant loops
        let new_offset = (offset.abs() as u32 % num_monitors) as i32 * offset.signum();
        let mut new_index = index as i32 + new_offset;

        if new_index < 0 {
            new_index += num_monitors as i32;
            assert!(new_index >= 0)
        }

        self.monitors
            .get((new_index as u32 % num_monitors) as usize)
            .ok_or(MonitorUtilsError::InvalidMonitorSetup)
    }

    /// Yields the next monitor in a clock-wise traversal of the MonitorSetup.
    pub fn next_monitor_clockwise(&self, monitor: &Monitor) -> LibResult<&Monitor> {
        self.monitor_at_offset_index(monitor.order, 1)
    }

    /// Yields the next monitor in a counter-clockwise traversal of the MonitorSetup.
    pub fn next_monitor_counterclockwise(&self, monitor: &Monitor) -> LibResult<&Monitor> {
        self.monitor_at_offset_index(monitor.order, -1)
    }

    /// Yields the monitor above the given monitor.
    pub fn monitor_above(&self, _monitor: &Monitor) -> LibResult<&Monitor> {
        todo!()
    }

    /// Yields the monitor below the given monitor.
    pub fn monitor_below(&self, _monitor: &Monitor) -> LibResult<&Monitor> {
        todo!()
    }

    /// Yields the monitor to the left of the given monitor.
    pub fn monitor_left_of(&self, _monitor: &Monitor) -> LibResult<&Monitor> {
        todo!()
    }

    /// Yields the monitor to the right of the given monitor.
    pub fn monitor_right_of(&self, _monitor: &Monitor) -> LibResult<&Monitor> {
        todo!()
    }
}

/// Trait which abstracts loading the list of monitors from the respective environment. By
/// implementing this trait, you can use the library's functionality for arbitrary windowing
/// systems.
pub trait LoadMonitors<E> {
    fn load_monitors(&self) -> Result<Vec<Monitor>, E>;
}
