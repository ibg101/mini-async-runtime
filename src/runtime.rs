use std::{
    pin::Pin,
    sync::{
        Arc,
        Mutex,
        mpsc::{self, Sender, Receiver}
    }
};

use futures::task::{waker_ref, ArcWake};


pub struct Executor {
    ready_queue: Receiver<Arc<Task>>  
}

pub struct Spawner {
    task_sender: Sender<Arc<Task>>
}

struct Task {
    future: Mutex<Option<Pin<Box<dyn Future<Output = ()> + 'static + Send>>>>,
    task_sender: Sender<Arc<Task>>
}

unsafe impl Send for Task {}
unsafe impl Sync for Task {}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let task = Arc::clone(&arc_self);
        arc_self.task_sender
            .send(task)
            .expect("Failed to send task onto a queue after being awaken!")
    }
}

impl Spawner {
    pub fn spawn<Fut: Future<Output = ()> + 'static + Send>(&self, future: Fut) -> () {
        let boxed_future = Box::pin(future);
        let task = Arc::new(Task {
            future: Mutex::new(Some(boxed_future)),
            task_sender: self.task_sender.clone()
        });
        self.task_sender.send(task).expect("Failed to send spawned task onto a queue");
    }
}

impl Executor {
    pub fn run(&self) -> () {
        while let Ok(task) = self.ready_queue.recv() {
            let mut future_lock = task.future.lock().unwrap();
            if let Some(mut future) = future_lock.take() {
                std::mem::drop(future_lock);  // releasing the lock to avoid the deadlock, if lock is acquired during the wake()
                
                let waker = waker_ref(&task);
                let mut context = std::task::Context::from_waker(&waker);

                if future.as_mut().poll(&mut context).is_pending() {
                    *task.future.lock().unwrap() = Some(future);
                }
            }
        }
    }
}

pub fn new_executor_and_spawner() -> (Executor, Spawner) {
    let (tx, rx) = mpsc::channel::<Arc<Task>>();
    let executor = Executor { ready_queue: rx };
    let spawner = Spawner { task_sender: tx };
    (executor, spawner)
}