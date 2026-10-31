use actix_web::web;
use atlas_rs::BoxError;
use atlas_rs::db_refresher::{FAILURE_RETRY_INTERVAL, UpdatableDB, start_db_update_daemon};
use std::collections::VecDeque;
use std::sync::Mutex;
use tokio::sync::mpsc;
use tokio::time::{Duration, Instant};

struct MockDb {
    outcomes: Mutex<VecDeque<bool>>,
    calls: mpsc::UnboundedSender<(Instant, u64)>,
}

impl UpdatableDB for MockDb {
    async fn update_db(&self, db_min_age_secs: u64) -> Result<(), BoxError> {
        self.calls.send((Instant::now(), db_min_age_secs)).unwrap();
        match self.outcomes.lock().unwrap().pop_front() {
            Some(false) => Err("update failed".into()),
            _ => Ok(()),
        }
    }
}

#[tokio::test(start_paused = true)]
async fn test_daemon_schedules_updates() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let db = web::Data::new(MockDb {
        outcomes: Mutex::new(VecDeque::from([false, true, true])),
        calls: tx,
    });

    let daemon = tokio::spawn(start_db_update_daemon(db, 100));

    let (first, min_age) = rx.recv().await.unwrap();
    let (second, _) = rx.recv().await.unwrap();
    let (third, _) = rx.recv().await.unwrap();
    let (fourth, _) = rx.recv().await.unwrap();
    daemon.abort();

    assert_eq!(min_age, 100);
    // Failure retries sooner, success waits for the full interval
    assert_eq!(second - first, FAILURE_RETRY_INTERVAL);
    assert_eq!(third - second, Duration::from_secs(100));
    assert_eq!(fourth - third, Duration::from_secs(100));
}
