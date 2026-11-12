use sha2::{Digest,Sha256};
use hmac::{Hmac,Mac};
fn hex(b:&[u8])->String {b.iter().map(|n|format!("{n:02x}")).collect()}
fn main(){
 assert_eq!(hex(&Sha256::digest(b"")),"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
 assert_eq!(hex(&Sha256::digest(b"abc")),"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
 let mut mac=<Hmac<Sha256> as Mac>::new_from_slice(&[0x0b;20]).unwrap();mac.update(b"Hi There");
 assert_eq!(hex(&mac.finalize().into_bytes()),"b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7");
 let chunk=vec![b'a';1024*1024];let started=std::time::Instant::now();
 let mut digest=Sha256::new();for _ in 0..256 {digest.update(std::hint::black_box(&chunk));}
 let hash=hex(&digest.finalize());assert_eq!(hash,"b4a0226ee3f9b159ac06a86332dca0d90a04adef7f88934aa2a75be2a011d504");
 println!("{{\"accelerated\":{},\"bytes\":268435456,\"elapsedMs\":{},\"sha256\":\"{}\",\"knownVectorsAndHmac\":true}}",cfg!(feature="accelerated"),started.elapsed().as_millis(),hash);
}
