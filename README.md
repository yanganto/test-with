# test-with
[![Crates.io][crates-badge]][crate-url]
[![MIT licensed][mit-badge]][mit-url]
[![Docs][doc-badge]][doc-url]

A lib help you run test with conditions, else the test will be ignored.

## Preamble
This crate provide a workable solution for this [issue][original-issue] of rust-lang.

Currently, the condition is checked on build-time not runtime and not perfect,
because of this [issue][original-issue] of rust-lang.
To avoid [known issue][known-issue] at this moment,
please clean before running test.
```bash
cargo clean; SOME_VAR=true cargo test
```

If you forget to add `#[test]` flag on the test case, `#[test_with]` macro will add it for you.

Before Rust version `1.61`, the ignore message does not show, such that the feature `ign-msg` can be used,
and the name of ignored test case will be rewritten, such that you can easier to know why the test is ignored.

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
or run all test cases for test module when the environment variable is set.
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

Result of `cargo test`
```text
running 2 tests
test tests::test_ignored ... ignored
test tests::test_works ... ok

test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

If the test depends on more than one environment variables,
you can write it with multiple variables, `#[test_with::env(VAR1, VAR2)]`.

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

## Memory/Swap condition
Run integration test case when the memory/swap is enough
```rust
#[test_with::mem(999GB)]
#[test]
fn test_ignored() {
    panic!("should be ignored")
}

#[test_with::swap(999GB)]
#[test]
fn test_ignored() {
    panic!("should be ignored")
}
```

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
