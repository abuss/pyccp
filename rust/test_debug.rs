use pyccp_core::propagators::{AllDiff, Registry};
use pyccp_core::store::Store;

fn main() {
    let mut s = Store::new(&[vec![1, 2], vec![1, 2], vec![1, 2, 3]]);
    println!("Initial domains: {:?}", vec![s.domain(0).to_vec(), s.domain(1).to_vec(), s.domain(2).to_vec()]);
    
    let mut r = Registry::new();
    let mut constraint = AllDiff::new(vec![0, 1, 2], &s);
    println!("Algorithm selected: {:?}", constraint.state.algorithm);
    r.add(Box::new(constraint));
    
    s.commit(0, 1).unwrap();
    println!("After commit(0, 1): {:?}", vec![s.domain(0).to_vec(), s.domain(1).to_vec(), s.domain(2).to_vec()]);
    
    s.commit(1, 2).unwrap();
    println!("After commit(1, 2): {:?}", vec![s.domain(0).to_vec(), s.domain(1).to_vec(), s.domain(2).to_vec()]);
    
    r.fixpoint(&mut s).unwrap();
    println!("After fixpoint: {:?}", vec![s.domain(0).to_vec(), s.domain(1).to_vec(), s.domain(2).to_vec()]);
}
