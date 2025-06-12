// Use tokio_runner for tokio tests
// Besides, use `test-with-async` feature for this example
test_with::tokio_runner!(
    env_tests,
    file_tests,
    path_tests,
    net_tests,
    user_tests,
    exe_tests,
    resource_tests,
    custom_mod,
    timezone_tests
);

#[test_with::module]
mod env_tests {
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

#[test_with::module]
mod file_tests {
    #[test_with::runtime_file(/etc/hostname)]
    async fn file_test_works() {
        assert!(true);
    }
}

#[test_with::module]
mod path_tests {
    #[test_with::runtime_path(/no_existing)]
    async fn test_not_works() {
        assert!(true);
    }
}

#[test_with::module]
mod net_tests {
    #[test_with::runtime_http(httpbin.org)]
    async fn http_test_works() {
        assert!(true);
    }
    #[test_with::runtime_https(httpbin.org)]
    async fn https_test_works() {
        assert!(true);
    }
    #[test_with::runtime_icmp(193.194.195.196)]
    async fn test_ignored_with_non_existing_host() {
        panic!("should be ignored with non existing host")
    }
    #[test_with::runtime_tcp(8.8.8.8:53)]
    async fn test_works_with_domain_name_server() {
        assert!(true);
    }
}

#[test_with::module]
mod user_tests {
    #[test_with::runtime_root()]
    async fn test_ignored_by_normal_user() {
        panic!("should be ignored")
    }
    #[test_with::runtime_group(avengers)]
    async fn test_ignored_by_normal_person() {
        panic!("should be ignored")
    }
    #[test_with::runtime_user(spider)]
    async fn test_ignored_by_normal_man() {
        panic!("should be ignored")
    }
}

#[test_with::module]
mod exe_tests {
    // `/bin/sh` executable exists
    #[test_with::runtime_executable(/bin/sh)]
    async fn test_executable_with_path() {
        assert!(true);
    }
}

#[test_with::module]
mod resource_tests {
    // Only works with enough cpu core
    #[test_with::runtime_cpu_core(32)]
    async fn test_ignored_core_not_enough() {
        panic!("should be ignored")
    }

    // Only works with enough physical cpu core
    #[test_with::runtime_phy_cpu_core(32)]
    async fn test_ignored_phy_core_not_enough() {
        panic!("should be ignored")
    }

    // Only works with enough memory size
    #[test_with::runtime_mem(100GB)]
    async fn test_ignored_mem_not_enough() {
        panic!("should be ignored")
    }

    // Only works with enough free memory size
    #[test_with::runtime_free_mem(100GB)]
    async fn test_ignored_free_mem_not_enough() {
        panic!("should be ignored")
    }

    // Only works with enough available memory size
    #[test_with::runtime_available_mem(100GB)]
    async fn test_ignored_available_mem_not_enough() {
        panic!("should be ignored")
    }

    // Only works with enough swap size
    #[test_with::runtime_swap(100GB)]
    async fn test_ignored_swap_not_enough() {
        panic!("should be ignored")
    }

    // Only works with enough free swap size
    #[test_with::runtime_free_swap(100GB)]
    async fn test_ignored_free_swap_not_enough() {
        panic!("should be ignored")
    }
}

fn something_happened() -> Option<String> {
    Some("because something happened".to_string())
}

#[test_with::module]
mod custom_mod {
    #[test_with::runtime_ignore_if(something_happened)]
    async fn test_ignored() {
        assert!(false);
    }
}

#[test_with::module]
mod timezone_tests {
    #[test_with::runtime_timezone(0)]
    async fn timezone_test_works() {
        assert!(true);
    }
    #[test_with::runtime_timezone(-1)]
    async fn timezone_test_ignored() {
        assert!(false);
    }
}
