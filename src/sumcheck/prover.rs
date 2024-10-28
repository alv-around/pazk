use crate::polynomial::{assign_value, cast_mv_to_uv_polynomial, reduced_to_univariate};
use ark_ff::{Field, Zero};
use ark_poly::univariate::SparsePolynomial as UnivariatePolynomial;
use ark_poly::{
    multivariate::{SparsePolynomial, SparseTerm},
    polynomial::DenseMVPolynomial,
    Polynomial,
};

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
}
