use halo2_proofs::pasta::group::ff;
use halo2_proofs::plonk::{Advice, Column, Selector};

#[derive(Debug, Clone)]
struct FiboConfig {
    pub advice: [Column<Advice>; 3],
    pub selector: Selector,
}

struct FiboChip<F: ff::Field> {
    config: FiboConfig,
    _marker: std::marker::PhantomData<F>,
}

fn main() {
    println!("Hello, lesson 2: Fibonacci!");
}
