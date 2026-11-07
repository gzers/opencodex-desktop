use std::{fs::OpenOptions, os::windows::{fs::OpenOptionsExt,io::AsRawHandle}, path::Path};
#[repr(C)] #[derive(Default)] struct Info { attrs:u32,created:[u32;2],access:[u32;2],write:[u32;2],volume:u32,size:[u32;2],links:u32,index:[u32;2] }
#[link(name="kernel32")] extern "system" {fn GetFileInformationByHandle(h:*mut core::ffi::c_void,p:*mut Info)->i32;}
fn main(){
 let profile=std::env::var("USERPROFILE").unwrap();
 for s in [format!("D:{}",char::from(92)),profile,format!("C:{}",char::from(92))]{
  for mask in [0x80020000u32,0x80000000,0x80,0x20080]{
   let opened=OpenOptions::new().access_mode(mask).share_mode(3).custom_flags(0x02200000).open(&s);
   match opened { Ok(f)=>{let mut i=Info::default();let ok=unsafe{GetFileInformationByHandle(f.as_raw_handle(),&mut i)};let error=if ok==0{std::io::Error::last_os_error().raw_os_error()}else{Some(0)};println!("path={s:?} access={mask:#010x} open_error=0 info_error={error:?} attrs={} links={}",i.attrs,i.links);},Err(e)=>println!("path={s:?} access={mask:#010x} open_error={:?}",e.raw_os_error()) }
  }
  for ancestor in Path::new(&s).ancestors(){println!("ancestor={ancestor:?}");}
 }
}
