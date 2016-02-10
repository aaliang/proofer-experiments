extern crate crypto;

use std::mem;
use crypto::sha1::Sha1;
use crypto::digest::Digest;

//const MAX_USIZE: usize = !0;

trait NextVal {
    fn next_val(&mut self);
}

impl NextVal for usize {
    fn next_val(&mut self) {
        *self = *self + 1
    }
}

pub fn get_proof (inp: &[u8], difficulty: usize) -> usize {
    let mut buffer = [0; 2048];
    for (a, b) in inp.iter().zip(buffer.iter_mut()) {
        *b = *a;
    }
    let b_start = inp.len();
    let mut s:usize = 0;

    'outer: loop {
        let arr : [u8; 8]= unsafe { mem::transmute(s.to_be()) };
        let mut new_buffer: [u8; 2048] = unsafe { mem::transmute_copy(&buffer) };
        {
            let slice = &mut new_buffer[b_start..];
            for (a, b) in slice.iter_mut().zip(arr.iter()) {
                *a = *b;
            }
        }
        let mut sha = Sha1::new();
        let _ = sha.input(&new_buffer);
        let mut result: [u8; 20] = unsafe {mem::uninitialized()};
        let _ = sha.result(&mut result);

        for i in 0..difficulty {
            if result[i] != 0 {
                s.next_val();
                continue 'outer;
            }
        }

        return s

    }
}
