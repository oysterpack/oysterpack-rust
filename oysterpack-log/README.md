Standardizes logging for the OysterPack platform on top of [log](https://crates.io/crates/log).
Given a LogConfig, this crate will know how to initialize the logging system and how to shut it down.

```rust
#[macro_use]
extern crate oysterpack_app_metadata_macros;

op_build_mod!();

fn main() {
    let app_build = build::get();
    oysterpack_log::init(log_config(),&app_build);
    // The LogConfig used to initialize the log system can be retrieved.
    // This enables the LogConfig to be inspected.
    let log_config = oysterpack_log::config().unwrap();

    run();

    oysterpack_log::shutdown();
}

/// This should be loaded from the app's configuration.
/// For this simple example, we are simply using the default LogConfig.
/// The default LogConfig sets the root log level to Warn and logs to stdout.
fn log_config() -> oysterpack_log::LogConfig {
    Default::default()
}

fn run() {}
```