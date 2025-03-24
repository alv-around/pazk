use ark_ff::Field;
use ark_poly::{
    multivariate::{SparsePolynomial, SparseTerm, Term},
    univariate::SparsePolynomial as UnivariatePolynomial,
    DenseMVPolynomial,
};
use itertools::{Either, Itertools};
use std::collections::HashSet;

type Factor = (usize, usize);

/// Assigns a value to an specific variable of the polynomial
pub fn assign_value<F: Field>(
    polynomial: SparsePolynomial<F, SparseTerm>,
    variable: usize,
    value: F,
) -> SparsePolynomial<F, SparseTerm> {
    assert!(
        variable < polynomial.num_vars,
        "Invalid variable: index has to be in range [0 , .. , i-1]"
    );

    let mut new_terms = vec![];
    for (coeff, term) in polynomial.terms {
        let (matches, failure): (Vec<Factor>, Vec<Factor>) =
            term.iter().partition_map(|r| match r {
                (var, _power) if *var == variable => Either::Left(r),
                (var, power) if *var > variable => Either::Right((*var - 1, *power)),
                _ => Either::Right(*r),
            });

        let new_coeff: F = matches
            .iter()
            .fold(coeff, |acc, (_, i)| acc * value.pow([*i as u64]));
        new_terms.push((new_coeff, SparseTerm::new(failure)));
    }

    // num_vars remain the same, otherwise variables must be reindex
    SparsePolynomial::from_coefficients_vec(polynomial.num_vars - 1, new_terms)
}

fn assign_values<F: Field>(
    polynomial: &SparsePolynomial<F, SparseTerm>,
    values: Vec<(usize, F)>,
) -> SparsePolynomial<F, SparseTerm> {
    let mut reduced_polynomial = polynomial.clone();
    let rev_sorted_values = values.iter().sorted_by(|a, b| Ord::cmp(&b.0, &a.0));
    for (variable, value) in rev_sorted_values {
        reduced_polynomial = assign_value(reduced_polynomial, *variable, *value);
    }
    reduced_polynomial
}

pub fn reduced_to_univariate<F: Field>(
    mvpoly: &SparsePolynomial<F, SparseTerm>,
    values: Vec<(usize, F)>,
) -> SparsePolynomial<F, SparseTerm> {
    let mut variables = HashSet::new();
    for (var, _value) in &values {
        variables.insert(var); // Insert the key into the HashSet, ensuring uniqueness
    }

    assert_eq!(
        variables.len(),
        mvpoly.num_vars - 1,
        "provide values for d-1 variables, where d is the number of variables in the given multivariate polynomial"
    );

    assign_values(mvpoly, values)
}

pub fn cast_mv_to_uv_polynomial<F: Field>(
    single_var_mv_poly: SparsePolynomial<F, SparseTerm>,
) -> UnivariatePolynomial<F> {
    assert_eq!(single_var_mv_poly.num_vars, 1);
    let mut univariate_terms: Vec<(usize, F)> = Vec::new();
    // Iterate through the terms of the multivariate polynomial
    for (coeff, term) in single_var_mv_poly.terms {
        // Check if the term involves the desired variable
        match term.first() {
            Some((_, exp)) => univariate_terms.push((*exp, coeff)),
            None => univariate_terms.push((0, coeff)),
        }
    }
    UnivariatePolynomial::from_coefficients_vec(univariate_terms)
}
#[cfg(test)]
mod test {
    use super::*;
    use ark_ff::fields::{Fp64, MontBackend, MontConfig};
    use ark_poly::multivariate::SparseTerm;
    use ark_poly::DenseMVPolynomial;

    #[derive(MontConfig)]
    #[modulus = "17"]
    #[generator = "3"]
    pub struct F17Config;
    pub type F17 = Fp64<MontBackend<F17Config, 1>>;

    /// examples and solutions taken from SumCheck example in
    /// Thaler's Chp. 4
    fn setup() -> SparsePolynomial<F17, SparseTerm> {
        // Create a multivariate polynomial in 3 variables, with 4 terms:
        // 2*x_0^3 + x_0*x_2 + x_1*x_2
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
    fn test_assign_first_variable() {
        let poly = setup();
        let should = SparsePolynomial::from_coefficients_vec(
            2,
            vec![
                (F17::from(2), SparseTerm::new(vec![])),
                (F17::from(1), SparseTerm::new(vec![(1, 1)])),
                (F17::from(1), SparseTerm::new(vec![(0, 1), (1, 1)])),
            ],
        );
        let poly_reduced = assign_value(poly, 0, F17::from(1));
        assert_eq!(should, poly_reduced);
    }

    #[test]
    fn test_assign_last_variable() {
        let poly = setup();
        let should = SparsePolynomial::from_coefficients_vec(
            2,
            vec![
                (F17::from(2), SparseTerm::new(vec![(0, 3)])),
                (F17::from(1), SparseTerm::new(vec![(0, 1)])),
                (F17::from(1), SparseTerm::new(vec![(1, 1)])),
            ],
        );
        let poly_reduced = assign_value(poly, 2, F17::from(1));
        assert_eq!(should, poly_reduced);
    }

    #[test]
    fn test_assign_multiple_variables() {
        // 2*x_0^3 + x_0*x_2 + x_1*x_2
        let poly = setup();
        let should = SparsePolynomial::from_coefficients_vec(
            0,
            vec![(F17::from(4), SparseTerm::new(vec![]))],
        );
        let values = vec![(0, F17::from(1)), (1, F17::from(1)), (2, F17::from(1))];
        let poly_reduced = assign_values(&poly, values);
        assert_eq!(should, poly_reduced);
    }
}
