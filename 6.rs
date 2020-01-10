use std::{
    collections::{
        hash_map::Entry::{Occupied, Vacant},
        HashMap
    },
    error::Error as StdError,
    marker::Unpin,
    pin::Pin,
    sync::Arc,
};
use anyhow::{anyhow, Error, Result};
use broadcaster::BroadcastChannel;
use futures::{
    prelude::*,
    join,
    channel::mpsc,
    lock::Mutex,
    stream::Stream,
    sink::Sink,
};
use nom::{
    IResult,
    bytes::complete::is_not,
    sequence::separated_pair,
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
        IntoNeighbors,
    },
    Undirected,
};
use tokio::{
    prelude::*,
    io::BufReader,
    fs::File,
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
type Ix = u32;
type NodeIndex = petgraph::graph::NodeIndex<Ix>;
type OrbitGraph = DiGraph<(), u32, Ix>;
type NodesMap = HashMap<String, NodeIndex>;

#[derive(Clone)]
struct Orbit {
    object: Object,
    target: Object,
}

#[derive(Clone)]
struct OrbitGraphMap(OrbitGraph, NodesMap);

fn orbit_spec(input: &str) -> IResult<&str, Orbit> {
    let parser =
        separated_pair(
            is_not(")"),
            nom_char(')'),
            is_not("\n"),
        );
    let (unparsed, (target, object)) = parser(input)?;
    let object = object.to_owned();
    let target = target.to_owned();

    Ok((unparsed, Orbit { object, target }))
}


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

async fn build_graph<Rx, Tx>(
    mut rx_orbits: Rx,
    mut tx_graph: Tx,
) -> Result<()>
where
    Rx: Stream<Item = Result<Orbit>> + Unpin,
    Tx: Sink<OrbitGraphMap> + Unpin,
    <Tx as Sink<OrbitGraphMap>>::Error: StdError + Send + Sync + 'static,
{
    let orbit_graph_map =
        Arc::new(
            Mutex::new(OrbitGraphMap(OrbitGraph::new(), NodesMap::new()))
        );
    while let Some(orbit) = rx_orbits.next().await {
        let graph_mtx = Arc::clone(&orbit_graph_map);
        let graph_mtx2 = Arc::clone(&orbit_graph_map);
        let orbit_chan = BroadcastChannel::new();
        let orbit_chan2 = orbit_chan.clone();
        let (mut tx_orbit, mut rx_orbit) = orbit_chan.split();
        let (_, mut rx_orbit2) = orbit_chan2.split();
        let get_object_node_fut = async {
            let (mut graph_map, orbit) =
                join!(
                    graph_mtx.lock(),
                    rx_orbit.next().map(|x: Option<Orbit>| x.unwrap()),
                );
            let OrbitGraphMap(ref mut graph, ref mut nodes_map) = *graph_map;
            let Orbit { ref object, .. } = orbit;
            get_or_insert_obj!(graph, nodes_map, object.to_owned())
        };
        let get_target_node_fut = async {
            let (mut graph_map, orbit) =
                join!(
                    graph_mtx2.lock(),
                    rx_orbit2.next().map(|x: Option<Orbit>| x.unwrap()),
                );
            let OrbitGraphMap(ref mut graph, ref mut nodes_map) = *graph_map;
            let Orbit { ref target, .. } = orbit;
            get_or_insert_obj!(graph, nodes_map, target.to_owned())
        };
        let (_, object_node, target_node) =
            future::try_join3(
                async {
                    tx_orbit.send(orbit?).await?;
                    tx_orbit.close().await?;
                    Ok::<_, Error>(())
                },
                get_object_node_fut.map(Ok),
                get_target_node_fut.map(Ok),
            ).await?;
        let OrbitGraphMap(ref mut graph, _) = *orbit_graph_map.lock().await;
        graph.add_edge(object_node, target_node, 1);
    }
    tx_graph.send(orbit_graph_map.lock().await.clone()).await?;
    tx_graph.close().await?;
    Ok(())
}

async fn count_orbits<Rx>(mut rx_graph: Rx) -> u32
where
    Rx: Stream<Item = OrbitGraphMap> + Unpin
{
    let OrbitGraphMap(graph, _) = rx_graph.next().await.unwrap();
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
    orbit_count
}

async fn calculate_orbital_transfers<Rx>(mut rx_graph: Rx) -> Result<u32>
where
    Rx: Stream<Item = OrbitGraphMap> + Unpin
{
    let OrbitGraphMap(graph, nodes_map) = rx_graph.next().await.unwrap();
    let you_node =
        *(
            nodes_map.get("YOU")
                .ok_or_else(|| anyhow!("Failed to find YOU node"))?
        );
    let san_node =
        *(
            nodes_map.get("SAN")
                .ok_or_else(|| anyhow!("Failed to find SAN node"))?
        );
    let source_node =
        get_orbit_target(&graph, you_node)
            .ok_or_else(|| anyhow!("Failed to find orbit target of YOU"))?;
    let destination_node =
        get_orbit_target(&graph, san_node)
            .ok_or_else(|| anyhow!("Failed to find orbit target of SAN"))?;

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
    }
}

#[tokio::main(threaded_scheduler)]
async fn main() -> Result<()> {
    let f = File::open("6.txt").await?;
    let buf_reader = BufReader::new(f);
    let lines = buf_reader.lines();
    let orbits =
        lines
            .err_into::<Error>()
            .map_ok(|line| {
                let (unparsed, orbit) =
                    orbit_spec(&line).map_err(|e| anyhow!("{:?}", e))?;
                if !unparsed.is_empty() {
                    Err(anyhow!("Unparsed data in line: {}", unparsed))
                } else {
                    Ok(orbit)
                }
            })
            .map(Ok);
    let (mut tx_orbits, rx_orbits) = mpsc::channel(0);
    let graph_chan = BroadcastChannel::new();
    let graph_chan2 = graph_chan.clone();
    let (tx_graph, rx_graph) = graph_chan.split();
    let (_, rx_graph2) = graph_chan2.split();

    future::try_join(
        async {
            orbits
                .forward(Pin::new(&mut tx_orbits))
                .err_into::<Error>()
                .await?;
            tx_orbits.close().await?;
            Ok::<_, Error>(())
        },
        build_graph(
            rx_orbits
                // the return type is wrapped in a
                // Result<_, mpsc::SendError>, which should cause
                // .forward() to return an error if one occurs, so the
                // value from the Receiver should be safe to .unwrap()
                .map(|x| x.unwrap()),
            tx_graph
        ),
    ).await?;
    let (orbit_count, orbital_transfers) =
        future::try_join(
            count_orbits(rx_graph).map(Ok),
            calculate_orbital_transfers(rx_graph2),
        ).await?;
    println!("Total orbits: {}", orbit_count);
    println!("Least number of orbital transfers: {}", orbital_transfers);

    Ok(())
}
