// Test-only Windows filesystem observations. Each probe continues independently;
// this executable never provisions runtime or changes user/installation policy.
use std::{env, fs, io::Write, path::{Path,PathBuf}};
fn quoted(s:&str)->String{let mut out=String::from("\"");for c in s.chars(){match c{'"'=>out.push_str("\\\""),'\\'=>out.push_str("\\\\"),'\n'=>out.push_str("\\n"),'\r'=>out.push_str("\\r"),'\t'=>out.push_str("\\t"),c if c.is_control()=>out.push_str(&format!("\\u{:04x}",c as u32)),c=>out.push(c)}}out.push('"');out}
fn report(name:&str,path:&Path,result:Result<String,std::io::Error>){match result{Ok(value)=>println!("{{\"probe\":{},\"path\":{},\"status\":\"pass\",\"value\":{},\"accepted\":false}}",quoted(name),quoted(&path.to_string_lossy()),quoted(&value)),Err(error)=>println!("{{\"probe\":{},\"path\":{},\"status\":\"fail\",\"firstError\":{},\"osError\":{},\"accepted\":false}}",quoted(name),quoted(&path.to_string_lossy()),quoted(&error.to_string()),error.raw_os_error().map(|n|n.to_string()).unwrap_or("null".into()))}}
fn inspect(path:&Path){let mut current=PathBuf::new();for component in path.components(){current.push(component);report("same-component-symlink-metadata",&current,fs::symlink_metadata(&current).map(|m|format!("directory={} file={} symlink={}",m.is_dir(),m.is_file(),m.file_type().is_symlink())))}report("canonicalize",path,fs::canonicalize(path).map(|p|p.to_string_lossy().to_string()));}
fn main(){let args=env::args_os().skip(1).map(PathBuf::from).collect::<Vec<_>>();if args.len()!=2{std::process::exit(2)}
 for p in &args{inspect(p)}
 let install=&args[0];let descriptor=install.join("runtime-bundle/runtime-bundle.manifest.json");report("read-descriptor",&descriptor,fs::read(&descriptor).map(|b|format!("bytes={}",b.len())));
 // CreateNew only in the explicit new test-owned private program copy. Keep
 // independent lock/flush results; remove just this exact owned file after drop.
 let file=install.join(".sweep-filesystem-owned-probe");match fs::OpenOptions::new().read(true).write(true).create_new(true).open(&file){Ok(mut f)=>{report("create-own-file",&file,Ok("created-new".into()));report("write-own-file",&file,f.write_all(b"owned diagnostic probe").map(|_|"written".into()));report("sync-own-file",&file,f.sync_all().map(|_|"synced".into()));report("try-lock-own-file",&file,f.try_lock().map(|_|"locked".into()).map_err(std::io::Error::other));drop(f);report("remove-own-file",&file,fs::remove_file(&file).map(|_|"removed".into()));},Err(e)=>report("create-own-file",&file,Err(e))}
}
