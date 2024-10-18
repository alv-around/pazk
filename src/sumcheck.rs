use crate::polynomial::{assign_value, cast_mv_to_uv_polynomial, reduced_to_univariate};
use ark_ff::{Field, Zero};
use ark_poly::univariate::SparsePolynomial as UnivariatePolynomial;
use ark_poly::{
    multivariate::{SparsePolynomial, SparseTerm},
    polynomial::DenseMVPolynomial,
    Polynomial,
};
use ark_std::test_rng;

pub struct Verifier<F: Field> {
    solution: F,
    poly: SparsePolynomial<F, SparseTerm>,
    total_rounds: usize,
    actual_round: usize,
    running_poly: UnivariatePolynomial<F>,
    rs: Vec<F>,
}

impl<F: Field> Verifier<F> {
    pub fn new(result: F, poly: SparsePolynomial<F, SparseTerm>) -> Self {
        let total_rounds = poly.num_vars;
        Verifier {
            solution: result,
            poly,
            running_poly: UnivariatePolynomial::<F>::zero(),
            total_rounds,
            actual_round: 0,
            rs: Vec::with_capacity(total_rounds),
        }
    }

    pub fn verify_round(&mut self, round_poly: UnivariatePolynomial<F>) -> F {
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

pub struct Prover<F: Field> {
    poly: SparsePolynomial<F, SparseTerm>, // Use concrete type
    total_rounds: usize,
    actual_round: usize,
    rs: Vec<F>,
}

impl<F: Field> Prover<F> {
    pub fn new(poly: SparsePolynomial<F, SparseTerm>) -> Self {
        // Accept concrete type
        let total_rounds = poly.num_vars();
        Prover {
            poly,
            total_rounds,
            actual_round: 0,
            rs: Vec::with_capacity(total_rounds),
        }
    }

    // convert number into {0, 1}^domain
    fn number_to_domain(number: usize, domain: usize) -> Vec<F> {
        (0..domain)
            .map(|j| {
                if (number & (1 << j)) != 0 {
                    F::ONE
                } else {
                    F::ZERO
                }
            })
            .collect()
    }

    pub fn calculate_sum(&self) -> F {
        let mut result = F::ZERO;
        for i in 0..(1 << self.total_rounds) {
            let binary = Prover::number_to_domain(i, self.total_rounds);
            result += self.poly.evaluate(&binary);
        }
        result
    }

    pub fn calculate_round_poly(&self) -> UnivariatePolynomial<F> {
        let mut round_poly = SparsePolynomial::<F, SparseTerm>::zero();
        let remaining_rounds = self.total_rounds - self.actual_round - 1;
        for i in 0..(1 << remaining_rounds) {
            let binary: Vec<F> = Prover::number_to_domain(i, remaining_rounds);
            let values = std::iter::zip(1..=remaining_rounds, binary).collect();
            round_poly += &reduced_to_univariate(&self.poly, values);
        }
        cast_mv_to_uv_polynomial(round_poly)
    }

    pub fn update_random_vars(&mut self, r: F) {
        self.poly = assign_value(self.poly.clone(), 0, r);
        self.rs.push(r);
        self.actual_round += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_ff::fields::{Fp64, MontBackend, MontConfig};
    use ark_poly::multivariate::Term;

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
    fn test_prover_calculate_sum() {
        let poly = setup();
        let prover = Prover::new(poly);
        let solution = prover.calculate_sum();
        assert_eq!(solution, F17::from(12));

        let round1_poly = prover.calculate_round_poly();
        assert_eq!(
            round1_poly.evaluate(&F17::ZERO) + round1_poly.evaluate(&F17::ONE),
            prover.calculate_sum()
        );
    }

    #[test]
    fn test_verifier_verify_round() {
        let poly = setup();
        let prover = Prover::new(poly.clone());
        let mut verifier = Verifier::new(F17::from(12), poly);
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
        let mut verifier = Verifier::new(F17::from(12), poly);
        let random_poly =
            UnivariatePolynomial::from_coefficients_vec(vec![(2, F17::from(1)), (0, F17::from(1))]);
        verifier.verify_round(random_poly);
    }

    #[test]
    fn test_prover_verifier_interaction_ith_round() {
        let poly = setup();
        let mut prover = Prover::new(poly.clone());

        let rand_field = F17::from(2);
        let mut verifier = Verifier {
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
        let mut prover = Prover::new(poly.clone());
        prover.update_random_vars(rs[0]);
        let s2 = prover.calculate_round_poly();
        prover.update_random_vars(rs[1]);
        let s3 = prover.calculate_round_poly();

        let mut verifier = Verifier {
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
