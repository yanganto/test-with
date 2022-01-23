#[cfg(feature = "example")]
#[tokio::main]
async fn main() {}

#[cfg(feature = "example")]
#[tokio::test]
async fn my_test() {
    assert!(true);
}
