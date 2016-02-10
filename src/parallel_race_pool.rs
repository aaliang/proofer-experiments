use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread;
use std::sync::Arc;

pub struct Pool <T, R> {
    workers: Vec<Sender<T>>,
    wait_rx: Receiver<R>
}

impl <T, R> Pool <T, R> where T:Send + 'static, R: Send + 'static {
    /// spawns multiple threads to parallel task f
    /// work is not started until it is started with Self::send_tasks_and_wait
    pub fn init <F> (num_threads: usize, f: F) -> Pool <T, R>
    where F: Fn (T) -> R + Sync + Send + 'static {
        let (done, wait) = channel();
        let func = Arc::new(f);
        let workers = (0..num_threads).map(|_| {
            let (snd, work) = channel();
            let _done = done.clone();
            let f = func.clone();
            let _ = thread::spawn(move || {
                loop {
                    let _ = match work.recv() {
                        Ok(task) => {let _ = _done.send(f(task));},
                        _ => ()
                    };
                }
            });
            snd
        }).collect();
        Pool {
            workers: workers,
            wait_rx: wait
        }
    }

    //TODO: this blocks indefinitely if an empty vec is sent
    //also need to make this typesafe -> perhaps wrap R in an option
    pub fn send_tasks_and_wait(&self, task_list: Vec<T>) -> R {
        let izip = task_list.into_iter().zip(self.workers.iter());
        for i in izip {
            let (_, ref send) = i;
            let (task, _) = i;
            let _  = send.send(task);
        }
        self.wait_rx.recv().unwrap()
    }
}

/// alternative to using Pool::init. Trait implementation is required
pub trait ParallelRacePool <T, R> where T:Send + 'static, R: Send + 'static {
    // the task to launch (divided)
    fn task_func (task: T) -> R;

    fn new (num_threads: usize) -> Pool<T, R> {
        let (done, wait) = channel();
        let workers = (0..num_threads).map(|_| {
            let (snd, work) = channel();
            let _done = done.clone();
            let _ = thread::spawn(move || {
                loop {
                    let _ = match work.recv() {
                        Ok(task) => {let _ = _done.send(Self::task_func(task));},
                        _ => ()
                    };
                }
            });
            snd
        }).collect();
        Pool {
            workers: workers,
            wait_rx: wait
        }
    }
 }

