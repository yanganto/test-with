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
