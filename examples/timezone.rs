fn main() {}

#[cfg(test)]
mod timezone_tests {
    #[test_with::timezone(0)]
    #[test]
    fn test_works() {
        assert!(true);
    }

    #[test_with::timezone(-1)]
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }
}

#[test_with::timezone(0)]
pub mod workable_timezone_mod {
    #[test]
    fn test_works() {
        assert!(true);
    }
}

#[test_with::timezone(-1)]
pub mod ignore_pub_timezone_mod {
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }
}

#[test_with::timezone(-1)]
mod ignore_private_timezone_mod {
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }
}

#[test_with::timezone(-1)]
#[cfg(test)]
pub mod ignore_pub_test_http_mod {
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }
}

#[test_with::timezone(-1)]
#[cfg(test)]
mod ignore_private_test_timezone_mod {
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }
}
