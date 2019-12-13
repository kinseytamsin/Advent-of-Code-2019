extern crate nom;
extern crate petgraph;

use std::collections::{
    hash_map::Entry::{Occupied, Vacant},
    HashMap
};
use std::io::BufReader;
use std::io::prelude::*;
use std::fs::File;

use nom::{
    IResult,
    bytes::complete::is_not,
    sequence::separated_pair
};
use nom::character::complete::char as nom_char;
use petgraph::{
    algo::astar,
    graph::DiGraph,
    visit::{
        Dfs,
        VisitMap,
        GraphRef,
        Visitable,
        IntoNeighbors
    },
    Undirected
};

macro_rules! get_or_insert_obj {
    ($graph:expr, $nodes_map:expr, $obj_str:expr, $obj_node:ident) => {
        match $nodes_map.entry($obj_str) {
            Vacant(e) => {
                $obj_node = $graph.add_node(());
                e.insert($obj_node);
            }
            Occupied(e) => {
                $obj_node = *(e.get());
            }
        }
    }
}

// objects are represented by unique identifying character sequences
type Object = String;

struct Orbit {
    object: Object,
    target: Object
}

fn orbit_spec(input: &str) -> IResult<&str, Orbit> {
    let (input, (target, object)) = separated_pair(
        is_not(")"),
        nom_char(')'),
        is_not("\n")
    )(input)?;
    let object = object.to_string();
    let target = target.to_string();

    Ok((input, Orbit { object, target }))
}

type Ix = u32;
type NodeIndex = petgraph::graph::NodeIndex<Ix>;
type OrbitGraph = DiGraph<(), u32, Ix>;
type NodesMap = HashMap<String, NodeIndex>;

fn get_orbit_target<VM, G>(graph: G, node: NodeIndex) -> Option<NodeIndex>
where VM: VisitMap<NodeIndex>,
      G: GraphRef + Visitable<NodeId = NodeIndex, Map = VM> + IntoNeighbors
{
    let mut dfs = Dfs::new(graph, node);
    // visit self, pushing neighbor to top of search stack
    dfs.next(graph);

    dfs.next(graph)
}

fn main() -> std::io::Result<()> {
    let f = File::open("6.txt")?;
    let buf_reader = BufReader::new(f);
    let lines_iter = buf_reader.lines();
    let mut graph = OrbitGraph::new();
    let mut nodes_map = NodesMap::new();

    for line in lines_iter {
        let line: String = line?;
        let orbit: Orbit = match orbit_spec(&line).unwrap() { (_, x) => x };
        let Orbit { object, target } = orbit;
        let object_node: NodeIndex;
        let target_node: NodeIndex;
        get_or_insert_obj!(graph, nodes_map, object, object_node);
        get_or_insert_obj!(graph, nodes_map, target, target_node);
        graph.add_edge(object_node, target_node, 1);
    }

    let mut orbit_count = 0;

    // count direct and indirect orbits
    for node in graph.node_indices() {
        let mut dfs = Dfs::new(&graph, node);
        // don't count self
        dfs.next(&graph);
        while let Some(_) = dfs.next(&graph) {
            orbit_count += 1;
        }
    }

    println!("Total orbits: {}", orbit_count);

    let you_node: NodeIndex = *(nodes_map.get("YOU").unwrap());
    let san_node: NodeIndex = *(nodes_map.get("SAN").unwrap());
    let source_node: NodeIndex = get_orbit_target(&graph, you_node).unwrap();
    let destination_node: NodeIndex = get_orbit_target(&graph, san_node)
                                      .unwrap();
    let orbital_transfers: u32;
    if source_node == destination_node {
        orbital_transfers = 0;
    } else {
        // we don't care about what is orbiting what anymore, we just
        // want to find the shortest path between nodes, so discard
        // direction information
        let graph_undir = graph.into_edge_type::<Undirected>();
        let path = astar(&graph_undir,
                         source_node,
                         |f| f == destination_node,
                         |e| *e.weight(),
                         |_| 0);
        if let Some((k, _)) = path {
            orbital_transfers = k;
        } else {
            panic!("Unable to find shortest path");
        }
    }
    println!("Least number of orbital transfers: {}", orbital_transfers);

    Ok(())
}
