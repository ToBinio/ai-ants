use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::ops::Not;

use crate::math::{mean, stddev};

mod math;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NeuralNetwork {
    inputs: usize,
    outputs: usize,

    pub nodes: Vec<Node>,
    pub connections: Vec<Connection>,

    #[serde(skip)]
    execution_order: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Connection {
    pub from: usize,
    pub to: usize,
    weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct SimpleConnection {
    from: usize,
    weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Node {
    bias: f32,
    activation_function: ActivationFunction,
    #[serde(skip)]
    connections: Vec<SimpleConnection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum ActivationFunction {
    Linear,
    Sigmoid,
    Relu,
}

impl NeuralNetwork {
    pub fn new(inputs: usize, outputs: usize) -> NeuralNetwork {
        let mut rng = thread_rng();
        NeuralNetwork::with_weight(inputs, outputs, || {
            rng.gen_range(-0.2..0.2) + rng.gen_range(-0.2..0.2)
        })
    }

    pub fn zero(inputs: usize, outputs: usize) -> NeuralNetwork {
        NeuralNetwork::with_weight(inputs, outputs, || 0.)
    }

    pub fn with_weight<F: FnMut() -> f32>(
        inputs: usize,
        outputs: usize,
        mut weight_fn: F,
    ) -> NeuralNetwork {
        let mut nodes = vec![];

        for _ in 0..(inputs + outputs) {
            nodes.push(Node {
                bias: 0.0,
                activation_function: ActivationFunction::Linear,
                connections: vec![],
            })
        }

        let mut connections = vec![];

        for input in 0..inputs {
            for output in inputs..(inputs + outputs) {
                connections.push(Connection {
                    from: input,
                    to: output,
                    weight: weight_fn(),
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

        network.build();

        network
    }

    pub fn get_input_size(&self) -> usize {
        self.inputs
    }

    pub fn get_output_size(&self) -> usize {
        self.outputs
    }

    pub fn run(&self, input: Vec<f32>) -> Vec<f32> {
        let mut node_vales = input;
        node_vales.resize(self.nodes.len(), 0.);

        for node_index in &self.execution_order[self.inputs..self.execution_order.len()] {
            let node = &self.nodes[*node_index];

            let mut sum = node
                .connections
                .iter()
                .map(|connection| node_vales[connection.from] * connection.weight)
                .sum::<f32>();

            sum += node.bias;

            let value = match node.activation_function {
                ActivationFunction::Linear => sum,
                ActivationFunction::Sigmoid => (1. / (1. + f32::exp(-sum))) * 2. - 1.,
                ActivationFunction::Relu => sum.max(0.),
            };

            node_vales[*node_index] = value;
        }

        let outputs = &node_vales[self.inputs..(self.inputs + self.outputs)];
        Vec::from(outputs)
    }

    pub fn randomize_weights(&mut self, weight_change_chance: f32, mutation_range: f32) {
        let mut rng = thread_rng();

        for node in &mut self.nodes {
            if rng.gen::<f32>() < weight_change_chance {
                node.bias += rng.gen_range((-mutation_range / 4.0)..(mutation_range / 4.0))
                    + rng.gen_range((-mutation_range / 4.0)..(mutation_range / 4.0));
            }
        }

        for connection in &mut self.connections {
            if rng.gen::<f32>() < weight_change_chance {
                connection.weight += rng.gen_range((-mutation_range / 2.0)..(mutation_range / 2.0))
                    + rng.gen_range((-mutation_range / 2.0)..(mutation_range / 2.0));
            }
        }

        self.build();
    }

    pub fn gradient_ascent(&mut self, learning_rate: f32, versions: Vec<(&NeuralNetwork, f32)>) {
        let rewards: Vec<_> = versions.iter().map(|(_, reward)| *reward).collect();

        let reward_mean = mean(&rewards);
        let mut reward_stddev = stddev(&rewards, reward_mean);

        if reward_stddev == 0. {
            reward_stddev = 1.;
        }

        //normalize reward
        let versions: Vec<_> = versions
            .into_iter()
            .map(|(network, reward)| (network, (reward as f32 - reward_mean) / reward_stddev))
            .collect();

        // ascent weights
        for connection_index in 0..self.connections.len() {
            let connection = &mut self.connections[connection_index];

            let mut gradient = 0.;

            for (network, reward) in &versions {
                let change = network.connections[connection_index].weight - connection.weight;
                gradient += change * reward;
            }

            connection.weight += learning_rate * (gradient / versions.len() as f32)
        }

        // ascent biases
        for node_index in 0..self.nodes.len() {
            let node = &mut self.nodes[node_index];

            let mut gradient = 0.;

            for (network, reward) in &versions {
                let change = network.nodes[node_index].bias - node.bias;

                gradient += change * reward;
            }

            node.bias += learning_rate * (gradient / versions.len() as f32);
        }

        self.build();
    }

    pub fn mutate_strucutre(&mut self) {
        let mut rng = thread_rng();
        let mutation_type: f32 = rng.gen();

        if mutation_type < 0.25 {
            //add new node between old connection
            if self.connections.is_empty() {
                return;
            }

            let new_node_index = self.nodes.len();
            self.nodes.push(Node {
                bias: 0.0,
                activation_function: ActivationFunction::Relu,
                connections: vec![],
            });

            let connection_index = rng.gen_range(0..self.connections.len());

            let old_to = self.connections[connection_index].to;
            self.connections[connection_index].to = new_node_index;

            self.connections.push(Connection {
                from: new_node_index,
                to: old_to,
                weight: rng.gen_range(-0.5..5.0) + rng.gen_range(-0.5..5.0),
            });

            self.build();
        } else {
            //add new connection
            let from_node = rng.gen_range(0..self.nodes.len());

            if (from_node + 1) == self.nodes.len() {
                return;
            }

            let to_node = rng.gen_range((from_node + 1)..self.nodes.len());

            let input_range = 0..self.inputs;
            let output_range = self.inputs..(self.outputs + self.inputs);

            let to_node = self.execution_order[to_node];
            let from_node = self.execution_order[from_node];

            //disallow existing connections
            if self
                .connections
                .iter()
                .any(|connection| connection.to == to_node && connection.from == from_node)
            {
                return;
            }

            if input_range.contains(&to_node) || output_range.contains(&from_node) {
                return;
            }

            self.connections.push(Connection {
                from: from_node,
                to: to_node,
                weight: rng.gen_range(-0.5..5.0) + rng.gen_range(-0.5..5.0),
            });

            self.build()
        }
    }

    pub fn build(&mut self) {
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
                panic!("something went wrong while building network");
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

        self.execution_order = execution_order;

        for (index, node) in self.nodes.iter_mut().enumerate() {
            let connections = self
                .connections
                .iter()
                .filter(|connection| connection.to == index)
                .map(|connection| SimpleConnection {
                    from: connection.from,
                    weight: connection.weight,
                })
                .collect();

            node.connections = connections;
        }
    }
}
