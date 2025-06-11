use clap::Parser;
use neural_network::NeuralNetwork;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use svg::node::element::path::Data;
use svg::node::element::{Circle, Path};
use svg::Document;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[arg(short, long)]
    path: String,
}

fn main() {
    let cli = Cli::parse();

    let file = File::open(cli.path).unwrap();
    let reader = BufReader::new(file);
    let neural_network: NeuralNetwork = serde_json::from_reader(reader).unwrap();

    let node_count = neural_network.nodes.len();

    let mut depths = vec![None; node_count];

    let mut current_depth = 0;

    while depths.iter().any(|node| node.is_none()) {
        let to_be_resolved: Vec<usize> = depths
            .iter()
            .enumerate()
            .filter(|(_, node)| node.is_none())
            .filter(|(index, _)| {
                for connection in &neural_network.connections {
                    if &connection.to == index {
                        for (index, value) in depths.iter().enumerate() {
                            if value.is_some() {
                                continue;
                            }

                            if connection.from == index {
                                return false;
                            }
                        }
                    }
                }

                true
            })
            .map(|(index, _)| index)
            .collect();

        for index in to_be_resolved {
            depths[index] = Some(current_depth)
        }

        current_depth += 1;
    }

    //set output always to last layer
    for depth in depths
        .iter_mut()
        .skip(neural_network.get_input_size())
        .take(neural_network.get_output_size())
    {
        *depth = Some(current_depth - 1);
    }

    let mut node_position = vec![];
    let mut depth_counts: HashMap<usize, usize> = HashMap::new();

    for depth in depths.iter() {
        let depth = depth.unwrap();
        let depth_count = depth_counts.get(&depth).unwrap_or(&0);

        node_position.push(((depth + 1) * 150, (depth_count + 1) * 75));

        depth_counts.insert(depth, depth_count + 1);
    }

    let mut svg = Document::new();

    for connection in neural_network.connections {
        let data = Data::new()
            .move_to(node_position[connection.from])
            .line_to(node_position[connection.to])
            .close();

        let path = Path::new()
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width", 3)
            .set("d", data);

        svg = svg.add(path)
    }

    for (x, y) in node_position {
        let circle = Circle::new().set("cx", x).set("cy", y).set("r", 25);
        svg = svg.add(circle);
    }

    svg = svg.set(
        "viewBox",
        (0, 0, (node_count + 1) * 150, (node_count + 1) * 75),
    );

    svg::save("test.svg", &svg).unwrap()
}
