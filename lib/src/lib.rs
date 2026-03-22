//! `test_with` provides [macro@env], [macro@file], [macro@path], [macro@http], [macro@https],
//! [macro@icmp], [macro@tcp], [macro@root], [macro@group], [macro@user], [macro@mem], [macro@swap],
//! [macro@cpu_core], [macro@phy_core], [macro@executable], [macro@timezone] macros to help you run
//! test case only with the condition is fulfilled.  If the `#[test]` is absent for the test case,
//! `#[test_with]` will add it to the test case automatically.
//!
//! This crate help you easier make integrating test case and has a good cargo summary on CI server,
//! and will not affect on your binary output when you dependent it as dev-dependency as following.
//! ```toml
//! [dev-dependencies]
//! test-with = "*"
//! ```
//! All features will be opt-in default feature, so this crate will be easier to use, if you using
//! a CI server with really limitation resource and want this crate as slim as possible, you can
//! select the feature you want as following.
//! ```toml
//! [dev-dependencies]
//! test-with = { version = "*", default-features = false, features = ["net"] }
//! ```
//!
//! The solution to have a real runtime condition check, we need to put the test as normal function
//! as an example, then use `cargo run --example`
//! The `test-with` need be included as normal dependency with `runtime` feature.
//! And also include the `libtest-with` with corresponding features in `Cargo.toml`
//! [macro@runner] and [macro@module] are for the basic skeleton of the test runner.
//! [macro@runtime_env], [macro@runtime_no_env], [macro@runtime_file], [macro@runtime_path],
//! [macro@runtime_http], [macro@runtime_https], [macro@runtime_icmp], [macro@runtime_tcp],
//! [macro@runtime_root], [macro@runtime_group], [macro@runtime_user], [macro@runtime_mem],
//! [macro@runtime_free_mem], [macro@runtime_available_mem], [macro@runtime_swap],
//! [macro@runtime_free_swap], [macro@runtime_available_swap], [macro@runtime_cpu_core],
//! [macro@runtime_phy_core], [macro@runtime_executable], [macro@runtime_timezone]
//! and [macro@runtime_ignore_if] are used to transform a normal function to a testcase.
//! The libtest-mimic support runtime ignore after 0.8.2, we do not need libtest-with any more,
//! and all runtime feature is still compatible and easy to use.
//!
//! ```toml
//! [dependencies]
//! test-with = { version = "*", default-features = false, features = ["runtime"] }
//! ```
//!
//! ```rust
//! // write as example in examples/*rs
//! test_with::runner!(env);
//! #[test_with::module]
//! mod env {
//! #[test_with::runtime_env(PWD)]
//! fn test_works() {
//!     assert!(true);
//!     }
//! }
//! ```

pub use test_with_derive::*;

#[cfg(feature = "runtime")]
pub use libtest_mimic::*;

#[cfg(all(feature = "runtime", feature = "resource"))]
pub use byte_unit;
#[cfg(all(feature = "runtime", feature = "timezone"))]
pub use chrono;
#[cfg(all(feature = "runtime", feature = "resource"))]
pub use num_cpus;
#[cfg(all(feature = "runtime", feature = "icmp"))]
pub use ping;
#[cfg(all(feature = "runtime", feature = "http"))]
pub use reqwest;
#[cfg(all(feature = "runtime", feature = "resource"))]
pub use sysinfo;
#[cfg(all(feature = "runtime", feature = "user", not(target_os = "windows")))]
pub use uzers;
#[cfg(all(feature = "runtime", feature = "executable"))]
pub use which;
