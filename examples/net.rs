fn main() {}

#[cfg(test)]
mod http_tests {
    #[test_with::http(httpbin.org)]
    fn test_works() {
        assert!(true);
    }

    #[test_with::https(not.exist.com)]
    fn test_ignored() {
        panic!("should be ignored")
    }
}

#[cfg(test)]
mod https_tests {
    #[test_with::https(www.rust-lang.org)]
    fn test_works() {
        assert!(true);
    }

    #[test_with::https(not.exist.com)]
    fn test_ignored() {
        panic!("should be ignored")
    }
}

#[cfg(test)]
mod icmp_tests {
    #[test_with::icmp(127.0.0.1)]
    fn test_works() {
        assert!(true);
    }

    #[test_with::icmp(193.194.195.196)]
    fn test_ignored() {
        panic!("should be ignored")
    }
}
