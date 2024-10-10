use ark_ff::Field;
use ark_poly::{
    multivariate::SparsePolynomial, polynomial::multivariate::SparseTerm,
    polynomial::DenseMVPolynomial, Polynomial,
};

struct Verifier<F: Field> {
    H: F,
    rounds: usize,
}

impl<F: Field> Verifier<F> {
    pub fn new(result: F, rounds: usize) -> Self {
        Verifier { H: result, rounds }
    }
}

struct Prover<F: Field> {
    poly: SparsePolynomial<F, SparseTerm>, // Use concrete type
    rounds: usize,
}

impl<F: Field> Prover<F> {
    pub fn new(poly: SparsePolynomial<F, SparseTerm>) -> Self {
        // Accept concrete type
        let rounds = poly.num_vars();
        Prover { poly, rounds }
    }

    pub fn calculate_solution(&self) -> F {
        let mut result = F::ZERO;
        for i in 0..(1 << self.rounds) {
            let binary: Vec<F> = (0..self.rounds)
                .map(|j| if (i & (1 << j)) != 0 { F::ONE } else { F::ZERO })
                .collect();

            result += self.poly.evaluate(&binary); // Reference the binary slice
        }
        result // Return the accumulated result
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
    fn it_works() {
        let one = F17::from(18);
        assert_eq!(one, Field::ONE);
        let poly = setup();
        let point = vec![one, Field::ZERO, Field::ZERO];

        let result = poly.evaluate(&point);
        assert_eq!(result, F17::from(2));
    }
}
