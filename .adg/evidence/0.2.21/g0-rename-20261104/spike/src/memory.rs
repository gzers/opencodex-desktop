//! Process-local credential builder, adapted from the repository's existing
//! config_file_scoped_tests::memory_keyring. No native backend is constructed.
use keyring::credential::{Credential, CredentialApi, CredentialBuilderApi};
use std::{any::Any, collections::BTreeMap, sync::{Arc, Mutex}};

#[derive(Default)]
struct State {
    values: BTreeMap<(String, String), Vec<u8>>,
    operations: usize,
    writes: usize,
    fail_at: Option<usize>,
}
#[derive(Clone, Default)]
pub struct Memory(Arc<Mutex<State>>);
pub struct Builder(pub Memory);
struct Entry { memory: Memory, identity: (String, String) }
impl Memory {
    pub fn install(&self) { keyring::set_default_credential_builder(Box::new(Builder(self.clone()))); }
    pub fn reset(&self) { *self.0.lock().unwrap() = State::default(); }
    pub fn fail_after(&self, n: usize) {
        let mut s = self.0.lock().unwrap(); s.fail_at = Some(s.operations + n);
    }
    pub fn clear_fault(&self) { self.0.lock().unwrap().fail_at = None; }
    pub fn snapshot(&self) -> BTreeMap<(String, String), Vec<u8>> { self.0.lock().unwrap().values.clone() }
    pub fn writes(&self) -> usize { self.0.lock().unwrap().writes }
}
fn operate(s: &mut State) -> keyring::Result<()> {
    s.operations += 1;
    if s.fail_at == Some(s.operations) {
        s.fail_at = None;
        return Err(keyring::Error::PlatformFailure(Box::new(std::io::Error::other("injected credential failure"))));
    }
    Ok(())
}
impl CredentialBuilderApi for Builder {
    fn build(&self, _: Option<&str>, service: &str, user: &str) -> keyring::Result<Box<Credential>> {
        Ok(Box::new(Entry { memory: self.0.clone(), identity: (service.into(), user.into()) }))
    }
    fn as_any(&self) -> &dyn Any { self }
}
impl CredentialApi for Entry {
    fn set_secret(&self, value: &[u8]) -> keyring::Result<()> {
        let mut s = self.memory.0.lock().unwrap(); operate(&mut s)?;
        s.values.insert(self.identity.clone(), value.to_vec()); s.writes += 1; Ok(())
    }
    fn get_secret(&self) -> keyring::Result<Vec<u8>> {
        let mut s = self.memory.0.lock().unwrap(); operate(&mut s)?;
        s.values.get(&self.identity).cloned().ok_or(keyring::Error::NoEntry)
    }
    fn delete_credential(&self) -> keyring::Result<()> {
        let mut s = self.memory.0.lock().unwrap(); operate(&mut s)?;
        s.values.remove(&self.identity).map(|_| {s.writes += 1;}).ok_or(keyring::Error::NoEntry)
    }
    fn as_any(&self) -> &dyn Any { self }
}
