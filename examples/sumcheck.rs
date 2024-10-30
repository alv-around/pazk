use ark_ff::fields::{Fp64, MontBackend, MontConfig};
use ark_poly::multivariate::Term;
use ark_poly::multivariate::{SparsePolynomial, SparseTerm};
use ark_poly::DenseMVPolynomial;
use pazk::sumcheck::{Prover, ProverMessage, Verifier, VerifierMessage};
use trpl::{self, Receiver, Sender};

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

    let (prover_tx, verifier_rx): (Sender<ProverMessage<F17>>, Receiver<ProverMessage<F17>>) =
        trpl::channel();
    let mut verifier = Verifier::new(verifier_rx);
    let mut prover = Prover::new(prover_tx, example_polynomial);

    trpl::run(async {
        verifier.listen().await;
        prover.prove().await;
        verifier.listen().await;
        prover.prove().await;
        verifier.listen().await;
        prover.prove().await;
        verifier.listen().await;
    })
}
