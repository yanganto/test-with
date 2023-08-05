test_with::runner!(env);

#[test_with::module]
mod env {
    #[test_with::runtime_env(PWD)]
    fn test_works() {
        assert!(true);
    }

    // Will rase error when using `#[test]`
    //
    // #[test_with::runtime_env(PWD)]
    // #[test]
    // fn test_works() {
    //     assert!(true);
    // }

    #[test_with::runtime_env(NOTHING)]
    fn test_ignored() {
        panic!("should be ignored")
    }

    // Will rase error when using non-runtime macro
    //
    // #[test_with::env(PWD, SAYING)]
    // fn test_works_too() {
    //     assert!(true);
    // }

    #[test_with::runtime_env(PWD, SAYING)]
    fn test_works_too() {
        assert!(true);
    }

    #[test_with::runtime_env(PWD, NOT_SAYING)]
    fn test_ignored_too() {
        panic!("should be ignored")
    }
}
