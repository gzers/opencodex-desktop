from pathlib import Path
import hashlib,json
CODE=Path(r"C:/Users/15119/.codex/worktrees/024-windows-native/code")
ROOT=Path(__file__).resolve().parent
base=CODE/"apps/desktop/tauri/src/infrastructure/windows_client_originals024"
p=base/"v2/adapter.rs";raw=p.read_bytes();text=raw.decode()
start=text.index("struct CurrentPin {");end=text.index("#[derive(PartialEq, Eq)]\nenum State",start) if "#[derive(PartialEq, Eq)]\nenum State" in text else text.index("#[derive(PartialEq, Eq)]\r\nenum State",start)
block=text[start:end];assert "struct Source {" in block
header='''//! Shared ORIGINAL Windows source pin for new-scope adapters. Strict existing
//! fixed-image/current/peer/token checks are reused; no parsed-data authority.
use super::{binding::Binding,bounded::{native_stamp,BoundedAccessV1},phase::{digest,invalid}};
use crate::{errors::{AppError,AppResult},infrastructure::{hash,windows_official_lifecycle024::Owner,windows_official_runtime024::{PinnedDirectory,RetainedPeer}},modules::runtime::{program_image,program_selection::{self,CurrentSelection},windows_lifecycle024_contract::OFFICIAL_PACKAGE_SHA256}};
use std::{fs::{File,OpenOptions},io::{self,Read,Seek,SeekFrom},os::windows::fs::OpenOptionsExt,path::Path,time::Instant};
fn refused(reason:impl std::fmt::Display)->AppError{AppError::RuntimeManaged{code:"windows024_originals_required".into(),detail:format!("原件持久恢复未确认；保留原代次/记录/句柄：{reason}")}}
fn before(end:Instant)->io::Result<()>{if Instant::now()>=end{Err(io::ErrorKind::TimedOut.into())}else{Ok(())}}
'''
out=block.replace("struct CurrentPin {","pub(super) struct CurrentPin {").replace("struct Source {","pub(super) struct Source {")
out=out.replace("    binding_raw: Vec<u8>,","    pub(super) binding_raw: Vec<u8>,")
out=out.replace("    fn capture(","    pub(super) fn capture(").replace("    fn check(","    pub(super) fn check(")
target=base/"source.rs";assert not target.exists();target.write_text(header+out,encoding="utf-8",newline="")
p.write_text(text[:start]+text[end:],encoding="utf-8",newline="")
with (ROOT/"source-extraction.json").open("x",encoding="utf-8") as o:json.dump(dict(source=str(p),target=str(target),originalSha256=hashlib.sha256(raw).hexdigest(),originalBlockSha256=hashlib.sha256(block.encode()).hexdigest(),transforms=["same CurrentPin and Source bodies, only module imports and limited visibility changed","existing adapter tests retained and consume moved CurrentPin"],semanticsIntendedUnchanged=True),o,indent=2)
