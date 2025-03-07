fn main() {}

#[cfg(test)]
mod tests {
    // `pwd` executable command exists
    #[test_with::executable(pwd)]
    #[test]
    fn test_executable() {
        assert!(true);
    }

    // `/bin/sh` executable exists
    #[test_with::executable(/bin/sh)]
    #[test]
    fn test_executable_with_path() {
        assert!(true);
    }

    // `non` does not exist
    #[test_with::executable(non)]
    #[test]
    fn test_non_existing_executable() {
        panic!("should be ignored")
    }

    // `pwd` and `ls` exist
    #[test_with::executable(pwd, ls)]
    #[test]
    fn test_executables_too() {
        assert!(true);
    }

    // `non` or `ls` exist
    #[test_with::executable(non || ls)]
    #[test]
    fn test_one_of_executables_exist() {
        assert!(true);
    }
}
