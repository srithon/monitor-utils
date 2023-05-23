use monitor_utils::{x11::XRandrMonitorLoader, Monitor, MonitorSetup, Point};

use bpaf::{construct, long, positional, short, OptionParser, Parser};

use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
enum Action {
    // need to have the () to satisfy bpaf
    MonitorAtPoint((), Point),

    NextMonitorClockwise,
    NextMonitorCounterClockwise,
    MonitorCenter,
}

#[derive(Debug)]
struct Options {
    refresh: bool,

    // actions are pipelined from left to right
    actions: Vec<Action>,
}

fn cli() -> OptionParser<Options> {
    let refresh = short('r')
        .long("refresh")
        .help("If specified, refreshes the cache before running actions")
        .req_flag(true)
        .fallback(false);

    let clockwise = long("clockwise")
        .help("Given an argument monitor, yields the next monitor in a clockwise rotation.")
        .req_flag(Action::NextMonitorClockwise);

    let counter_clockwise = long("counter-clockwise")
        .help("Given an argument monitor, yields the next monitor in a counter-clockwise rotation.")
        .req_flag(Action::NextMonitorCounterClockwise);

    let center = long("center")
        .help("Given an argument monitor, yields the point at the center of the monitor.")
        .req_flag(Action::MonitorCenter);

    fn monitor_at_point() -> impl Parser<Action> {
        let monitor_at_point = long("at-point").req_flag(()).group_help(
            "Takes 2 arguments: X and Y, and yields the monitor containing the point (X,Y)",
        );
        let x = positional::<u32>("X");
        let y = positional::<u32>("Y");

        let point = construct!(Point::new(x, y));

        construct!(Action::MonitorAtPoint(monitor_at_point, point)).adjacent()
    }

    let actions = construct!([clockwise, counter_clockwise, center, monitor_at_point()]).many();

    let parser = construct!(Options { refresh, actions });
    parser
        .to_options()
        .version(env!("CARGO_PKG_VERSION"))
        .descr("CLI for monitor-utils")
}

fn main() -> Result<()> {
    let cli = cli();
    let options = cli.run();

    let mut monitor_setup = None;

    if !options.refresh {
        match MonitorSetup::from_global_cache() {
            Ok(setup) => {
                monitor_setup.replace(setup);
            }
            Err(_) => (),
        };
    }

    if monitor_setup.is_none() {
        // use a different loader depending on enabled feature
        #[cfg(feature = "x11")]
        let loader = XRandrMonitorLoader::new()?;

        // Example future code:
        //
        // #[cfg(feature = "wayland")]
        // let loader = WaylandMonitorLoader::new()?;

        let setup = MonitorSetup::with_loader(loader)?;
        setup.to_global_cache()?;
        monitor_setup.replace(setup);
    }

    let monitor_setup = monitor_setup.expect("Monitor setup must exist");

    // now, let's run our actions
    enum Accumulator<'a> {
        AccumPoint(Point),
        AccumMonitor(&'a Monitor),
    }

    use Accumulator::*;
    use Action::*;

    let res = options
        .actions
        .into_iter()
        .try_fold(AccumPoint(Point::new(0, 0)), |acc, act| match act {
            MonitorAtPoint((), point) => Ok(AccumMonitor(
                monitor_setup.monitor_containing_point(&point)?,
            )),
            _ => {
                let monitor = match acc {
                    AccumMonitor(monitor) => monitor,
                    _ => return Err(anyhow!("Expected Monitor in accumulator but found Point")),
                };

                match act {
                    NextMonitorClockwise => Ok(AccumMonitor(
                        monitor_setup.next_monitor_clockwise(monitor).unwrap(),
                    )),
                    NextMonitorCounterClockwise => Ok(AccumMonitor(
                        monitor_setup
                            .next_monitor_counterclockwise(monitor)
                            .unwrap(),
                    )),
                    MonitorCenter => Ok(AccumPoint(monitor.rect.center())),
                    _ => unreachable!(),
                }
            }
        })?;

    match res {
        AccumPoint(point) => println!("{:?}", point),
        AccumMonitor(monitor) => println!("{}", monitor.name()),
    }

    Ok(())
}
