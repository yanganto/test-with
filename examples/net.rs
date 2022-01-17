fn main() {}

// HTTP

#[cfg(test)]
mod http_tests {
    #[test_with::http(httpbin.org)]
    #[test]
    fn test_works() {
        assert!(true);
    }

    #[test_with::http(not.exist.com)]
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }
}

#[test_with::http(httpbin.org)]
pub mod workable_http_mod {
    #[test]
    fn test_works() {
        assert!(true);
    }
}

#[test_with::http(not.exist.com)]
pub mod ignore_pub_http_mod {
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }
}

#[test_with::http(not.exist.com)]
mod ignore_private_http_mod {
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }
}

#[test_with::http(not.exist.com)]
#[cfg(test)]
pub mod ignore_pub_test_http_mod {
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }
}

#[test_with::http(not.exist.com)]
#[cfg(test)]
mod ignore_private_test_http_mod {
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }
}

// HTTPS

#[cfg(test)]
mod https_tests {
    #[test_with::https(www.rust-lang.org)]
    #[test]
    fn test_works() {
        assert!(true);
    }

    #[test_with::https(not.exist.com)]
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }
}

#[test_with::https(httpbin.org)]
pub mod workable_https_mod {
    #[test]
    fn test_works() {
        assert!(true);
    }
}

#[test_with::https(not.exist.com)]
pub mod ignore_pub_https_mod {
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }
}

#[test_with::https(not.exist.com)]
mod ignore_private_https_mod {
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }
}

#[test_with::https(not.exist.com)]
#[cfg(test)]
pub mod ignore_pub_test_https_mod {
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }
}

#[test_with::https(not.exist.com)]
#[cfg(test)]
mod ignore_private_test_https_mod {
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }
}

// ICMP

#[cfg(test)]
mod icmp_tests {
    #[test_with::icmp(127.0.0.1)]
    #[test]
    fn test_works() {
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

// TCP

#[cfg(test)]
mod tcp_tests {
    #[test_with::tcp(8.8.8.8:53)]
    #[test]
    fn test_works() {
        assert!(true);
    }

    #[test_with::tcp(193.194.195.196)]
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }
}

#[test_with::tcp(8.8.8.8:53)]
pub mod workable_tcp_mod {
    #[test]
    fn test_works() {
        assert!(true);
    }
}

#[test_with::tcp(193.194.195.196)]
pub mod ignore_pub_tcp_mod {
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }
}

#[test_with::tcp(193.194.195.196)]
mod ignore_private_tcp_mod {
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }
}

#[test_with::tcp(193.194.195.196)]
#[cfg(test)]
pub mod ignore_pub_test_tcp_mod {
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }
}
