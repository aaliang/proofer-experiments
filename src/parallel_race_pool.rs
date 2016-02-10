use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread;
use std::sync::Arc;

pub trait ParallelRacePoolTrait <T, R, I> where T:Send + 'static, R: Send + 'static {

    fn get_workers (&self) -> &Vec<Sender<T>>;
    fn get_wait_rx (&self) -> &Receiver<R>;
    fn init_pool (workers: Vec<Sender<T>>, wait_rx: Receiver<R>) -> I;
    fn task_func (task: T) -> R;

    fn new (num_threads: usize) -> I {
        let (done, wait) = channel();
        let workers = (0..num_threads).map(|_| {
            let (snd, work) = channel();
            let _done = done.clone();
            let _ = thread::spawn(move || {
                loop {
                    let task = match work.recv() {
                        Ok(task) => {let _ = _done.send(Self::task_func(task));},
                        _ => ()
                    };
                }
            });
            snd
        }).collect();
        Self::init_pool(workers, wait)
    }

    //TODO: this blocks indefinitely if an empty vec is sent
    //also need to make this typesafe -> perhaps wrap R in an option
    fn send_tasks_and_wait(&self, task_list: Vec<T>) -> R {
        let workers = self.get_workers();
        let izip = task_list.into_iter().zip(workers.iter());
        for i in izip {
            let (_, ref send) = i;
            let (task, _) = i;
            let _  = send.send(task);
        }
        self.get_wait_rx().recv().unwrap()
    }
}

pub struct Pool <T, R> {
    workers: Vec<Sender<T>>,
    wait_rx: Receiver<R>
}

impl <T, R> Pool <T, R> where T:Send + 'static, R: Send + 'static {

    pub fn new <F> (num_threads: usize, f: F) -> Pool <T, R>
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
