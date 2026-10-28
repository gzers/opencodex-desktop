use std::{path::Path,process::{Command,Stdio}};
fn main(){
 let base=Path::new(r"D:\Software\Develop\OpenCodeX Desktop\data");
 let cwd=base.join(r"runtime\opencodex.new-native-diagnostic\.downloads");
 std::fs::create_dir_all(&cwd).unwrap();
 let mut c=Command::new(r"C:\Users\15119\.nvmd\bin\npm.exe");
 c.env_clear().env("PATH",r"C:\Users\15119\.nvmd\bin;C:\WINDOWS\System32;C:\WINDOWS")
 .env("LANG","C.UTF-8").env("LC_ALL","C.UTF-8").env("npm_config_update_notifier","false").env("npm_config_fund","false").env("npm_config_audit","false").env("npm_config_progress","false")
 .env("HOME",r"C:\Users\15119").env("USERPROFILE",r"C:\Users\15119").env("APPDATA",r"C:\Users\15119\AppData\Roaming").env("LOCALAPPDATA",r"C:\Users\15119\AppData\Local")
 .env("COMSPEC",r"C:\WINDOWS\System32\cmd.exe").env("SystemRoot",r"C:\WINDOWS").env("WINDIR",r"C:\WINDOWS")
 .env("TEMP",std::env::temp_dir()).env("TMP",std::env::temp_dir()).env("npm_config_cache",base.join("cache/npm"))
 .current_dir(&cwd).args(["pack","@bitkyc08/opencodex@latest","--pack-destination"]).arg(&cwd).args(["--loglevel","info"])
 .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
 println!("command={c:?}");
 let o=c.output().unwrap();
 std::fs::write("test/out/native-install/rust-baseline-stdout.bin",o.stdout).unwrap();
 std::fs::write("test/out/native-install/rust-baseline-stderr.bin",o.stderr).unwrap();
 println!("exit={:?}",o.status);
}
