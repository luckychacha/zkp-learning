use halo2_proofs::arithmetic::Field;
use halo2_proofs::circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value};
use halo2_proofs::dev::MockProver;
use halo2_proofs::pasta::group::ff;
use halo2_proofs::pasta::Fp;
use halo2_proofs::plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Selector};
use halo2_proofs::poly::Rotation;
use std::alloc::Layout;

struct ACell<F: ff::Field>(AssignedCell<F, F>);

#[derive(Debug, Clone)]
struct FibonacciConfig {
    pub advice: [Column<Advice>; 3],
    pub selector: Selector,
}

#[derive(Debug, Clone)]
struct FibonacciChip<F: ff::Field> {
    config: FibonacciConfig,
    _marker: std::marker::PhantomData<F>,
}

impl<F: ff::Field> FibonacciChip<F> {
    fn construct(config: FibonacciConfig) -> Self {
        Self {
            config,
            _marker: std::marker::PhantomData,
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>, advice: [Column<Advice>; 3]) -> FibonacciConfig {
        let [col_a, col_b, col_c] = advice;
        let selector = meta.selector();

        meta.enable_equality(col_a);
        meta.enable_equality(col_b);
        meta.enable_equality(col_c);

        meta.create_gate("add", |meta| {
            let s = meta.query_selector(selector);
            let a = meta.query_advice(col_a, Rotation::cur());
            let b = meta.query_advice(col_b, Rotation::cur());
            let c = meta.query_advice(col_c, Rotation::cur());
            vec![s * (a + b - c)]
        });

        FibonacciConfig {
            advice: [col_a, col_b, col_c],
            selector,
        }
    }

    fn assign_first_row(
        &self,
        mut layouter: impl Layouter<F>,
        a: Value<F>,
        b: Value<F>,
    ) -> Result<(ACell<F>, ACell<F>, ACell<F>), Error> {
        layouter.assign_region(
            || "first row",
            |mut region| {
                self.config.selector.enable(&mut region, 0)?;
                let a_cell = region
                    .assign_advice(|| "a", self.config.advice[0], 0, || a)
                    .map(ACell)?;
                let b_cell = region
                    .assign_advice(|| "b", self.config.advice[1], 0, || b)
                    .map(ACell)?;

                let c_val = a.and_then(|a| b.map(|b| a + b));

                let c_cell = region
                    .assign_advice(|| "c", self.config.advice[2], 0, || a + b)
                    .map(ACell)?;

                Ok((a_cell, b_cell, c_cell))
            },
        )
    }

    fn assign_row(
        &self,
        mut layout: impl Layouter<F>,
        prev_b: &ACell<F>,
        prev_c: &ACell<F>,
    ) -> Result<ACell<F>, Error> {
        layout.assign_region(
            || "next row",
            |mut region| {
                self.config.selector.enable(&mut region, 0)?;

                prev_b
                    .0
                    .copy_advice(|| "a", &mut region, self.config.advice[0], 0)?;
                prev_c
                    .0
                    .copy_advice(|| "b", &mut region, self.config.advice[1], 0)?;

                let c_val = prev_b.0.value().copied() + prev_c.0.value();

                let c_cell = region
                    .assign_advice(|| "c", self.config.advice[2], 0, || c_val)
                    .map(ACell)?;

                Ok(c_cell)
            },
        )
    }
}

#[derive(Default)]
struct MyCircuit<F> {
    pub a: Value<F>,
    pub b: Value<F>,
}

impl<F: ff::Field> Circuit<F> for MyCircuit<F> {
    type Config = FibonacciConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let col_a = meta.advice_column();
        let col_b = meta.advice_column();
        let col_c = meta.advice_column();
        FibonacciChip::configure(meta, [col_a, col_b, col_c])
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let chip = FibonacciChip::construct(config);

        let (_, mut prev_b, mut prev_c) =
            chip.assign_first_row(layouter.namespace(|| "assign first row"), self.a, self.b)?;

        for _i in 3..10 {
            let c_cell = chip.assign_row(layouter.namespace(|| "next row"), &prev_b, &prev_c)?;
            prev_b = prev_c;
            prev_c = c_cell;
        }

        Ok(())
    }
}

fn main() {
    let k = 4;
    let a = Fp::from(1);
    let b = Fp::from(1);
    let circuit = MyCircuit {
        a: Value::known(a),
        b: Value::known(b),
    };

    let prover = MockProver::run(k, &circuit, vec![]).unwrap();

    prover.assert_satisfied();
}
