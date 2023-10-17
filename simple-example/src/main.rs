use ff::Field;
use halo2_proofs::circuit::{AssignedCell, Chip, Layouter, Value};
use halo2_proofs::plonk::{Advice, Assigned, Column, Error, Instance, Selector};

trait NumericInstructions<F: Field>: Chip<F> {
    type Num;

    fn load_private(&self, layouter: impl Layouter<F>, a: Value<F>) -> Result<Self::Num, Error>;

    fn load_constant(&self, layouter: impl Layouter<F>, constant: F) -> Result<Self::Num, Error>;

    fn mul(
        &self,
        layouter: impl Layouter<F>,
        a: Self::Num,
        b: Self::Num,
    ) -> Result<Self::Num, Error>;

    // Expose a number as a public input to the circuit.
    fn expose_public(
        &self,
        layouter: impl Layouter<F>,
        num: Self::Num,
        row: usize,
    ) -> Result<(), Error>;
}

struct FieldChip<F: Field> {
    config: FieldConfig,
    _marker: std::marker::PhantomData<F>,
}

struct FieldConfig {
    advice: [Column<Advice>; 2],
    instance: Column<Instance>,
    s_mul: Selector,
}

struct Number<F: Field>(AssignedCell<F, F>);

fn main() {
    println!("Hello, lesson 1: Simple example!");
}
