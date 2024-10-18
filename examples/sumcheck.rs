use ark_ff::fields::{Fp64, MontBackend, MontConfig};
use ark_poly::multivariate::Term;
use ark_poly::multivariate::{SparsePolynomial, SparseTerm};
use ark_poly::DenseMVPolynomial;
use pazk::sumcheck::{Prover, Verifier};

#[derive(MontConfig)]
#[modulus = "17"]
#[generator = "3"]
pub struct F17Config;
pub type F17 = Fp64<MontBackend<F17Config, 1>>;

fn main() {
    // examples taken from SumCheck example in Thaler's book chp 4
    let example_polynomial = SparsePolynomial::from_coefficients_vec(
        3,
        vec![
            (F17::from(2), SparseTerm::new(vec![(0, 3)])),
            (F17::from(1), SparseTerm::new(vec![(0, 1), (1, 1)])),
            (F17::from(1), SparseTerm::new(vec![(1, 1), (2, 1)])),
        ],
    );

    // TODO: add RC to polynomial
    let mut prover = Prover::new(example_polynomial.clone());
    let mut verifier = Verifier::new(prover.calculate_sum(), example_polynomial);

    // TODO: add channel communication between Verifier and Prover

    // round 1
    let s1 = prover.calculate_round_poly();
    let r1 = verifier.verify_round(s1);
    prover.update_random_vars(r1);

    // round 2
    let s2 = prover.calculate_round_poly();
    let r2 = verifier.verify_round(s2);
    prover.update_random_vars(r2);

    // final round
    let s3 = prover.calculate_round_poly();
    verifier.verify_round(s3);
}
