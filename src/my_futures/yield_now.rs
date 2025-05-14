use std::{
    pin::Pin,
    task::{Context, Poll}
};


#[derive(Default)]
pub struct YieldNow {
    must_yield: bool
}

impl Future for YieldNow {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {        
        if self.must_yield {
            Poll::Ready(())
        } else {
            self.get_mut().must_yield = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

pub fn yield_now() -> YieldNow {
    YieldNow::default()
}