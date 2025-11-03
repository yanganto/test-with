fn main() {}

#[cfg(feature = "rhai")]
#[test_with::rhai()]
fn test_rhai() {
    // The code is rhai syntax, not Rust anymore
    let a = 10;
    let b = a + 1;
    print("Hello world! b = " + b);
}
