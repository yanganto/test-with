test_with::runner!(env, file, path, net, user);

#[test_with::module]
mod env {
    #[test_with::runtime_env(PWD)]
    fn env_test_works() {
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
    fn env_test_works_too() {
        assert!(true);
    }

    #[test_with::runtime_env(PWD, NOT_SAYING)]
    fn test_ignored_too() {
        panic!("should be ignored")
    }

    #[test_with::runtime_no_env(GITHUB_ACTIONS)]
    fn test_ignore_in_github_action() {
        panic!("should be ignored in github action")
    }
}

#[test_with::module]
mod file {
    #[test_with::runtime_file(/etc/hostname)]
    fn file_test_works() {
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

#[test_with::module]
mod net {
    #[test_with::runtime_http(httpbin.org)]
    fn http_test_works() {
        assert!(true);
    }
    #[test_with::runtime_https(httpbin.org)]
    fn https_test_works() {
        assert!(true);
    }
    #[test_with::runtime_icmp(193.194.195.196)]
    fn test_ignored_with_non_existing_host() {
        panic!("should be ignored with non existing host")
    }
    #[test_with::runtime_tcp(8.8.8.8:53)]
    fn test_works_with_domain_name_server() {
        assert!(true);
    }
}

#[test_with::module]
mod user {
    #[test_with::runtime_root()]
    fn test_ignored_by_normal_user() {
        panic!("should be ignored")
    }
    #[test_with::runtime_group(avengers)]
    fn test_ignored_by_normal_person() {
        panic!("should be ignored")
    }
}
