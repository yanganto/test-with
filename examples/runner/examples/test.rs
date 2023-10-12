test_with::runner!(env, file, path, net, user, exe, resource, custom_mod);

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
    #[test_with::runtime_user(spider)]
    fn test_ignored_by_normal_man() {
        panic!("should be ignored")
    }
}

#[test_with::module]
mod exe {
    // `/bin/sh` executable exists
    #[test_with::runtime_executable(/bin/sh)]
    fn test_executable_with_path() {
        assert!(true);
    }
}

#[test_with::module]
mod resource {
    // Only works with enough cpu core
    #[test_with::runtime_cpu_core(32)]
    fn test_ignored_core_not_enough() {
        panic!("should be ignored")
    }

    // Only works with enough physical cpu core
    #[test_with::runtime_phy_cpu_core(32)]
    fn test_ignored_phy_core_not_enough() {
        panic!("should be ignored")
    }

    // Only works with enough memory size
    #[test_with::runtime_mem(100GB)]
    fn test_ignored_mem_not_enough() {
        panic!("should be ignored")
    }

    // Only works with enough free memory size
    #[test_with::runtime_free_mem(100GB)]
    fn test_ignored_free_mem_not_enough() {
        panic!("should be ignored")
    }

    // Only works with enough available memory size
    #[test_with::runtime_available_mem(100GB)]
    fn test_ignored_available_mem_not_enough() {
        panic!("should be ignored")
    }

    // Only works with enough swap size
    #[test_with::runtime_swap(100GB)]
    fn test_ignored_swap_not_enough() {
        panic!("should be ignored")
    }

    // Only works with enough free swap size
    #[test_with::runtime_free_swap(100GB)]
    fn test_ignored_free_swap_not_enough() {
        panic!("should be ignored")
    }
}

fn something_happend() -> Option<String> {
    Some("because something happened".to_string())
}

#[test_with::module]
mod custom_mod {
    #[test_with::runtime_ignore_if(something_happend)]
    fn test_ignored() {
        assert!(false);
    }
}
