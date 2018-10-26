// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! provides support for initializing logging for testing purposes

/// Generates a module named `tests`, which provides test support functionality.
/// - `run_test` function ensures logging is configured for the [log crate](https://crates.io/crates/log)
///   - the root log level will be set to Warn
///   - the crate's log level will be set to Debug
/// - `run_test` function will log the test execution time
/// - `run_test` will be bound to the crate's root path, i.e., it can be invoked as `::run_test("test_name",|| { ... })`,
///    but annotated with `#[cfg(test)]`. Thus, it will only be available when running tests.
///
/// ## Example
/// ```rust
///
/// #[cfg(test)]
/// #[macro_use]
/// extern crate oysterpack_testing;
///
/// #[cfg(test)]
/// op_tests_mod!();
///
/// #[test]
/// fn foo_test() {
///     ::run_test("foo_test", || info!("SUCCESS"));
/// }
///
/// # fn main(){}
///
/// ```
///
/// ## Example - configuring target log levels
/// ```rust
///
/// #[cfg(test)]
/// #[macro_use]
/// extern crate oysterpack_testing;
///
/// #[cfg(test)]
/// op_tests_mod! {
///     "foo" => Info,
///     "bar" => Error
/// }
///
/// #[test]
/// fn foo_test() {
///     ::run_test("foo_test", || info!(target: "foo", "SUCCESS"));
/// }
///
/// # fn main(){}
///
/// ```
/// - in the above example, the `foo` target log level is set to `Info` and the `bar` target log level
///   is set to Error
#[macro_export]
macro_rules! op_tests_mod {
    ( $($target:expr => $level:ident),* ) => {
        #[cfg(test)]
        pub(crate) mod tests {

            /// Used to track logging initialization
            #[derive(Eq, PartialEq)]
            pub enum LogInitState {
                NotInitialized,
                Initializing,
                Initialized,
            }

            pub static mut _FERN_INITIALIZED: LogInitState = LogInitState::NotInitialized;

            fn init_log() {
                unsafe {
                    if _FERN_INITIALIZED == LogInitState::NotInitialized {
                        _FERN_INITIALIZED = LogInitState::Initializing;
                        if _FERN_INITIALIZED == LogInitState::Initializing {
                            const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");
                            let _ = $crate::fern::Dispatch::new()
                                .format(|out, message, record| {
                                    out.finish(format_args!(
                                        "{}[{}][{}][{}:{}] {}",
                                        $crate::chrono::Local::now().format("[%H:%M:%S%.3f]"),
                                        record.level(),
                                        record.target(),
                                        record.file().unwrap(),
                                        record.line().unwrap(),
                                        message
                                    ))
                                }).level($crate::log::LevelFilter::Warn)
                                .level_for(CARGO_PKG_NAME, $crate::log::LevelFilter::Debug)
                                .chain(::std::io::stdout())
                                $(
                                .level_for($target,$crate::log::LevelFilter::$level)
                                )*
                                .apply();
                            _FERN_INITIALIZED = LogInitState::Initialized;
                            info!("logging has been initialized for {}", CARGO_PKG_NAME);
                        }
                    }
                    // There may be a race condition because tests may run in parallel.
                    // Thus, wait until logging has been initialized before running the test.
                    while _FERN_INITIALIZED != LogInitState::Initialized {
                        ::std::thread::yield_now();
                    }
                }
            }

            /// - ensures logging is configured and initialized
            /// - collects test execution time and logs it
            pub fn run_test<F: FnOnce() -> ()>(name: &str, test: F) {
                init_log();
                let before = ::std::time::Instant::now();
                test();
                let after = ::std::time::Instant::now();
                info!(
                    "{}: test run time: {:?}",
                    name,
                    after.duration_since(before)
                );
            }

            #[test]
            fn compiles() {
                run_test("compiles", || info!("it compiles :)"));
            }
        }

        #[cfg(test)]
        pub use tests::run_test;
    };
}

/// Creates a test function, which executes the specified expression block.
/// It reduces some boilerplate and hides the internals of the `tests` module
/// generated by [op_tests_mod](macro.op_tests_mod.html).
///
/// Metadata attributes can be specified on the test function, e.g., `#[ignore]`
///
/// ## Example
/// ```rust
///
/// #[cfg(test)]
/// #[macro_use]
/// extern crate oysterpack_testing;
///
/// #[cfg(test)]
/// op_tests_mod!();
///
/// #[cfg(test)]
/// mod foo_test {
///    op_test!(foo, {
///      info!("SUCCESS");
///    });
/// }
///
#[macro_export]
macro_rules! op_test {
    (
        $(#[$outer:meta])*
        $Name:ident $Fn:block
    ) => {
        #[test]
        fn $Name() {
            ::tests::run_test(stringify!($Name), || $Fn);
        }
    };
}

#[cfg(test)]
mod tests {

    use tests::run_test;

    #[test]
    fn tests_op_test() {
        run_test("tests_op_test", || {
            info!("tests_op_test passed !!!");
            info!(target: "foo", "foo info");
            info!(target: "bar", "*** bar info should not be logged ***");
            error!(target: "bar", "bar error");
        });
    }

    #[test]
    fn test_op_test_fn() {
        run_test("test_op_test_fn", test_bar);
    }

    fn test_bar() {
        info!("bar passed !!!")
    }

    op_test!(bar {test_bar()});
}
