use std::ops::Drop;
use std::process::{Child, Command};

test_with::runner!(net);

pub struct Moc {
    p: Child,
}

impl Default for Moc {
    fn default() -> Moc {
        let p = Command::new("python")
            .args(["-m", "http.server"])
            .spawn()
            .expect("failed to execute child");
        let mut count = 0;
        while count < 3 {
            if libtest_with::reqwest::blocking::get("http://127.0.0.1:8000").is_ok() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
            count += 1;
        }
        Moc { p }
    }
}

impl Drop for Moc {
    fn drop(&mut self) {
        self.p.kill().expect("fail to kill python http.server");
    }
}

#[test_with::module]
mod net {
    pub type TestEnv = super::Moc;

    #[test_with::runtime_http(127.0.0.1:8000)]
    fn test_with_environment() {
        assert!(true);
    }

    #[test_with::runtime_http(127.0.0.1:9000)]
    fn test_will_ignore() {
        assert!(false);
    }
}
