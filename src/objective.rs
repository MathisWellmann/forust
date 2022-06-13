use crate::data::FloatData;
use serde::{Deserialize, Serialize};

type ObjFn<T> = fn(&[T], &[T], &[T]) -> Vec<T>;

#[derive(Debug, Deserialize, Serialize)]
pub enum ObjectiveType {
    LogLoss,
    SquaredLoss,
}

pub fn gradient_hessian_callables<T: FloatData<T>>(
    objective_type: &ObjectiveType,
) -> (ObjFn<T>, ObjFn<T>) {
    match objective_type {
        ObjectiveType::LogLoss => (LogLoss::calc_grad, LogLoss::calc_hess),
        ObjectiveType::SquaredLoss => (SquaredLoss::calc_grad, SquaredLoss::calc_hess),
    }
}

pub trait ObjectiveFunction<T>
where
    T: FloatData<T>,
{
    fn calc_loss(y: &[T], yhat: &[T], sample_weight: &[T]) -> Vec<T>;
    fn calc_grad(y: &[T], yhat: &[T], sample_weight: &[T]) -> Vec<T>;
    fn calc_hess(y: &[T], yhat: &[T], sample_weight: &[T]) -> Vec<T>;
}

#[derive(Default)]
pub struct LogLoss {}

impl<T> ObjectiveFunction<T> for LogLoss
where
    T: FloatData<T>,
{
    #[inline]
    fn calc_loss(y: &[T], yhat: &[T], sample_weight: &[T]) -> Vec<T> {
        y.iter()
            .zip(yhat)
            .zip(sample_weight)
            .map(|((y_, yhat_), w_)| {
                let yhat_ = T::ONE / (T::ONE + (-*yhat_).exp());
                -(*y_ * yhat_.ln() + (T::ONE - *y_) * ((T::ONE - yhat_).ln())) * *w_
            })
            .collect()
    }

    #[inline]
    fn calc_grad(y: &[T], yhat: &[T], sample_weight: &[T]) -> Vec<T> {
        y.iter()
            .zip(yhat)
            .zip(sample_weight)
            .map(|((y_, yhat_), w_)| {
                let yhat_ = T::ONE / (T::ONE + (-*yhat_).exp());
                (yhat_ - *y_) * *w_
            })
            .collect()
    }
    #[inline]
    fn calc_hess(_: &[T], yhat: &[T], sample_weight: &[T]) -> Vec<T> {
        yhat.iter()
            .zip(sample_weight)
            .map(|(yhat_, w_)| {
                let yhat_ = T::ONE / (T::ONE + (-*yhat_).exp());
                yhat_ * (T::ONE - yhat_) * *w_
            })
            .collect()
    }
}

#[derive(Default)]
pub struct SquaredLoss {}

impl<T> ObjectiveFunction<T> for SquaredLoss
where
    T: FloatData<T>,
{
    #[inline]
    fn calc_loss(y: &[T], yhat: &[T], sample_weight: &[T]) -> Vec<T> {
        y.iter()
            .zip(yhat)
            .zip(sample_weight)
            .map(|((y_, yhat_), w_)| {
                let s = *y_ - *yhat_;
                s * s * *w_
            })
            .collect()
    }

    #[inline]
    fn calc_grad(y: &[T], yhat: &[T], sample_weight: &[T]) -> Vec<T> {
        y.iter()
            .zip(yhat)
            .zip(sample_weight)
            .map(|((y_, yhat_), w_)| (*yhat_ - *y_) * *w_)
            .collect()
    }

    #[inline]
    fn calc_hess(_: &[T], _: &[T], sample_weight: &[T]) -> Vec<T> {
        sample_weight.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_logloss_loss() {
        let y = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let yhat1 = vec![-1.0, -1.0, -1.0, 1.0, 1.0, 1.0];
        let w = vec![1.; y.len()];
        let l1 = LogLoss::calc_loss(&y, &yhat1, &w);
        let yhat2 = vec![0.0, 0.0, -1.0, 1.0, 0.0, 1.0];
        let l2 = LogLoss::calc_loss(&y, &yhat2, &w);
        assert!(l1.iter().sum::<f64>() < l2.iter().sum::<f64>());
    }

    #[test]
    fn test_logloss_grad() {
        let y = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let yhat1 = vec![-1.0, -1.0, -1.0, 1.0, 1.0, 1.0];
        let w = vec![1.; y.len()];
        let g1 = LogLoss::calc_grad(&y, &yhat1, &w);
        let yhat2 = vec![0.0, 0.0, -1.0, 1.0, 0.0, 1.0];
        let g2 = LogLoss::calc_grad(&y, &yhat2, &w);
        assert!(g1.iter().sum::<f64>() < g2.iter().sum::<f64>());
    }

    #[test]
    fn test_logloss_hess() {
        let y = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let yhat1 = vec![-1.0, -1.0, -1.0, 1.0, 1.0, 1.0];
        let w = vec![1.; y.len()];
        let h1 = LogLoss::calc_hess(&y, &yhat1, &w);
        let yhat2 = vec![0.0, 0.0, -1.0, 1.0, 0.0, 1.0];
        let h2 = LogLoss::calc_hess(&y, &yhat2, &w);
        assert!(h1.iter().sum::<f64>() < h2.iter().sum::<f64>());
    }
}
