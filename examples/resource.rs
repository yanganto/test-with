fn main() {}

#[cfg(test)]
mod tests {
    #[test_with::mem(999GB)]
    #[test]
    fn test_ignored() {
        panic!("should be ignored")
    }
}
