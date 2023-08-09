use crate::info::OSInfo;

pub struct iOSInfo {}
impl iOSInfo {
    pub fn new() -> Self {
        Self {}
    }
}
impl OSInfo for iOSInfo {
    fn sys_font(&self) -> Option<String> {
        None
    }

    fn cursor(&self) -> Option<String> {
        None
    }

    fn terminal(&self) -> Option<String> {
        None
    }

    fn term_font(&self) -> Option<String> {
        None
    }

    fn gpus(&self) -> Vec<std::sync::Arc<str>> {
        Vec::new()
    }

    fn memory(&self) -> Option<String> {
        None
    }

    fn disks(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    fn battery(&self) -> Option<String> {
        None
    }

    fn locale(&self) -> Option<String> {
        None
    }

    fn theme(&self) -> Option<String> {
        None
    }

    fn icons(&self) -> Option<String> {
        None
    }

    fn os(&self) -> Option<String> {
        None
    }

    fn displays(&self) -> Vec<std::sync::Arc<str>> {
        Vec::new()
    }

    fn machine(&self) -> Option<String> {
        None
    }

    fn kernel(&self) -> Option<String> {
        None
    }

    fn wm(&self) -> Option<String> {
        None
    }

    fn de(&self) -> Option<String> {
        None
    }

    fn shell(&self) -> Option<String> {
        None
    }

    fn cpu(&self) -> Option<String> {
        None
    }

    fn username(&self) -> Option<std::sync::Arc<str>> {
        None
    }

    fn id(&self) -> std::sync::Arc<str> {
        todo!()
    }

    fn uptime(&self) -> Option<String> {
        todo!()
    }

    fn ip(&self) -> Vec<std::sync::Arc<str>> {
        todo!()
    }

    fn hostname(&self) -> Option<std::sync::Arc<str>> {
        todo!()
    }
}
