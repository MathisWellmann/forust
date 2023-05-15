use crate::errors::ForustError;
use crate::utils::items_to_strings;
use rand::rngs::StdRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize)]
pub enum SampleMethod {
    None,
    Random,
    Goss,
}

impl FromStr for SampleMethod {
    type Err = ForustError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "random" => Ok(SampleMethod::Random),
            "goss" => Ok(SampleMethod::Goss),
            _ => Err(ForustError::ParseString(
                s.to_string(),
                "SampleMethod".to_string(),
                items_to_strings(vec!["random", "goss"]),
            )),
        }
    }
}

// A sampler can be used to subset the data prior to fitting a new tree.
pub trait Sampler {
    /// Sample the data, returning a tuple, where the first item is the samples
    /// chosen for training, and the second are the samples excluded.
    fn sample(
        &mut self,
        rng: &mut StdRng,
        index: &[usize],
        grad: &mut [f32],
        hess: &mut [f32],
    ) -> (Vec<usize>, Vec<usize>);
}

pub struct RandomSampler {
    subsample: f32,
}

impl RandomSampler {
    #[allow(dead_code)]
    pub fn new(subsample: f32) -> Self {
        RandomSampler { subsample }
    }
}

impl Sampler for RandomSampler {
    fn sample(
        &mut self,
        rng: &mut StdRng,
        index: &[usize],
        grad: &mut [f32],
        hess: &mut [f32],
    ) -> (Vec<usize>, Vec<usize>) {
        let subsample = self.subsample;
        let mut chosen = Vec::new();
        let mut excluded = Vec::new();
        for i in index {
            if rng.gen_range(0.0..1.0) < subsample {
                chosen.push(*i);
            } else {
                excluded.push(*i)
            }
        }
        (chosen, excluded)
    }
}

#[allow(dead_code)]
pub struct GossSampler {
    a: f64, // https://lightgbm.readthedocs.io/en/latest/Parameters.html#top_rate
    b: f64, // https://lightgbm.readthedocs.io/en/latest/Parameters.html#other_rate
}

impl Default for GossSampler {
    fn default() -> Self {
        GossSampler { a: 0.2, b: 0.1 }
    }
}

#[allow(dead_code)]
impl GossSampler {
    pub fn new(a: f64, b: f64) -> Self {
        if !(a >= 0. && a <= 1.) {
            panic!("move to gradientbooster constructor");
        } else if !(b >= 0. && b <= 1.) {
            panic!("move to gradientbooster constructor");
        } else {
            GossSampler { a, b }
        }
    }
}

impl Sampler for GossSampler {
    #[allow(unused_variables)]
    fn sample(
        &mut self,
        rng: &mut StdRng,
        index: &[usize],
        grad: &mut [f32],
        hess: &mut [f32],
    ) -> (Vec<usize>, Vec<usize>) {
        let fact = ((1. - self.a) / self.b) as f32;
        let topN = (self.a * index.len() as f64) as usize;
        let randN = (self.b * index.len() as f64) as usize;

        // sort gradient by absolute value from highest to lowest
        let mut sorted = (0..index.len()).collect::<Vec<_>>();
        sorted.sort_unstable_by(|&a, &b| grad[b].abs().total_cmp(&grad[a].abs()));

        // select the topN largest gradients
        let topSet = &sorted[0..topN];

        // sample the rest based on randN
        let subsample = (randN / (index.len() - topN)) as f64;
        let mut randomSet = Vec::new();
        for i in &sorted[topN..sorted.len()] {
            if rng.gen_range(0.0..1.0) < subsample {
                randomSet.push(*i);
            }
        }

        let usedSet = [topSet, &randomSet].concat();

        // literally, multiply the weight *= hess and grad
        for i in &randomSet {
            grad[*i] *= fact;
            hess[*i] *= fact;
        }

        (usedSet, Vec::new())
    }
}
