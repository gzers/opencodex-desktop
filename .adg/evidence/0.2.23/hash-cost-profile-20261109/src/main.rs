use sha2::{Digest, Sha256};
use std::{fs::File,io::Read,time::Instant};
fn main() {
 let path=std::env::args().nth(1).expect("fixture path");
 for sample in 0..3 {
  let t=Instant::now();let file=File::open(&path).unwrap();
  let expected=file.metadata().unwrap().len();let mut reader=file.take(512*1024*1024+1);
  let mut hash=Sha256::new();let mut buffer=[0u8;64*1024];let mut bytes=0u64;
  loop{let count=reader.read(&mut buffer).unwrap();if count==0{break;}bytes+=count as u64;assert!(bytes<=expected);hash.update(&buffer[..count]);}
  assert_eq!(bytes,expected);assert_eq!(reader.get_ref().metadata().unwrap().len(),bytes);
  let digest=format!("{:x}",hash.finalize());
  println!("sample={sample} bytes={bytes} milliseconds={} sha256={digest}",t.elapsed().as_millis());
 }
}
