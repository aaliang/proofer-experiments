extern crate proof;
extern crate time;
extern crate rand;

use rand::{Rng};
use proof::{proofer};
use proof::parallel_race_pool::{Pool, ParallelRacePool};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub struct ProofTask {
    input: String,
    lower_bound: usize,
    upper_bound: usize
}

//trait implementation example
struct Executor;
impl ParallelRacePool<ProofTask, usize> for Executor {
    fn task_func (task: ProofTask, should_continue: &Arc<AtomicBool>) -> Option<usize> {
        proofer::get_proof_para(&task.input.into_bytes(), 4, task.lower_bound, task.upper_bound, should_continue)
    }
}

fn test_prp_as_trait (input: String) -> Option <usize> {
    let concurrency = 4;
    let pool = Executor::init(concurrency);
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

//inline callback example
fn test_prp_as_callback (input: String) -> Option <usize> {
    let concurrency = 4;
    let pool = Pool::new(concurrency, |task: ProofTask, should_continue: &Arc<AtomicBool>| {
        proofer::get_proof_para(&task.input.into_bytes(), 4, task.lower_bound, task.upper_bound, should_continue)
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

    let x = test_prp_as_trait(input);

    let elapsed = (time::precise_time_ns() - before) as f64;
    let as_ms = elapsed/1000000.0;
    println!("{:?}, took {}ms", x, as_ms);
}
