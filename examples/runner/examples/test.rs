test_with::runner!(env, file, path);

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

    #[test_with::runtime_no_env(GITHUB_ACTIONS)]
    fn test_ignore_in_github_action() {
        //This will be ignored in GITHUB_ACTION;
    }
}

#[test_with::module]
mod file {
    #[test_with::runtime_file(/etc/hostname)]
    fn test_works() {
        assert!(true);
    }
}

#[test_with::module]
mod path {
    #[test_with::runtime_path(/no_existing)]
    fn test_not_works() {
        assert!(true);
    }
}
