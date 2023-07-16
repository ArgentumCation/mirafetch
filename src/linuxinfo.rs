#![cfg(target_family = "unix")]
use crate::util::OSInfo;
pub struct UnixInfo {}
impl OSInfo for UnixInfo {
    fn new() -> Self {
        Self {}
    }
}
