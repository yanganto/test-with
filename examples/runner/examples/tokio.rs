// use tokio_runner for tokio tests
test_with::tokio_runner!(tokio_tests);

#[test_with::module]
mod tokio_tests {
    #[test_with::runtime_env(PWD)]
    async fn test_tokio_runtime() {
        assert!(true);
    }
    #[test_with::runtime_env(NOTHING)]
    async fn test_tokio_runtime_ignore() {
        assert!(false);
    }
    #[test_with::runtime_env(PWD)]
    async fn test_tokio_runtime_with_result() -> Result<(), ()> {
        assert!(true);
        Ok(())
    }
}
