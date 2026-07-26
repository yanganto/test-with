test_with::runner!(ip);

#[test_with::module]
mod ip {
    // Run only when one of the interface ip is in 127.0.0.0/8 (loopback always present)
    #[test_with::runtime_ip_in(127.0.0.0/8)]
    fn test_works_with_loopback() {
        assert!(true);
    }

    // Ignored because no interface ip should be in this documentation-only range
    #[test_with::runtime_ip_in(203.0.113.0/24)]
    fn test_ignored_without_interface_ip() {
        panic!("should be ignored")
    }

    // Run only when the public ip can be got from https://ip.me (default check site)
    #[test_with::runtime_public_ip]
    fn test_works_with_public_ip() {
        assert!(true);
    }

    // Override the check site (url has to be a string literal)
    #[test_with::runtime_public_ip(ip_check_site = "https://ifconfig.me/ip")]
    fn test_works_with_public_ip_other_site() {
        assert!(true);
    }

    // Ignored because the public ip should not be in this documentation-only range
    #[test_with::runtime_public_ip_in(203.0.113.0/24)]
    fn test_ignored_public_ip_not_in_cidr() {
        panic!("should be ignored")
    }

    // Ignored because the public ip should not be this documentation-only ip
    #[test_with::runtime_public_ip_is(203.0.113.1)]
    fn test_ignored_public_ip_is_not() {
        panic!("should be ignored")
    }
}
