use crate::blockchain::{Data, JobSync};

use std::sync::atomic::Ordering;

use tracing_test::traced_test;

#[test]
#[traced_test]
fn test_create_new_permit() {
    let js = JobSync::<Data>::new(false);
    js.new_permit();
    let permits: usize = js.permits.load(Ordering::SeqCst);
    assert_eq!(permits, 1);
}

#[tokio::test]
#[traced_test]
async fn test_claim_permit() {
    let js = JobSync::<Data>::new(false);
    js.new_permit();
    let permits: usize = js.permits.load(Ordering::SeqCst);
    assert_eq!(permits, 1);
    js.claim_permit().await;
    let permits: usize = js.permits.load(Ordering::SeqCst);
    assert_eq!(permits, 0);
}
