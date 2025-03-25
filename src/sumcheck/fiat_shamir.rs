use ark_ff::Field;
use ark_poly::multivariate::{SparsePolynomial, SparseTerm};
use nimue::plugins::ark::*;

type Poly<F> = SparsePolynomial<F, SparseTerm>;

pub trait SumCheckExtensionIOPattern<F: Field> {
    fn new_sumcheck(domsep: &str, poly: &Poly<F>) -> Self;
    fn add_sumcheck(self, poly: &Poly<F>) -> Self;
}

impl<F, H> SumCheckExtensionIOPattern<F> for IOPattern<H>
where
    F: Field,
    H: DuplexHash,
    IOPattern<H>: FieldIOPattern<F>,
{
    fn new_sumcheck(domsep: &str, poly: &Poly<F>) -> Self {
        IOPattern::new(domsep).add_sumcheck(poly)
    }

    fn add_sumcheck(mut self, poly: &Poly<F>) -> Self {
        // FIXME: lazy fiat-shamir. prover should commit to poly.
        for _ in 0..poly.num_vars {
            self = self
                .add_scalars(1, "Univariate polynomial coefficients")
                .challenge_scalars(1, "random scalar challenge");
        }

        self
    }
}
