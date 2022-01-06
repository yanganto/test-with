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
