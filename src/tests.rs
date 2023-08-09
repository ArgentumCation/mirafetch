#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::{process::Command};

    use lazy_static::lazy_static;
    

    use crate::Info;
    use crate::OSInfo;
    lazy_static! {
        static ref FF_INFO: HashMap<String, String> = {
            let mut ff_tmp = HashMap::new();
            let output: String = String::from_utf8_lossy(
                Command::new("fastfetch")
                    .args(["--pipe"])
                    .output()
                    .unwrap()
                    .stdout
                    .as_slice(),
            )
            .to_string();
            let mut lines = output.lines();
            let (x, y) = lines.next().unwrap().split_once('@').unwrap();
            ff_tmp.insert("username".to_string(), x.into());
            ff_tmp.insert("hostname".to_string(), y.into());
            lines.next();
            for line in lines {
                if let Some((x, y)) = line.split_once(": ") {
                    ff_tmp.insert(x.into(), y.into());
                }
            }
            ff_tmp
        };
        static ref MIRA_INFO: Info = Info::new();
    }
    #[test]
    fn test_username() {
        assert_eq!(
            &AsRef::<str>::as_ref(FF_INFO.get("username").unwrap()).trim_matches('\0'),
            &(MIRA_INFO.username.as_ref().unwrap().as_ref())
        );
    }
    #[test]
    fn test_hostname() {
        assert_eq!(
            &AsRef::<str>::as_ref(FF_INFO.get("hostname").unwrap()).trim(),
            &(MIRA_INFO.hostname.as_ref().unwrap().as_ref())
        );
    }
    #[test]
    #[ignore = "Mirafetch returns this differently from fastfetch"]
    fn test_os() {
        assert_eq!(
            &AsRef::<str>::as_ref(FF_INFO.get("OS").unwrap()).trim(),
            &(MIRA_INFO.os.as_ref().unwrap())
        );
    }
    #[test]
    fn test_kernel() {
        assert_eq!(
            &AsRef::<str>::as_ref(FF_INFO.get("Kernel").unwrap()).trim(),
            &(MIRA_INFO.kernel.as_ref().unwrap())
        );
    }
    #[test]
    fn test_cpu() {
        assert_eq!(
            &AsRef::<str>::as_ref(FF_INFO.get("CPU").unwrap()).trim(),
            &(MIRA_INFO.cpu.as_ref().unwrap())
        );
    }
}
