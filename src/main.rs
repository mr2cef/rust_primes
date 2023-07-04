use std::thread;
use std::sync::{Arc, RwLock};
use crossbeam::channel::{unbounded, Sender, Receiver};

const N_THREADS: usize = 16;

#[derive(Clone, Copy, Debug)]
struct CompTrimple {
    counter: i128,
    thesqrt: i128,
    subcounter: i64,
    subthreshhold: i64,
    is_prime: Option<bool>,
    is_done: bool,
}

impl CompTrimple {
    fn iterate(&mut self) -> () {
        self.counter += 2;
        self.subcounter += 2;
        if self.subcounter > self.subthreshhold {
            self.subcounter = 0;
            self.subthreshhold += 2;
            self.thesqrt += 1;
        }
    }
}

fn is_prime(arc: Arc<RwLock<Vec<i128>>>, cr_candidate: Receiver<CompTrimple>, cs_result: Sender<CompTrimple>, _id: usize) -> Result<(),String> {
    loop {
        let mut c = cr_candidate.recv().unwrap();
        {        
            let vec_ro = arc.read().unwrap();
            if c.is_done {
                break;
            }
            //println!("thread {_id} reads {:?}", vec_ro);
            for &d in vec_ro.iter() {
                //println!("reading counter:{} prime:{} threadid:{}", c.counter, d, _id);
                if c.counter % d == 0 {
                    c.is_prime = Some(false);
                    break;
                }
                if d > c.thesqrt {
                    c.is_prime = Some(true);
                    break;
                }
            }
        }
        cs_result.send(c).unwrap();
        //println!("thread {_id} sends {:?}", c);
    }
    Ok(())
}
 
fn main() {
    let mut vec: Vec<i128> = Vec::new();
    let mut n_primes: i128 = 4;
    vec.push(2_i128);
    vec.push(3_i128);
    vec.push(5_i128);
    vec.push(7_i128);
    let mut c: CompTrimple = CompTrimple {
        counter: 9,
        thesqrt: 3,
        subcounter: 3,
        subthreshhold: 5,
        is_prime: None,
        is_done: false,
    };
    let arc = Arc::new(RwLock::new(vec));
    let (cs_candidate, cr_candidate): (Sender<CompTrimple>, Receiver<CompTrimple>) = unbounded();
    let (cs_result, cr_result): (Sender<CompTrimple>, Receiver<CompTrimple>) = unbounded();
    
    let mut children: [Option<thread::JoinHandle<Result<(),String>>>; N_THREADS] = Default::default();
    for id in 0..N_THREADS {
        let v = arc.clone();
        let crc = cr_candidate.clone();
        let csr = cs_result.clone();
        children[id] = Some(thread::spawn(move|| is_prime(v, crc, csr, id)));
    }
    while n_primes < 50000 {
        for _ in 0..N_THREADS {
            cs_candidate.send(c).unwrap();
            c.iterate();
        }
        for _ in 0..N_THREADS {
            //println!("waiting for result...");
            let d = cr_result.recv().unwrap();
            //println!("result received {:?}", d);
            match d.is_prime {
                Some(true) => {
                    n_primes += 1;
                    let mut v = arc.write().unwrap();
                    v.push(d.counter);  
                    v.sort()
                },
                _ => {}
            }
        }
    }
    c.is_done = true;
    for _ in 0..N_THREADS {
        cs_candidate.send(c).unwrap();
    }
    for child in children {
        match child {
            Some(child) => {
                let _ = child.join().unwrap();
            },
            None => println!("none"),
        }
    }
    //println!("{} numberes checked", N_THREADS);
    println!("{:?}", arc);
}
