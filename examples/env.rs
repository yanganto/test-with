fn main() {}

#[cfg(test)]
mod tests {
    #[test_with::env(PWD)]
    fn test_works() {
        assert!(true);
    }

    #[test_with::env(NOTHING)]
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }

    #[test_with::env(PWD, SAYING)]
    #[test]
    fn test_works_too() {
        assert!(true);
    }

    #[test_with::env(PWD, NOT_SAYING)]
    #[test]
    fn test_ignored_too() {
        panic!("should be ignored")
    }

    #[test_with::no_env(GITHUB_ACTIONS)]
    #[test]
    fn test_ignore_in_github_action() {
        println!("should be ignored in GITHUB_ACTION");
    }
}

#[test_with::env(PWD)]
pub mod workable_mod {
    #[test]
    fn test_works() {
        assert!(true);
    }
}

#[test_with::env(NOTHING)]
pub mod ignore_pub_mod {
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }
}

#[test_with::env(NOTHING)]
mod ignore_private_mod {
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }
}

#[test_with::env(NOTHING)]
#[cfg(test)]
pub mod ignore_pub_test_mod {
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }
}

#[test_with::env(NOTHING)]
#[cfg(test)]
mod ignore_private_test_mod {
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }
}
