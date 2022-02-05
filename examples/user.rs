fn main() {}

#[cfg(test)]
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
}
