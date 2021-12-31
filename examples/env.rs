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
}
