extern crate proof;
extern crate time;
extern crate rand;

use rand::{Rng};
use proof::{proofer};
use proof::parallel_race_pool::{Pool, ParallelRacePoolTrait};
use std::sync::mpsc::{Sender, Receiver, channel};

pub struct TaskPool {
    workers: Vec<Sender<ProofTask>>,
    wait_rx: Receiver<Option<usize>>
}

pub struct ProofTask {
    input: String,
    lower_bound: usize,
    upper_bound: usize
}

impl ParallelRacePoolTrait<ProofTask, Option<usize>, TaskPool> for TaskPool {
    fn init_pool (workers: Vec<Sender<ProofTask>>, wait_rx: Receiver<Option<usize>>) -> TaskPool {
        TaskPool {
            workers: workers,
            wait_rx: wait_rx
        }
    }

    fn get_workers (&self) -> &Vec<Sender<ProofTask>> {
        &self.workers
    }

    fn get_wait_rx (&self) -> &Receiver<Option<usize>> {
        &self.wait_rx
    }

    fn task_func (task: ProofTask) -> Option<usize> {
        proofer::get_proof_para(&task.input.into_bytes(), 2, task.lower_bound, task.upper_bound)
    }
}

fn test_prp_as_trait (input: String) -> Option <usize> {
    let concurrency = 4;
    let pool = TaskPool::new(concurrency);
    let max_size = usize::max_value();
    let frac = max_size / concurrency;
    let task_list = (0..concurrency).map(|x| {
        let lower = x * frac;
        let upper = (x + 1) * frac;
        ProofTask{
            input: input.clone(),
            upper_bound: upper,
            lower_bound: lower
        }
    }).collect::<Vec<ProofTask>>();
    pool.send_tasks_and_wait(task_list)
}

fn test_prp_callback (input: String) -> Option <usize> {
    let concurrency = 4;
    let pool = Pool::new(concurrency, |task: ProofTask| {
        proofer::get_proof_para(&task.input.into_bytes(), 2, task.lower_bound, task.upper_bound)
    });
    let max_size = usize::max_value();
    let frac = max_size / concurrency;
    let task_list = (0..concurrency).map(|x| {
        let lower = x * frac;
        let upper = (x + 1) * frac;
        ProofTask{
            input: input.clone(),
            upper_bound: upper,
            lower_bound: lower
        }
    }).collect::<Vec<ProofTask>>();
    pool.send_tasks_and_wait(task_list)
}



fn main () {
    let input = rand::thread_rng()
        .gen_ascii_chars()
        .take(10)
        .collect::<String>();

    let before = time::precise_time_ns();

    let input2 = "eeee".to_string();
    let x = test_prp_callback(input2);

    let elapsed = (time::precise_time_ns() - before) as f64;
    let as_ms = elapsed/1000000.0;
    println!("{:?}, took {}ms", x, as_ms);
}
