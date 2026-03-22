fn main() {}

#[cfg(test)]
mod icmp_tests {
    #[test_with::icmp(127.0.0.1)]
    #[test]
    fn test_works() {
        assert!(true);
    }

    #[test_with::icmp(::1)]
    #[test]
    fn test_ipv6_works() {
        assert!(true);
    }

    #[test_with::icmp(193.194.195.196)]
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }
}

#[test_with::icmp(127.0.0.1)]
pub mod workable_icmp_mod {
    #[test]
    fn test_works() {
        assert!(true);
    }
}

#[test_with::icmp(193.194.195.196)]
pub mod ignore_pub_icmp_mod {
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }
}

#[test_with::icmp(193.194.195.196)]
mod ignore_private_icmp_mod {
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }
}

#[test_with::icmp(193.194.195.196)]
#[cfg(test)]
pub mod ignore_pub_test_icmp_mod {
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }
}

#[test_with::icmp(193.194.195.196)]
#[cfg(test)]
mod ignore_private_test_icmp_mod {
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }
}
