use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::ops::Range;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralNetwork {
    layers: Vec<Layer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]

struct Layer {
    nodes: Vec<Node>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Node {
    bias: f32,
    weights: Vec<f32>,
}

impl NeuralNetwork {
    pub fn new(sizes: Vec<usize>) -> NeuralNetwork {
        let mut rng = thread_rng();

        let mut layers = vec![];

        for (index, size) in sizes.iter().enumerate() {
            let mut nodes = vec![];

            for _ in 0..*size {
                let mut weights = vec![];

                if index != 0 {
                    let size = sizes[index - 1];
                    for _ in 0..size {
                        weights.push(rng.gen_range(-1.0..1.0))
                    }
                }

                nodes.push(Node { weights, bias: 0. })
            }

            layers.push(Layer { nodes });
        }

        NeuralNetwork { layers }
    }

    pub fn get_input_size(&self) -> usize {
        self.layers[0].nodes.len()
    }

    pub fn get_output_size(&self) -> usize {
        self.layers[self.layers.len() - 1].nodes.len()
    }

    pub fn run(&self, input: Vec<f32>) -> Vec<f32> {
        let mut current_values = input;

        for layer in self.layers.iter().skip(1) {
            current_values = layer
                .nodes
                .iter()
                .map(|node| {
                    let mut sum = 0.;

                    for (index, weight) in node.weights.iter().enumerate() {
                        sum += weight * current_values[index]
                    }

                    sum += node.bias;

                    //sigmoid
                    (1. / (1. + f32::exp(-sum))) * 2. - 1.
                })
                .collect();
        }

        current_values
    }

    pub fn mutate(&mut self, mutation_chance: f32, range: Range<f32>) {
        let mut rng = thread_rng();

        for layer in &mut self.layers {
            for node in &mut layer.nodes {
                if rng.gen::<f32>() < mutation_chance {
                    node.bias += rng.gen_range((range.start / 2.0)..(range.end / 2.0));
                }

                for index in 0..node.weights.len() {
                    if rng.gen::<f32>() < mutation_chance {
                        node.weights[index] += rng.gen_range(range.clone());
                    }
                }
            }
        }
    }
}
