# monitor-utils

`monitor-utils` is a Rust library and binary for making spatial queries with monitors. It provides functionality for working with monitors and their configurations. This README provides an overview of the library's public API, cargo features, and information about the CLI.

## Cargo Features

- `default`: By default, only the `x11` feature is enabled.
- `x11`: Enables the `x11` module, which contains a `LoadMonitors` implementation using `XRandr`.
- `serialize`: Uses `miniserde` to serialize/deserialize `MonitorSetup`.
- `global-cache`: Uses `serialize` to read/write setup from a global cache.
- `cli`: Enables compilation of the binary: `bin/monitor-utils`.

## Public API

### `Rect` struct

Represents a rectangle lying on a virtual screen.

#### Methods

- `center() -> Point`: Returns the point at the center of the `Rect`.
    - this can be used in conjunction with `Monitor` to get the point at the center of a `Monitor`

### `LoadMonitors` trait

A trait that abstracts loading the list of monitors from the respective environment. By implementing this trait, you can use the library's functionality for arbitrary windowing systems.

#### Methods

- `load_monitors() -> Result<Vec<Monitor>, E>`: Loads the list of monitors and returns a vector of `Monitor` instances. Generic over the Error type.

### `MonitorSetup` struct

Represents a group of monitors used in conjunction with one another.

#### Methods

- `with_loader(loader: impl LoadMonitors<E>) -> Result<MonitorSetup, E>`: Creates a `MonitorSetup` instance using the provided `LoadMonitors` implementation.
- `reload(loader: impl LoadMonitors<E>) -> Result<(), E>`: Reloads the monitor setup using the provided `LoadMonitors` implementation.

- `from_json(json_string: &str) -> Result<Self>`: (`serialize` feature) Creates a `MonitorSetup` instance by deserializing from a JSON string.

- `from_global_cache() -> Result<Self>`: (`global-cache` feature) Creates a `MonitorSetup` instance by reading from the global cache.
- `to_global_cache() -> Result<()>`: (`global-cache` feature) Writes the `MonitorSetup` instance to the global cache.

- `monitor_containing_point(point: &Point) -> Result<&Monitor>`: Returns the monitor that contains the given point.
- `next_monitor_clockwise(monitor: &Monitor) -> Result<&Monitor>`: Returns the next monitor in a clockwise traversal of the `MonitorSetup`.
- `next_monitor_counterclockwise(monitor: &Monitor) -> Result<&Monitor>`: Returns the next monitor in a counterclockwise traversal of the `MonitorSetup`.

- TODO: `monitor_above(monitor: &Monitor) -> Result<&Monitor>`: Returns the monitor above the given monitor.
- TODO: `monitor_below(monitor: &Monitor) -> Result<&Monitor>`: Returns the monitor below the given monitor.
- TODO: `monitor_left_of(monitor: &Monitor) -> Result<&Monitor>`: Returns the monitor to the left of the given monitor.
- TODO: `monitor_right_of(monitor: &Monitor) -> Result<&Monitor>`: Returns the monitor to the right of the given monitor.

## CLI Usage

The `monitor-utils` CLI has a unique interface for interacting with the library. It allows you to perform various actions and chain them together using a pipeline-like syntax.

### Usage

```plaintext
CLI for monitor-utils

Usage: [-s] [-r] [--at-point <X> <Y> | (--clockwise | --counter-clockwise | --center)]...

Available options:
    -s, --shell      If specified, spit out output in POSIX shell variable format, such that it may
                     be eval'd
    -r, --refresh    If specified, refreshes the cache before running actions
  The following options are commands, which pipeline data from the left of the command-line to the
  right.
  --at-point <X> <Y>
  Takes 2 arguments: X and Y, and yields the monitor containing the point (X,Y)
        --at-point


  These commands each take in a Monitor through the pipeline, and yield either a Point or another
  Monitor.
        --clockwise  Given an argument monitor, yields the next monitor in a clockwise rotation.
        --counter-clockwise  Given an argument monitor, yields the next monitor in a
                     counter-clockwise rotation.
        --center     Given an argument monitor, yields the point at the center of the monitor.


    -h, --help       Prints help information
    -V, --version    Prints version information
```

### Pipeline Example

```plaintext
$ monitor-utils --at-point 10 10 --clockwise --center
Point { x: 2432, y: 384 }
$ monitor-utils --shell --at-point 10 10 --clockwise --center
X=2432
Y=384
```

### Real Application Example

The `monitor-utils` CLI can be used in conjunction with `xdotool` to perform actions based on monitor configurations. For example, you can use it to move your cursor to the center of the next monitor.

```bash
# Get the X and Y coordinates of the mouse
eval $(xdotool getmouselocation)
# Get the X and Y coordinates of the center of the next monitor
eval $(monitor-utils --at-point $X $Y --clockwise --center)
# Move the mouse to the center of the next monitor
xdotool mousemove $X $Y
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
