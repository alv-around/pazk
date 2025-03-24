use ark_ff::{Field, Zero};
use ark_poly::univariate::SparsePolynomial as UnivariatePolynomial;
use ark_poly::{
    multivariate::{SparsePolynomial, SparseTerm},
    Polynomial,
};
use ark_std::test_rng;

pub struct VerifierState<F: Field> {
    solution: F,
    poly: SparsePolynomial<F, SparseTerm>,
    total_rounds: usize,
    actual_round: usize,
    running_poly: UnivariatePolynomial<F>,
    rs: Vec<F>,
}

impl<F: Field> VerifierState<F> {
    pub fn new(result: F, poly: SparsePolynomial<F, SparseTerm>) -> Self {
        let total_rounds = poly.num_vars;
        VerifierState {
            solution: result,
            poly,
            running_poly: UnivariatePolynomial::<F>::zero(),
            total_rounds,
            actual_round: 0,
            rs: Vec::with_capacity(total_rounds),
        }
    }

    pub fn get_total_rounds(&self) -> usize {
        self.total_rounds
    }

    pub fn get_actual_rounds(&self) -> usize {
        self.actual_round
    }

    pub fn verify_round(&mut self, round_poly: UnivariatePolynomial<F>) -> F {
        // TODO: Improve Error handling with result
        assert!(
            self.actual_round < self.total_rounds,
            "Invalid round number"
        );

        let round_value = round_poly.evaluate(&F::ZERO) + round_poly.evaluate(&F::ONE);
        if self.actual_round == 0 {
            assert_eq!(round_value, self.solution);
        } else {
            assert_eq!(
                round_value,
                self.running_poly.evaluate(self.rs.last().unwrap())
            );
        }

        self.actual_round += 1;
        let field = F::rand(&mut test_rng());
        self.rs.push(field);
        self.running_poly = round_poly;

        if self.actual_round == self.total_rounds {
            assert_eq!(
                self.running_poly.evaluate(&field),
                self.poly.evaluate(&self.rs)
            );
        }

        field
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sumcheck::prover::ProverState;
    use ark_ff::fields::{Fp64, MontBackend, MontConfig};
    use ark_poly::{multivariate::Term, DenseMVPolynomial};

    #[derive(MontConfig)]
    #[modulus = "17"]
    #[generator = "3"]
    pub struct F17Config;
    pub type F17 = Fp64<MontBackend<F17Config, 1>>;

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
    fn test_verifier_verify_round() {
        let poly = setup();
        let prover = ProverState::new(poly.clone());
        let mut verifier = VerifierState::new(F17::from(12), poly);
        let round1_poly = prover.calculate_round_poly();
        let should_poly = UnivariatePolynomial::from_coefficients_vec(vec![
            (3, F17::from(8)),
            (1, F17::from(2)),
            (0, F17::from(1)),
        ]);
        assert_eq!(round1_poly, should_poly);
        verifier.verify_round(round1_poly);
    }

    #[test]
    #[should_panic]
    fn test_verifier_wrong_poly() {
        let poly = setup();
        let mut verifier = VerifierState::new(F17::from(12), poly);
        let random_poly =
            UnivariatePolynomial::from_coefficients_vec(vec![(2, F17::from(1)), (0, F17::from(1))]);
        verifier.verify_round(random_poly);
    }

    #[test]
    fn test_prover_verifier_interaction_ith_round() {
        let poly = setup();
        let mut prover = ProverState::new(poly.clone());

        let rand_field = F17::from(2);
        let mut verifier = VerifierState {
            total_rounds: 3,
            actual_round: 1,
            poly,
            rs: vec![rand_field],
            solution: F17::from(12),
            running_poly: UnivariatePolynomial::from_coefficients_vec(vec![
                (3, F17::from(8)),
                (1, F17::from(2)),
                (0, F17::from(1)),
            ]),
        };
        prover.update_random_vars(rand_field);
        let round2_poly = prover.calculate_round_poly();
        let should_poly = UnivariatePolynomial::from_coefficients_vec(vec![(1, F17::from(1))]);
        assert_eq!(round2_poly, should_poly);
        verifier.verify_round(round2_poly);
    }

    #[test]
    fn test_verifier_final_round() {
        let poly = setup();
        let rs = vec![F17::from(2), F17::from(3)];
        let mut prover = ProverState::new(poly.clone());
        prover.update_random_vars(rs[0]);
        let s2 = prover.calculate_round_poly();
        prover.update_random_vars(rs[1]);
        let s3 = prover.calculate_round_poly();

        let mut verifier = VerifierState {
            total_rounds: 3,
            actual_round: 2,
            running_poly: s2,
            poly,
            solution: F17::from(12),
            rs,
        };

        verifier.verify_round(s3);
    }
}
