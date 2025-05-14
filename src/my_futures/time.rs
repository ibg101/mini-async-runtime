use std::{
    pin::Pin,
    sync::{Arc, RwLock},
    task::{Context, Poll, Waker}
};


#[derive(Default)]
pub struct Sleep {
    inner: Arc<RwLock<State>>
}

#[derive(Default)]
struct State {
    is_ready: bool,
    waker: Option<Waker>
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.inner.read().unwrap().is_ready {
            Poll::Ready(())
        } else {
            self.inner.write().unwrap().waker = Some(cx.waker().clone()); 
            Poll::Pending
        }
    }
}

pub fn sleep(dur: std::time::Duration) -> Sleep {
    let sleep = Sleep::default();
    let state = Arc::clone(&sleep.inner);

    std::thread::spawn(move || {
        let mut state_lock = state.write().unwrap();
        std::thread::sleep(dur);
        state_lock.is_ready = true;
        
        if let Some(waker) = state_lock.waker.take() {
            waker.wake();
        }
    });

    sleep
}