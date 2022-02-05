fn main() {}

#[cfg(test)]
mod tests {
    #[test_with::root()]
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }
}
