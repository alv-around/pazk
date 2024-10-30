use crate::sumcheck::ProverState;
use crate::sumcheck::VerifierState;
use ark_ff::Field;
use ark_poly::multivariate::{SparsePolynomial, SparseTerm};
use ark_poly::univariate::SparsePolynomial as UnivariatePolynomial;
use trpl::{self, Receiver, Sender};

pub enum ProverMessage<F: Field> {
    Statement(
        Sender<VerifierMessage<F>>,
        SparsePolynomial<F, SparseTerm>,
        F,
    ),
    Argument(UnivariatePolynomial<F>),
}

pub enum VerifierMessage<F: Field> {
    Confirmation,
    Ok(F),
    Failure(String),
    Sucess,
}

pub struct Prover<F: Field> {
    tx: Sender<ProverMessage<F>>,
    rx: Receiver<VerifierMessage<F>>,
    state: ProverState<F>,
}

pub struct Verifier<F: Field> {
    tx: Option<Sender<VerifierMessage<F>>>,
    rx: Receiver<ProverMessage<F>>,
    state: Option<VerifierState<F>>,
}

impl<F: Field> Prover<F> {
    pub fn new(tx: Sender<ProverMessage<F>>, poly: SparsePolynomial<F, SparseTerm>) -> Self {
        let (v_tx, p_rx) = trpl::channel();
        let state = ProverState::<F>::new(poly.clone());
        let solution = state.calculate_sum();
        let message = ProverMessage::Statement(v_tx, poly, solution);
        tx.send(message)
            .expect("unable to communicate with verifier");
        Prover {
            tx,
            rx: p_rx,
            state,
        }
    }

    pub async fn prove(&mut self) {
        match self.rx.recv().await {
            Some(message) => match message {
                VerifierMessage::Confirmation => {
                    let univariate_poly = self.state.calculate_round_poly();
                    self.tx.send(ProverMessage::Argument(univariate_poly));
                }
                VerifierMessage::Ok(random_challenge) => {
                    self.state.update_random_vars(random_challenge);
                    let univariate_poly = self.state.calculate_round_poly();
                    self.tx.send(ProverMessage::Argument(univariate_poly));
                }
                VerifierMessage::Sucess => {
                    println!("Verification Succeded!!");
                }
                VerifierMessage::Failure(err_message) => {
                    println!("Verification failed: {}", err_message);
                }
            },
            None => (),
        }
    }
}

impl<F: Field> Verifier<F> {
    pub fn new(rx: Receiver<ProverMessage<F>>) -> Self {
        Verifier {
            tx: None,
            rx,
            state: None,
        }
    }

    pub async fn listen(&mut self) {
        match self.rx.recv().await {
            Some(message) => match message {
                ProverMessage::Statement(tx, poly, solution) => {
                    self.registration(tx, poly, solution)
                }
                ProverMessage::Argument(univariate_poly) => self.verify_step(univariate_poly),
            },
            None => {
                if self.state.is_some() {
                    self.state = None;
                }
                if self.tx.is_some() {
                    self.tx = None;
                }
            }
        }
    }

    fn registration(
        &mut self,
        tx: Sender<VerifierMessage<F>>,
        poly: SparsePolynomial<F, SparseTerm>,
        solution: F,
    ) {
        if self.tx.is_some() || self.state.is_some() {
            tx.send(VerifierMessage::Failure(
                "Other verification taking place".to_string(),
            ));
        } else {
            tx.send(VerifierMessage::Confirmation);
            self.tx = Some(tx);
            self.state = Some(VerifierState::new(solution, poly));
        }
    }

    fn verify_step(&mut self, univariate_poly: UnivariatePolynomial<F>) {
        let message = match &mut self.state {
            Some(state) => {
                let random_challenge = state.verify_round(univariate_poly);
                VerifierMessage::Ok(random_challenge)
            }
            None => VerifierMessage::Failure("Nothing to verify".to_string()),
        };

        match &self.tx {
            Some(tx) => tx.send(message).expect("Communication Failure"),
            None => (),
        }
    }
}
