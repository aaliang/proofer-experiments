use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread;
/*
pub struct ParallelRacePool <T, R> where T: Send + 'static, R: Send + 'static {
    workers: Vec<(Sender<T>, thread::JoinHandle<()>)>,
    wait_rx: Receiver<R>
}
*/
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
