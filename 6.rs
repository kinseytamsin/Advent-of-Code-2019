#[macro_use]
extern crate anyhow;
extern crate nom;
extern crate petgraph;

use std::collections::{
    hash_map::Entry::{Occupied, Vacant},
    HashMap
};
use std::io::BufReader;
use std::io::prelude::*;
use std::fs::File;

use anyhow::Result;
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
    ($graph:expr, $nodes_map:expr, $obj_str:expr) => {
        match $nodes_map.entry($obj_str) {
            Vacant(e) => {
                let new_node = $graph.add_node(());
                e.insert(new_node);
                new_node
            }
            Occupied(e) => {
                *(e.get())
            }
        }
    }
}

// objects are represented by unique identifying character sequences
type Object = String;

struct Orbit {
    object: Object,
    target: Object,
}

fn orbit_spec(input: &str) -> IResult<&str, Orbit> {
    let parser =
        separated_pair(
            is_not(")"),
            nom_char(')'),
            is_not("\n"),
        );
    let (unparsed, (target, object)) = parser(input)?;

    Ok((unparsed, Orbit { object.to_string(), target.to_string() }))
}

type Ix = u32;
type NodeIndex = petgraph::graph::NodeIndex<Ix>;
type OrbitGraph = DiGraph<(), u32, Ix>;
type NodesMap = HashMap<String, NodeIndex>;

fn get_orbit_target<VM, G>(graph: G, node: NodeIndex) -> Option<NodeIndex>
where
    VM: VisitMap<NodeIndex>,
    G:  GraphRef<NodeId = NodeIndex> + Visitable<Map = VM> + IntoNeighbors
{
    let mut dfs = Dfs::new(graph, node);
    // visit self, pushing neighbor to top of search stack
    dfs.next(graph);

    dfs.next(graph)
}

fn main() -> Result<()> {
    let f = File::open("6.txt")?;
    let buf_reader = BufReader::new(f);
    let lines = buf_reader.lines();
    let mut graph = OrbitGraph::new();
    let mut nodes_map = NodesMap::new();

    for line in lines {
        let orbit = match orbit_spec(&line?) {
            Ok((unparsed, x)) => {
                if unparsed != "" {
                    Err(anyhow!("Unparsed data in line: {}", unparsed))
                } else {
                    Ok(x)
                }
            },
            Err(e) => Err(anyhow!("{:?}", e)),
        };
        let Orbit { object, target } = orbit?;
        let object_node = get_or_insert_obj!(graph, nodes_map, object);
        let target_node = get_or_insert_obj!(graph, nodes_map, target);
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

    let you_node =
        *(
            nodes_map
                .get("YOU")
                .ok_or_else(|| anyhow!("Failed to find YOU node"))?
        );
    let san_node =
        *(
            nodes_map
                .get("SAN")
                .ok_or_else(|| anyhow!("Failed to find SAN node"))?
        );
    let source_node =
        get_orbit_target(&graph, you_node)
            .ok_or_else(|| anyhow!("Failed to find orbit target of YOU"))?;
    let destination_node =
        get_orbit_target(&graph, san_node)
            .ok_or_else(|| anyhow!("Failed to find orbit target of SAN"))?;
    let orbital_transfers =
        if source_node == destination_node {
            Ok(0)
        } else {
            // we don't care about what is orbiting what anymore, we
            // just want to find the shortest path between nodes, so
            // discard direction information
            let graph_undir = graph.into_edge_type::<Undirected>();
            match astar(
                &graph_undir,
                source_node,
                |finish| finish == destination_node,
                |e| *e.weight(),
                |_| 0,
            ) {
                Some((k, _)) => Ok(k),
                None         => Err(anyhow!("Unable to find shortest path")),
            }
        };
    println!("Least number of orbital transfers: {}", orbital_transfers?);

    Ok(())
}
