use std::{thread};
use std::sync::{Arc, RwLock};

const N_THREADS: usize = 16;

#[derive(Clone, Copy, Debug)]
struct CompTrimple {
    counter: i128,
    thesqrt: i128,
    subcounter: i64,
    subthreshhold: i64,
    is_prime: bool,
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

fn is_prime(arc: Arc<RwLock<Vec<i128>>>, mut c: CompTrimple, _id: usize) -> CompTrimple {
    {
        let vec_ro = arc.read().unwrap();
        //println!("thread {_id} reads {:?}", vec_ro);
        for &d in vec_ro.iter() {
            //println!("reading counter:{} prime:{} threadid:{}", c.counter, d, _id);
            if c.counter % d == 0 {
                c.is_prime = false;
                break;
            }
            if d > c.thesqrt {
                c.is_prime = true;
                break;
            }
        }
    }
    c
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
        is_prime: false,
    };
    let arc = Arc::new(RwLock::new(vec));  
    while n_primes < 50000 {
        let mut children: [Option<thread::JoinHandle<CompTrimple>>; N_THREADS] = Default::default();
        for id in 0..N_THREADS {
            let v = arc.clone();
            children[id] = Some(thread::spawn(move|| is_prime(v, c, id)));
            c.iterate();
        }
        for child in children {
            match child {
                Some(child) => {
                    let d = child.join().unwrap();
                    //println!("{:?}", d);
                    if d.is_prime {
                        n_primes += 1;
                        let mut v = arc.write().unwrap();
                        v.push(d.counter);  
                    }
                },
                None => println!("none"),
            }
        }
        //println!("{} numberes checked", N_THREADS);
    }
println!("{:?}", arc);
}
