use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::ops::{Not, Range};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralNetwork {
    inputs: usize,
    outputs: usize,

    nodes: Vec<Node>,
    connections: Vec<Connection>,

    execution_order: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Connection {
    from: usize,
    to: usize,
    weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Node {
    bias: f32,
    activation_function: ActivationFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum ActivationFunction {
    LINEAR,
    SIGMOID,
}

impl NeuralNetwork {
    pub fn new(inputs: usize, outputs: usize) -> NeuralNetwork {
        let mut nodes = vec![];

        for _ in 0..(inputs + outputs) {
            nodes.push(Node {
                bias: 0.0,
                activation_function: ActivationFunction::LINEAR,
            })
        }

        let mut connections = vec![];

        for input in 0..inputs {
            for output in inputs..(inputs + outputs) {
                connections.push(Connection {
                    from: input,
                    to: output,
                    weight: 0.0,
                })
            }
        }

        let mut network = NeuralNetwork {
            inputs,
            outputs,
            nodes,
            connections,
            execution_order: vec![],
        };

        let execution_order = network.build().unwrap();
        network.execution_order = execution_order;

        network
    }

    pub fn get_input_size(&self) -> usize {
        self.inputs
    }

    pub fn get_output_size(&self) -> usize {
        self.outputs
    }

    pub fn run(&self, input: Vec<f32>) -> Vec<f32> {
        let input_range = 0..self.inputs;
        let mut node_vales = vec![];

        for _ in &self.nodes {
            node_vales.push(0.);
        }

        for node_index in &self.execution_order {
            let node = &self.nodes[*node_index];

            let mut sum: f32 = self
                .connections
                .iter()
                .filter(|connection| connection.to != *node_index)
                .map(|connection| node_vales[connection.from] * connection.weight)
                .sum();

            if input_range.contains(node_index) {
                sum = input[*node_index];
            }

            sum += node.bias;

            let value = match node.activation_function {
                ActivationFunction::LINEAR => sum,
                ActivationFunction::SIGMOID => (1. / (1. + f32::exp(-sum))) * 2. - 1.,
            };

            node_vales[*node_index] = value;
        }

        let mut outputs = vec![];
        for output_index in self.inputs..(self.inputs + self.outputs) {
            outputs.push(node_vales[output_index])
        }

        outputs
    }

    pub fn mutate(&mut self) {
        const MUTATION_CHANCE: f32 = 0.2;
        const MUTATION_RANGE: Range<f32> = -0.5..0.5;

        let mut rng = thread_rng();
        let mutation_type: f32 = rng.gen();

        if mutation_type < 0.1 {
            //add new node
            if self.connections.is_empty() {
                return;
            }

            let new_node_index = self.nodes.len();
            self.nodes.push(Node {
                bias: 0.0,
                activation_function: ActivationFunction::SIGMOID,
            });

            let connection_index = rng.gen_range(0..self.connections.len());

            let old_to = self.connections[connection_index].to;
            self.connections[connection_index].to = new_node_index;

            self.connections.push(Connection {
                from: new_node_index,
                to: old_to,
                weight: 1.0,
            });

            self.execution_order = self.build().unwrap();
        } else if mutation_type < 0.3 {
            //add new connection

            let from_node = rng.gen_range(0..self.nodes.len());

            if (from_node + 1) == self.nodes.len() {
                return;
            }

            let to_node = rng.gen_range((from_node + 1)..self.nodes.len());

            let input_range = 0..self.inputs;
            let output_range = self.inputs..(self.outputs + self.inputs);

            //todo disallow existing connections

            if input_range.contains(&to_node) || output_range.contains(&from_node) {
                return;
            }

            self.connections.push(Connection {
                from: self.execution_order[from_node],
                to: self.execution_order[to_node],
                weight: rng.gen_range(MUTATION_RANGE.clone()),
            });

            self.execution_order = self.build().unwrap();
        } else if mutation_type < 0.8 {
            //change values

            for node in &mut self.nodes {
                if rng.gen::<f32>() < MUTATION_CHANCE {
                    node.bias +=
                        rng.gen_range((MUTATION_RANGE.start / 2.0)..(MUTATION_RANGE.end / 2.0));
                }
            }

            for connection in &mut self.connections {
                if rng.gen::<f32>() < MUTATION_CHANCE {
                    connection.weight += rng.gen_range(MUTATION_RANGE.clone());
                }
            }
        }
    }

    fn build(&self) -> Option<Vec<usize>> {
        let mut execution_order = vec![];
        let mut open_nodes: Vec<usize> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(index, _)| index)
            .collect();

        while open_nodes.is_empty().not() {
            let nodes_to_close: Vec<usize> = open_nodes
                .iter()
                .cloned()
                .filter(|node_index| {
                    let count = self
                        .connections
                        .iter()
                        //only nodes that go "to" the current one
                        .filter(|connection| connection.to == *node_index)
                        //only nodes where the "from" is already resolved
                        .filter(|connection| open_nodes.contains(&connection.from))
                        .count();
                    count == 0
                })
                .collect();

            if nodes_to_close.is_empty() {
                return None;
            }

            for node_index in nodes_to_close {
                execution_order.push(node_index);
                let open_node_index = open_nodes
                    .iter()
                    .position(|index| index == &node_index)
                    .unwrap();

                open_nodes.remove(open_node_index);
            }
        }

        Some(execution_order)
    }
}
