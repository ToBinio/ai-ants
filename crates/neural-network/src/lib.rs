use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

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
    weights: Vec<f64>,
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

                nodes.push(Node { weights })
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

    pub fn run(&self, input: Vec<f64>) -> Vec<f64> {
        let mut current_values = input;
        let mut next_values = vec![];

        for layer in self.layers.iter().skip(1) {
            for node in &layer.nodes {
                let mut val = 0.;

                for (index, weight) in node.weights.iter().enumerate() {
                    val += current_values[index] * weight;
                }

                //todo activation function
                next_values.push(val);
            }

            current_values = next_values;
            next_values = vec![];
        }

        current_values
    }
}
