use serial_test::serial;

#[tokio::main]
async fn main() {}

#[serial]
#[test_with::env(NOTHING)]
#[tokio::test]
async fn my_test_1() {
    assert!(false);
}

#[serial]
#[test_with::env(NOTHING)]
#[tokio::test]
async fn my_test_2() {
    assert!(false);
}
