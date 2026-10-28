use std::{path::PathBuf,process::Command};
fn main(){
 let out=PathBuf::from("test/out/windows-path-audit");
 for (name,program,comspec) in [
  ("mixed-both",r"C:\Users\15119\.nvmd/bin\npm.exe",r"C:\WINDOWS\System32/cmd.exe"),
  ("mixed-entry",r"C:\Users\15119\.nvmd/bin\npm.exe",r"C:\WINDOWS\System32\cmd.exe"),
  ("mixed-comspec",r"C:\Users\15119\.nvmd\bin\npm.exe",r"C:\WINDOWS\System32/cmd.exe"),
  ("native-both",r"C:\Users\15119\.nvmd\bin\npm.exe",r"C:\WINDOWS\System32\cmd.exe")]{
  let mut command=Command::new(program);
  command.arg("--version").env_clear().current_dir(out.canonicalize().unwrap())
   .env("COMSPEC",comspec).env("SystemRoot",r"C:\WINDOWS").env("WINDIR",r"C:\WINDOWS")
   .env("HOME",r"C:\Users\15119").env("USERPROFILE",r"C:\Users\15119")
   .env("APPDATA",r"C:\Users\15119\AppData\Roaming").env("LOCALAPPDATA",r"C:\Users\15119\AppData\Local")
   .env("TEMP",std::env::temp_dir()).env("TMP",std::env::temp_dir())
   .env("PATH",r"C:\Users\15119\.nvmd\bin;C:\WINDOWS\System32;C:\WINDOWS");
  let result=command.output().unwrap();
  std::fs::write(out.join(format!("nvmd-{name}-stdout.bin")),&result.stdout).unwrap();
  std::fs::write(out.join(format!("nvmd-{name}-stderr.bin")),&result.stderr).unwrap();
  println!("case={name} program={program:?} COMSPEC={comspec:?} cwd={:?} exit={:?} stdout={:?} stderr_bytes={}",command.get_current_dir(),result.status.code(),String::from_utf8_lossy(&result.stdout).trim(),result.stderr.len());
 }
}
