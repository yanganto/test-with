fn main() {}

#[cfg(test)]
mod tests {
    #[test_with::mem(999GB)]
    #[test]
    fn mem_test_ignored() {
        panic!("should be ignored")
    }

    #[test_with::swap(999GB)]
    #[test]
    fn swap_test_ignored() {
        panic!("should be ignored")
    }
}
