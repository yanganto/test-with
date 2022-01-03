# test-with
[![Crates.io][crates-badge]][crate-url]
[![MIT licensed][mit-badge]][mit-url]
[![Docs][doc-badge]][doc-url]

A lib help you run test with condition

## Environment variable
Run test case when the environment variable is set.
A solution for this [issue][original-issue] of rust-lang.

```rust
#[cfg(test)]
mod tests {

    // PWD environment variable exists
    #[test_with::env(PWD)]
    fn test_works() {
        assert!(true);
    }

    // NOTHING environment variable does not exist
    #[test_with::env(NOTHING)]
    fn test_ignored() {
        panic!("should be ignored")
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

## Relating issues
* [Solve this in runtime][original-issue]
* [provide ignore message in cargo][rust-pre-rfc]

[crates-badge]: https://img.shields.io/crates/v/test-with.svg
[crate-url]: https://crates.io/crates/test-with
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/yanganto/test-with/blob/readme/LICENSE
[doc-badge]: https://img.shields.io/badge/docs-rs-orange.svg
[doc-url]: https://docs.rs/test-with/0.1.0/test_with/
[original-issue]: https://github.com/rust-lang/rust/issues/68007
[rust-pre-rfc]: https://internals.rust-lang.org/t/pre-rfc-provide-ignore-message-when-the-test-ignored/15904
