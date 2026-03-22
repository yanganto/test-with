use tokio;

#[tokio::main]
async fn main() {}

#[cfg(test)]
mod tokio_tests {
    #[test_with::env(NOTHING)]
    #[tokio::test]
    async fn my_test() {
        assert!(false);
    }
}
