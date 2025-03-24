use super::prover::ProverState;
use ark_ff::Field;
use ark_poly::{
    multivariate::{SparsePolynomial, SparseTerm},
    Polynomial,
};
use nimue::{
    plugins::ark::{FieldChallenges, FieldWriter},
    DuplexHash, Merlin,
};

pub struct SumCheckProof<'a, F: Field> {
    solution: F,
    polys: Vec<F>,
    transcript: &'a [u8],
}

pub fn prove<H, F>(
    merlin: &mut Merlin<H>,
    poly: SparsePolynomial<F, SparseTerm>,
) -> SumCheckProof<F>
where
    F: Field,
    H: DuplexHash,
    Merlin<H>: FieldWriter<F> + FieldChallenges<F>,
{
    let mut prover = ProverState::new(poly);
    let solution = prover.calculate_sum();
    let polys = Vec::new();
    for _ in 0..prover.total_rounds {
        let poly = prover.calculate_round_poly();
        let commit = poly.evaluate(&F::ZERO) + poly.evaluate(&F::ONE);
        merlin.add_scalars(&[commit]).unwrap();
        let r: [F; 1] = merlin.challenge_scalars().unwrap();
        prover.update_random_vars(r[0]);
    }

    let transcript = merlin.transcript();

    SumCheckProof {
        solution,
        polys,
        transcript,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sumcheck::fiat_shamir::SumCheckExtensionIOPattern;
    use ark_ff::fields::{Fp64, MontBackend, MontConfig};
    use ark_poly::multivariate::Term;
    use ark_poly::DenseMVPolynomial;
    use nimue::IOPattern;
    use sha2;

    #[derive(MontConfig)]
    #[modulus = "17"]
    #[generator = "3"]
    pub struct F17Config;
    pub type F17 = Fp64<MontBackend<F17Config, 1>>;
    type H = nimue::hash::legacy::DigestBridge<sha2::Sha256>;

    /// examples and solutions taken from SumCheck example in
    /// Thaler's Chp. 4
    fn setup() -> SparsePolynomial<F17, SparseTerm> {
        // Create a multivariate polynomial in 3 variables, with 4 terms:
        // /// // 2*x_0^3 + x_0*x_2 + x_1*x_2
        SparsePolynomial::from_coefficients_vec(
            3,
            vec![
                (F17::from(2), SparseTerm::new(vec![(0, 3)])),
                (F17::from(1), SparseTerm::new(vec![(0, 1), (2, 1)])),
                (F17::from(1), SparseTerm::new(vec![(1, 1), (2, 1)])),
            ],
        )
    }

    #[test]
    fn test_non_interactive_sumcheck() {
        let poly = setup();
        let transcript: IOPattern<H> =
            SumCheckExtensionIOPattern::<F17>::new_sumcheck("➕✅", &poly);
        let mut merlin = transcript.to_merlin();

        let _proof = prove(&mut merlin, poly);
    }
}
