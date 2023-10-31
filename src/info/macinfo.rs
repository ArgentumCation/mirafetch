#[cfg(target_os = "macos")]
use arcstr::ArcStr;
pub struct MacInfo {}

impl OSInfo for MacInfo {
    fn os(&self) -> Option<ArcStr> {
        let info =
            plist::from_file("/System/Library/CoreServices/SystemVersion.plist").as_dictionary();
        let name = info.get("ProductName").unwrap().as_string().unwrap();
        let version = info
            .get("ProductuservisibleVersion")
            .unwrap()
            .as_string()
            .unwrap();
        let build = info
            .get("ProductBuildVersion")
            .unwrap()
            .as_string()
            .unwrap();
    }
}
