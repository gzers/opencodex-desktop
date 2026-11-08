//! Share only an unfinished observation. Terminal results are never cached.
use super::*;
use std::{collections::BTreeMap, future::Future, sync::Mutex};
use tokio::sync::watch;
type Observation = Option<AppResult<EnvironmentReportDto>>;
#[derive(Default)]
pub(super) struct Flights {
    active: Mutex<BTreeMap<usize, watch::Receiver<Observation>>>,
}
impl Flights {
    pub(super) async fn run<F>(
        self: Arc<Self>,
        key: usize,
        task: F,
    ) -> AppResult<EnvironmentReportDto>
    where
        F: Future<Output = AppResult<EnvironmentReportDto>> + Send + 'static,
    {
        let mut receiver = {
            let mut active = self.active.lock().map_err(|_| AppError::NotConfigured)?;
            if let Some(receiver) = active.get(&key) {
                receiver.clone()
            } else {
                let (sender, receiver) = watch::channel(None);
                active.insert(key, receiver.clone());
                let flights = self.clone();
                // Worker survives a cancelled WebView observer. The runtime Arc
                // captured by task keeps its pointer identity live until removal.
                tokio::spawn(async move {
                    let result = tokio::spawn(task)
                        .await
                        .unwrap_or_else(|_| Err(error("discovery_worker", "集成来源检查中断")));
                    if let Ok(mut active) = flights.active.lock() {
                        active.remove(&key);
                    }
                    let _ = sender.send(Some(result));
                });
                receiver
            }
        };
        loop {
            let result = receiver.borrow_and_update().clone();
            if let Some(result) = result {
                return result;
            }
            receiver
                .changed()
                .await
                .map_err(|_| error("discovery_worker", "集成来源检查中断"))?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::sync::Notify;
    #[tokio::test]
    async fn overlapping_observers_share_work_but_terminal_results_never_satisfy_a_new_request() {
        let flights = Arc::new(Flights::default());
        let entered = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());
        let calls = Arc::new(AtomicUsize::new(0));
        let start = {
            let f = flights.clone();
            let e = entered.clone();
            let r = release.clone();
            let c = calls.clone();
            tokio::spawn(async move {
                f.run(1, async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    e.notify_one();
                    r.notified().await;
                    Err(AppError::NotConfigured)
                })
                .await
            })
        };
        entered.notified().await;
        let c = calls.clone();
        let second = flights.clone().run(1, async move {
            c.fetch_add(1, Ordering::SeqCst);
            Err(AppError::Timeout)
        });
        tokio::pin!(second);
        // Biased select polls the second observer until it is waiting on the
        // unfinished watch result, before the release branch is selected.
        tokio::select! {biased;value=&mut second=>panic!("premature observation: {value:?}"),_ = tokio::task::yield_now()=>{}}
        release.notify_one();
        assert_eq!(start.await.unwrap(), Err(AppError::NotConfigured));
        assert_eq!(second.await, Err(AppError::NotConfigured));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(flights.active.lock().unwrap().is_empty());
        let result = flights.run(1, async { Err(AppError::Timeout) }).await;
        assert_eq!(result, Err(AppError::Timeout));
    }
    #[tokio::test]
    async fn cancelling_observer_preserves_worker_and_other_runtime_handles_are_independent() {
        let flights = Arc::new(Flights::default());
        let entered = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());
        let observer = {
            let f = flights.clone();
            let e = entered.clone();
            let r = release.clone();
            tokio::spawn(async move {
                f.run(1, async move {
                    e.notify_one();
                    r.notified().await;
                    Err(AppError::NotConfigured)
                })
                .await
            })
        };
        entered.notified().await;
        observer.abort();
        let _ = observer.await;
        assert_eq!(
            flights
                .clone()
                .run(2, async { Err(AppError::Timeout) })
                .await,
            Err(AppError::Timeout)
        );
        assert!(flights.active.lock().unwrap().contains_key(&1));
        release.notify_one();
        loop {
            if flights.active.lock().unwrap().is_empty() {
                break;
            }
            tokio::task::yield_now().await;
        }
    }
}
