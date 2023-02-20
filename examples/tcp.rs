fn main() {}

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
