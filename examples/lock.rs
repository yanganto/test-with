fn main() {}

#[cfg(test)]
mod tests {
    // `LOCK` is file based lock to prevent test1 an test2 run at the same time
    #[test_with::lock(LOCK)]
    fn test_1() {
        assert!(true);
    }

    // `LOCK` is file based lock to prevent test1 an test2 run at the same time
    #[test_with::lock(LOCK)]
    fn test_2() {
        assert!(true);
    }

    // `ANOTHER_LOCK` is file based lock to prevent test3 an test4 run at the same time with 3 sec
    // waiting time.
    #[test_with::lock(ANOTHER_LOCK, 3)]
    fn test_3() {
        assert!(true);
    }

    // `ANOTHER_LOCK` is file based lock to prevent test3 an test4 run at the same time with 3 sec
    // waiting time.
    #[test_with::lock(ANOTHER_LOCK, 3)]
    fn test_4() {
        assert!(true);
    }
}
