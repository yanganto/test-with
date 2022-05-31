use tokio;

#[tokio::main]
async fn main() {}

#[cfg(test)]
mod tokio_tests {
    #[tokio::test]
    async fn my_test() {
        assert!(true);
    }
}