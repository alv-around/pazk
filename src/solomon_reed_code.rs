use ark_ff::Field;

pub struct ReedSolomon<F>(Vec<F>);

impl<F: Field> ReedSolomon<F> {
    pub fn new(a: Vec<F>) -> Self {
        ReedSolomon(a)
    }

    pub fn draw_random() -> F {
        let mut rng = ark_std::test_rng();
        F::rand(&mut rng)
    }

    // calculates h(a1, ..., an) = sum^{n}_{i=1} a_i * r^{i-1)}
    pub fn univariate_fingerprint(&self, r: &F) -> F {
        let mut h = F::ZERO;
        let mut x = F::ONE;
        for a_i in self.0.iter() {
            h += *a_i * x;
            x *= r;
        }

        h
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_ff::fields::{Fp64, MontBackend, MontConfig};
    use ark_test_curves::bls12_381::Fr as F;

    #[test]
    fn test_custom_field() {
        #[derive(MontConfig)]
        #[modulus = "17"]
        #[generator = "3"]
        pub struct FqConfig;
        pub type Fq = Fp64<MontBackend<FqConfig, 1>>;

        let a = Fq::from(9);
        let b = Fq::from(10);

        let rs = ReedSolomon::new(vec![a, b]);
        let r = Fq::from(8);
        let result = Fq::from(9 + 80);
        assert_eq!(rs.univariate_fingerprint(&r), result);
    }

    #[test]
    fn test_inconsistent_data() {
        #[derive(MontConfig)]
        #[modulus = "17"]
        #[generator = "3"]
        pub struct FqConfig;
        pub type Fq = Fp64<MontBackend<FqConfig, 1>>;

        let a = vec![Fq::from(9), Fq::from(10)];
        let rs_a = ReedSolomon::new(a);
        let rs_b = ReedSolomon::new(vec![Fq::from(9), Fq::from(11)]);

        let r = Fq::from(8);
        assert_ne!(
            rs_a.univariate_fingerprint(&r),
            rs_b.univariate_fingerprint(&r)
        );
    }

    #[test]
    fn test_with_arkcurve() {
        let a = F::from(9);
        let b = F::from(10);

        let rs = ReedSolomon::new(vec![a, b]);
        let r = F::from(8);
        assert_eq!(rs.univariate_fingerprint(&r), F::from(89));
    }
}
