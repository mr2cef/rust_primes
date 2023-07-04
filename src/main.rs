use crossbeam::channel::{unbounded, Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread;

const N_THREADS: usize = 32;

#[derive(Clone, Copy, Debug)]
struct CompTrimple {
    counter: u128,
    thesqrt: u128,
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

fn is_prime(
    arc: Arc<RwLock<Vec<u128>>>,
    cr_candidate: Receiver<CompTrimple>,
    cs_result: Sender<CompTrimple>,
    _id: usize,
) -> Result<(), String> {
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

fn init() -> (
    Arc<RwLock<Vec<u128>>>,
    Sender<CompTrimple>,
    Receiver<CompTrimple>,
    CompTrimple,
    u128,
    Vec<thread::JoinHandle<Result<(), String>>>,
) {
    let mut vec: Vec<u128> = Vec::new();
    let n_primes: u128 = 4;
    vec.push(2_u128);
    vec.push(3_u128);
    vec.push(5_u128);
    vec.push(7_u128);
    let c: CompTrimple = CompTrimple {
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
    let mut children: Vec<thread::JoinHandle<Result<(), String>>> = Vec::new();
    for id in 0..N_THREADS {
        let v = arc.clone();
        let crc = cr_candidate.clone();
        let csr = cs_result.clone();
        children.push(thread::spawn(move || is_prime(v, crc, csr, id)));
    }
    (arc, cs_candidate, cr_result, c, n_primes, children)
}

fn main() {
    let (arc, cs_candidate, cr_result, mut c, mut n_primes, children) = init();
    while n_primes < 5000000 {
        for _ in 0..N_THREADS {
            cs_candidate.send(c).unwrap();
            c.iterate();
        }
        let mut more_primes: Vec<u128> = Vec::new();
        for _ in 0..N_THREADS {
            //println!("waiting for result...");
            let d = cr_result.recv().unwrap();
            //println!("result received {:?}", d);
            match d.is_prime {
                Some(true) => {
                    n_primes += 1;
                    more_primes.push(d.counter);
                }
                _ => {}
            }
        }
        more_primes.sort();
        let mut primes = arc.write().unwrap();
        primes.append(&mut more_primes);
    }
    c.is_done = true;
    for _ in 0..N_THREADS {
        cs_candidate.send(c).unwrap();
    }
    for child in children {
        let _ = child.join().unwrap();
    }
    //println!("{} numberes checked", N_THREADS);
    //println!("{:?}", arc);
}
