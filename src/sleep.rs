use async_timer::oneshot::*;
use std::time::Duration;

pub async fn sleep_ms(n: u64) {
    if n > 0 {
        Timer::new(Duration::from_millis(n)).await;
    }
}