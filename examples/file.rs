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
    fn test_ignored() {
        panic!("should be ignored")
    }

    // hostname and hosts exist
    #[test_with::file(/etc/hostname, /etc/hosts)]
    fn test_works_too() {
        assert!(true);
    }

    // nothing file does not exist
    #[test_with::file(/etc/hostname, /etc/nothing)]
    fn test_ignored_too() {
        panic!("should be ignored")
    }

    // etc exists, but not file
    #[test_with::file(/etc)]
    fn test_ignored_for_file() {
        panic!("should be ignored")
    }

    // hostname exists
    #[test_with::path(/etc/hostname)]
    fn test_works_for_file() {
        assert!(true);
    }

    // etc exists
    #[test_with::path(/etc)]
    fn test_works_for_path() {
        assert!(true);
    }

    // nothing does not exist
    #[test_with::path(/nothing)]
    fn test_ignored_for_path() {
        panic!("should be ignored")
    }

    // etc and tmp exist
    #[test_with::path(/etc, /tmp)]
    fn test_works_for_paths_too() {
        assert!(true);
    }

    // nothing does not exist
    #[test_with::file(/etc, /nothing)]
    fn test_ignored_for_paths_too() {
        panic!("should be ignored")
    }
}
