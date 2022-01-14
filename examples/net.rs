fn main() {}

#[cfg(test)]
mod http_tests {
    #[test_with::http(httpbin.org)]
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
