fn main() {}

#[cfg(test)]
mod tests {
    #[test_with::env(PWD)]
    fn test_works() {
        assert!(true);
    }

    #[test_with::env(NOTHING)]
    fn test_ignored() {
        panic!("should be ignored")
    }

    #[test_with::env(PWD, SAYING)]
    fn test_works_too() {
        assert!(true);
    }

    #[test_with::env(PWD, NOT_SAYING)]
    fn test_ignored_too() {
        panic!("should be ignored")
    }
}
