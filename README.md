# test-with
[![Crates.io][crates-badge]][crate-url]
[![MIT licensed][mit-badge]][mit-url]
[![Docs][doc-badge]][doc-url]

A lib help you run test with conditions, else the test will be ignored with clear message.

## Introduction
It is good to use this crate in dev dependency as following
```toml
[dev-dependencies]
test-with = "*"
```

If you want the dependency smaller with a shorter compiling time, you can disable default features and use specific one.
For example, if you only checking a remote web server, you can use the `net` or `http` feature as the following.
```toml
[dev-dependencies]
test-with = { version = "*", default-features = false, features = ["http"] }
```
The features you can use are `net`(`http`, `icmp`), `resource`, `user`, `executable`.

Currently, the condition is checked on build-time not runtime and not perfect and good for most develop scenario,
because of this [issue][original-issue] of rust-lang.
Here is the [slides][coscup-slides] of a talk in COSCUP and help you know more about it.
If you really want to check the condition in runtime, please check [runtime section](https://github.com/yanganto/test-with#runtime).
The `runtime` feature and runtime macros (`test_with::runner!`, `#[test_with::module]`, `#[test_with::runtime_env()]`) can help you run the test and check the conditions in runtime.

If you forget to add `#[test]` flag on the test case, `#[test_with]` macro will add it for you.

Rust version `1.61` of stable channel or `2022-03-30` of nightly channel will show the ignore message.
If the ignore message does not show in the previous Rust version you used, the feature `ign-msg` can be used to work around.
and the name of ignored test case will be rewritten, such that you can easier to know why the test is ignored.

The order of test macros(`#[test]`, `#[tokio::test]`, `#[serial_test::serial]`, ...) is important, please check out examples.

## Environment Variable
Run test case when the environment variable is set.

```rust
// PWD environment variable exists
#[test_with::env(PWD)]
#[test]
fn test_works() {
    assert!(true);
}

// NOTHING environment variable does not exist
#[test_with::env(NOTHING)]
#[test]
fn test_ignored() {
    panic!("should be ignored")
}
```

Result of `cargo test`
```text
running 2 tests
test tests::test_ignored ... ignored, because following variable not found: NOTHING
test tests::test_works ... ok

test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Or run all test cases for test module when the environment variable is set.
```rust
#[test_with::env(PWD)]
#[cfg(test)]
mod tests {

    #[test]
    fn test_works() {
        assert!(true);
    }
}
```

If the test depends on more than one environment variables,
you can write it with multiple variables, `#[test_with::env(VAR1, VAR2)]`.

Also, the test case can be ignored with the specific environment variable.

```rust
// The test will be ignored in Github actions.
#[test_with::no_env(GITHUB_ACTIONS)]
#[test]
fn test_ignore_in_github_action() {
    println!("Should be ignored in GITHUB_ACTION");
}
```

## File/Folder
Run test case when the file or folder exist.  This is good for testing with database config.
If you want to check the folder exist or not, please use `path`.

```rust
// hostname exists
#[test_with::file(/etc/hostname)]
#[test]
fn test_works() {
    assert!(true);
}

// nothing file does not exist
#[test_with::file(/etc/nothing)]
#[test]
fn test_ignored() {
    panic!("should be ignored")
}

// etc exists
#[test_with::path(/etc)]
#[test]
fn test_works_for_path() {
    assert!(true);
}
```

If the test depends on more than one file or path,
you can write it with multiple file/path,
`#[test_with::file(/file1, /file2)]` or `#[test_with::path(/folder, /file)]`.

## Http/Https Service
Run test case when the http/https service available.  This is good for integration testing.
Require `http` feature, if default features are disabled.

```rust
// https service exists
#[test_with::https(www.rust-lang.org)]
#[test]
fn test_works() {
    assert!(true);
}

// There is no not.exist.com
#[test_with::https(not.exist.com)]
#[test]
fn test_ignored() {
    panic!("should be ignored")
}
```

If the test depends on more than one service,
you can write it with multiple service,
`#[test_with::http(service1, service2)]` or `#[test_with::http2(service1, service2)]`.

## TCP socket
Run integration test case when the remote tcp socket is listening.

```rust
#[test_with::tcp(8.8.8.8:53)]
#[test]
fn test_works() {
    assert!(true);
}

#[test_with::tcp(193.194.195.196)]
#[test]
fn test_ignored() {
    panic!("should be ignored")
}
```

## Remote Server Online Status
Run integration test case when the remote server online.
**Please note the user running test case should have capability to open socket**.
Require `icmp` feature, if default features are disabled.

```rust
// localhost is online
#[test_with::icmp(127.0.0.1)]
#[test]
fn test_works() {
    assert!(true);
}

// 193.194.195.196 is offline
#[test_with::icmp(193.194.195.196)]
#[test]
fn test_ignored() {
    panic!("should be ignored")
}
```

## User/Group condition
Run integration test case when the user is specific user or in specific group
Require `user` feature, if default features are disabled.
```rust
#[test_with::root()]
#[test]
fn test_ignored() {
    panic!("should be ignored")
}

#[test_with::group(avengers)]
#[test]
fn test_ignored2() {
    panic!("should be ignored")
}

#[test_with::user(spider)]
#[test]
fn test_ignored3() {
    panic!("should be ignored")
}
```

## CPU/Memory/Swap condition
Run integration test case when the memory/swap is enough
Require `resource` feature, if default features are disabled.
```rust
#[test_with::cpu_core(32)]
#[test]
fn test_ignored_by_cpu_core() {
    panic!("should be ignored")
}


#[test_with::phy_core(32)]
#[test]
fn test_ignored_by_physical_cpu_core() {
    panic!("should be ignored")
}

#[test_with::mem(999GB)]
#[test]
fn test_ignored_by_mem() {
    panic!("should be ignored")
}

#[test_with::swap(999GB)]
#[test]
fn test_ignored_by_swap() {
    panic!("should be ignored")
}
```

## Executable condition
Run integration test case when the executables can be accessed
Require `executable` feature, if default features are disabled.
```rust
    // `pwd` executable command exists
    #[test_with::executable(pwd)]
    #[test]
    fn test_executable() {
        assert!(true);
    }

    // `/bin/sh` executable exists
    #[test_with::executable(/bin/sh)]
    #[test]
    fn test_executable_with_path() {
        assert!(true);
    }

    // `non` does not exist
    #[test_with::executable(non)]
    #[test]
    fn test_non_existing_executable() {
        panic!("should be ignored")
    }

    // `pwd` and `ls` exist
    #[test_with::executable(pwd, ls)]
    #[test]
    fn test_executables_too() {
        assert!(true);
    }
```

## Runtime
We can let an example to do thing that cargo test runner do, and ignore testcase in runtime.
The testcase of in the example will not in `#[cfg(test)]` or `#[test]` anymore, and use `#[test_with::runtime_*]`,
the test runner will treat it as the test in Rust and also provide the same summary as `cargo test`.

The `runtime` feature should be enabled and also include the `libtest-with` in `Cargo.toml`
```toml
test-with = { version = "0.10", features = ["runtime"] }
libtest-with = "0.6.1-0"
```

Create an example with the following runtime macros (`test_with::runner!`, `#[test_with::module]`, `#[test_with::runtime_env()]`).
```rust

test_with::runner!(module_name);

#[test_with::module]
mod module_name {
    #[test_with::runtime_env(PWD)]
    fn test_works() {
    }
}

```

Please check out the [example/runner](https://github.com/yanganto/test-with/tree/main/examples/runner).


## Relating issues
* [Solve this in runtime][original-issue]

[crates-badge]: https://img.shields.io/crates/v/test-with.svg
[crate-url]: https://crates.io/crates/test-with
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/yanganto/test-with/blob/readme/LICENSE
[doc-badge]: https://img.shields.io/badge/docs-rs-orange.svg
[doc-url]: https://docs.rs/test-with/latest/test_with/
[original-issue]: https://github.com/rust-lang/rust/issues/68007
[rust-pre-rfc]: https://internals.rust-lang.org/t/pre-rfc-provide-ignore-message-when-the-test-ignored/15904
[known-issue]: https://github.com/yanganto/test-with/issues/18
[coscup-slides]: http://slides.com/yanganto/rust-ignore
