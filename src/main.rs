mod my_futures;
mod runtime;

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};


fn main() {
    let (executor, spawner) = runtime::new_executor_and_spawner();
    
    // this is used instead in order to terminate the `yield loop task` when the `timer task` finishes
    let can_be_terminated = Arc::<AtomicBool>::default();
    let can_be_terminated_clone = Arc::clone(&can_be_terminated);

    spawner.spawn(async move {
        loop {
            // This gives the control back to the runtime, allowing executing concurrently other tasks
            //   without yielding back the control => it would be impossible to run other tasks, because the Executor doesnt spawn separated threads
            //   for every new task 
            my_futures::yield_now::yield_now().await;
            if can_be_terminated_clone.load(Ordering::Relaxed) { break; }
        }
    });

    spawner.spawn(async move {
        println!("sleep for 1 second!");
        my_futures::time::sleep(std::time::Duration::from_secs(1)).await;
        println!("wake up!");
        can_be_terminated.store(true, Ordering::Relaxed);
    });

    std::mem::drop(spawner);  // droping the tx half, so the rx half in the executor terminates the while loop

    executor.run();
}