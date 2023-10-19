use ff::Field;
use halo2_proofs::circuit::{AssignedCell, Chip, Layouter, Value};
use halo2_proofs::plonk::{
    Advice, Assigned, Column, ConstraintSystem, Error, Fixed, Instance, Selector,
};
use halo2_proofs::poly::Rotation;
use std::fmt;
use std::fmt::Formatter;

#[derive(Clone)]
pub struct MyError {
    pub msg: String,
}

impl fmt::Debug for MyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Debug: MyError {{ msg: {} }}", self.msg)
    }
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Display: MyError {{ msg: {} }}", self.msg)
    }
}

fn test_error(info: &str, res: bool) -> Result<(), MyError> {
    if res {
        Ok(())
    } else {
        Err(MyError {
            msg: info.to_string(),
        })
    }
}

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

impl<F: Field> FieldChip<F> {
    fn construct(config: <Self as Chip<F>>::Config) -> Self {
        Self {
            config,
            _marker: std::marker::PhantomData,
        }
    }

    fn configure(
        meta: &mut ConstraintSystem<F>,
        advice: [Column<Advice>; 2],
        instance: Column<Instance>,
        constant: Column<Fixed>,
    ) -> <Self as Chip<F>>::Config {
        meta.enable_equality(instance);
        meta.enable_constant(constant);
        for column in &advice {
            meta.enable_equality(*column);
        }

        let s_mul = meta.selector();

        meta.create_gate("mul", |meta| {
            let lhs = meta.query_advice(advice[0], Rotation::cur());
            let rhs = meta.query_advice(advice[1], Rotation::cur());
            let out = meta.query_advice(advice[0], Rotation::next());

            let s_mul = meta.query_selector(s_mul);
            vec![s_mul * (lhs * rhs - out)]
        });

        FieldConfig {
            advice,
            instance,
            s_mul,
        }
    }
}

impl<F: Field> Chip<F> for FieldChip<F> {
    type Config = FieldConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

#[derive(Debug, Clone)]
struct FieldConfig {
    advice: [Column<Advice>; 2],
    instance: Column<Instance>,
    s_mul: Selector,
}

struct Number<F: Field>(AssignedCell<F, F>);

impl<F: Field> NumericInstructions<F> for FieldChip<F> {
    type Num = Number<F>;

    fn load_private(
        &self,
        mut layouter: impl Layouter<F>,
        value: Value<F>,
    ) -> Result<Self::Num, Error> {
        let config = self.config();

        layouter.assign_region(
            || "load private",
            |mut region| {
                region
                    .assign_advice(|| "private input", config.advice[0], 0, || value)
                    .map(Number)
            },
        )
    }

    fn load_constant(
        &self,
        mut layouter: impl Layouter<F>,
        constant: F,
    ) -> Result<Self::Num, Error> {
        let config = self.config();

        layouter.assign_region(
            || "load constant",
            |mut region| {
                region
                    .assign_advice_from_constant(|| "constant value", config.advice[0], 0, constant)
                    .map(Number)
            },
        )
    }

    fn mul(
        &self,
        mut layouter: impl Layouter<F>,
        a: Self::Num,
        b: Self::Num,
    ) -> Result<Self::Num, Error> {
        let config = self.config();

        layouter.assign_region(
            || "mul",
            |mut region| {
                config.s_mul.enable(&mut region, 0)?;

                a.0.copy_advice(|| "lhs", &mut region, config.advice[0], 0)?;
                b.0.copy_advice(|| "rhs", &mut region, config.advice[1], 0)?;

                let value = a.0.value().copied() * b.0.value();

                region
                    .assign_advice(|| "lhs * rhs", config.advice[0], 1, || value)
                    .map(Number)
            },
        )
    }

    fn expose_public(
        &self,
        mut layouter: impl Layouter<F>,
        num: Self::Num,
        row: usize,
    ) -> Result<(), Error> {
        let config = self.config();

        layouter.constrain_instance(num.0.cell(), config.instance, row)
    }
}

fn main() {
    if let Ok(_) = test_error("ok", true) {
        println!("Ok");
    }
    if let Err(e) = test_error("error 1", false) {
        println!("Error: {}", e);
    }
    if let Err(e) = test_error("error 2", false) {
        println!("Error: {}", e);
    }
    println!("Hello, lesson 1: Simple example!");
}
