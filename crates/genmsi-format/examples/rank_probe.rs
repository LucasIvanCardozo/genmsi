//! Probe: what ranks does dagre give to the chains of a model?

use dagre::graph::{Graph, GraphOptions};
use dagre::{EdgeLabel, LayoutOptions, NodeLabel, RankDir, Ranker};

fn main() {
    let mut graph = Graph::with_options(GraphOptions {
        directed: true,
        multigraph: true,
        compound: true,
    });
    for id in 19..=21 {
        graph.set_node(id.to_string(), Some(NodeLabel { width: 120.0, height: 48.0, ..NodeLabel::default() }));
    }
    for id in 22..=27 {
        graph.set_node(id.to_string(), Some(NodeLabel { width: 120.0, height: 48.0, ..NodeLabel::default() }));
    }
    for id in 38..=40 {
        graph.set_node(id.to_string(), Some(NodeLabel { width: 120.0, height: 48.0, ..NodeLabel::default() }));
    }
    graph.set_edge("19", "20", Some(EdgeLabel::default()), None);
    graph.set_edge("20", "21", Some(EdgeLabel::default()), None);
    for (a, b) in [("22", "23"), ("23", "24"), ("24", "25"), ("25", "26"), ("26", "27")] {
        graph.set_edge(a, b, Some(EdgeLabel::default()), None);
    }
    graph.set_edge("38", "39", Some(EdgeLabel::default()), None);
    graph.set_edge("39", "40", Some(EdgeLabel::default()), None);

    let options = LayoutOptions {
        rankdir: RankDir::TB,
        ranker: Ranker::LongestPath,
        nodesep: 30.0,
        ranksep: 60.0,
        edgesep: 20.0,
        ..LayoutOptions::default()
    };
    dagre::layout(&mut graph, Some(options));
    for id in ["19", "22", "38", "21", "27", "40"] {
        let node = graph.node(id).expect("node");
        println!("{id}: rank={:?} x={:?} y={:?}", node.rank, node.x, node.y);
    }
}
