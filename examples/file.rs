fn main() {}

#[cfg(test)]
mod tests {

    // hostname exists
    #[test_with::file(/etc/hostname)]
    fn test_works() {
        assert!(true);
    }

    // nothing file does not exist
    #[test_with::file(/etc/nothing)]
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }

    // hostname and hosts exist
    #[test_with::file(/etc/hostname, /etc/hosts)]
    #[test]
    fn test_works_too() {
        assert!(true);
    }

    // nothing file does not exist
    #[test_with::file(/etc/hostname, /etc/nothing)]
    #[test]
    fn test_ignored_too() {
        panic!("should be ignored")
    }

    // etc exists, but not file
    #[test_with::file(/etc)]
    #[test]
    fn test_ignored_for_file() {
        panic!("should be ignored")
    }

    // hostname exists
    #[test_with::path(/etc/hostname)]
    #[test]
    fn test_works_for_file() {
        assert!(true);
    }

    // etc exists
    #[test_with::path(/etc)]
    #[test]
    fn test_works_for_path() {
        assert!(true);
    }

    // nothing does not exist
    #[test_with::path(/nothing)]
    #[test]
    fn test_ignored_for_path() {
        panic!("should be ignored")
    }

    // etc and tmp exist
    #[test_with::path(/etc, /tmp)]
    #[test]
    fn test_works_for_paths_too() {
        assert!(true);
    }

    // nothing does not exist
    #[test_with::file(/etc, /nothing)]
    #[test]
    fn test_ignored_for_paths_too() {
        panic!("should be ignored")
    }
}
