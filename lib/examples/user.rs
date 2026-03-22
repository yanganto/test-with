fn main() {}

#[cfg(test)]
#[cfg(not(target_os = "windows"))]
mod tests {
    #[test_with::root()]
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }

    #[test_with::group(avengers)]
    #[test]
    fn test_ignored2() {
        panic!("should be ignored")
    }

    #[test_with::user(spider)]
    #[test]
    fn test_ignored3() {
        panic!("should be ignored")
    }
}
