use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct Pool <T, R> {
    workers: Vec<Sender<T>>,
    wait_rx: Receiver<Option<R>>,
    signal_flag: Arc<AtomicBool>
}

impl <T, R> Pool <T, R> where T:Send + 'static, R: Send + 'static {
    /// spawns multiple threads to parallel task f
    /// work is not started until it is started with Self::send_tasks_and_wait
    pub fn new <F> (num_threads: usize, f: F) -> Pool <T, R>
    where F: Fn (T, &Arc<AtomicBool>) -> Option<R> + Sync + Send + 'static {
        let (done, wait) = channel();
        let func = Arc::new(f);
        let signal_flag = Arc::new(AtomicBool::new(true));
        let workers = (0..num_threads).map(|_| {
            let (snd, work) = channel();
            let _done = done.clone();
            let f = func.clone();
            let thread_flag = signal_flag.clone();
            let _ = thread::spawn(move || {
                loop {
                    let _ = match work.recv() {
                        Ok(task) => {
                            match f(task, &thread_flag) {
                                None => (),
                                some => {let _ = _done.send(some);}
                            };
                        }
                        _ => ()
                    };
                }
            });
            snd
        }).collect();
        Pool {
            workers: workers,
            wait_rx: wait,
            signal_flag: signal_flag
        }
    }

    //TODO: this blocks indefinitely if an empty vec is sent
    //also need to make this typesafe -> perhaps wrap R in an option
    pub fn send_tasks_and_wait(&self, task_list: Vec<T>) -> Option<R> {
        let izip = task_list.into_iter().zip(self.workers.iter());
        for i in izip {
            let (_, ref send) = i;
            let (task, _) = i;
            let _  = send.send(task);
        }
        let ret = self.wait_rx.recv().unwrap();
        // this is done outside, on the rx side for now to not have a race to return within the task
        self.signal_flag.store(false, Ordering::Relaxed);
        ret
    }
}

/// alternative to using Pool::new. Trait implementation is required
pub trait ParallelRacePool <T, R> where T:Send + 'static, R: Send + 'static {
    // the task to launch (divided)
    fn task_func (task: T, thread_flag: &Arc<AtomicBool>) -> Option<R>;

    fn init (num_threads: usize) -> Pool<T, R> {
        let (done, wait) = channel();
        let signal_flag = Arc::new(AtomicBool::new(true));
        let workers = (0..num_threads).map(|_| {
            let (snd, work) = channel();
            let _done = done.clone();
            let thread_flag = signal_flag.clone();
            let _ = thread::spawn(move || {
                loop {
                    let _ = match work.recv() {
                        Ok(task) => {
                            match Self::task_func(task, &thread_flag) {
                                None => (),
                                some => {let _ = _done.send(some);}
                            };
                        }, 
                            //let _ = _done.send(Self::task_func(task, &thread_flag));},
                        _ => ()
                    };
                }
            });
            snd
        }).collect();
        Pool {
            workers: workers,
            wait_rx: wait,
            signal_flag: signal_flag
        }
    }
 }

