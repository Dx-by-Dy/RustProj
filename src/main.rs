use std::collections::{HashMap, LinkedList};
use std::collections::linked_list::Iter;
use std::time::{Instant};

#[derive(Debug)]
struct Graph {
    nodes: Vec<String>,
    neighbors: HashMap<String, LinkedList<String>>,
}

impl Graph {
    fn new() -> Self {
        Graph{ nodes: vec![], neighbors: HashMap::new()}
    }
    
    fn add_edge(&mut self, edge: (String, String)) {
        if self.neighbors.contains_key(&edge.0) {
            if !self.neighbors[&edge.0].contains(&edge.1) {
                self.neighbors.get_mut(&edge.0).unwrap().push_back(edge.1.clone());
            }
        }
        else {
            self.neighbors.insert(edge.0.clone(), LinkedList::new());
            self.neighbors.get_mut(&edge.0).unwrap().push_back(edge.1.clone());
            self.nodes.push(edge.0.clone());
        }
        
        if self.neighbors.contains_key(&edge.1) {
            if !self.neighbors[&edge.1].contains(&edge.0) {
                self.neighbors.get_mut(&edge.1).unwrap().push_back(edge.0.clone());
            }
        }
        else {
            self.neighbors.insert(edge.1.clone(), LinkedList::new());
            self.neighbors.get_mut(&edge.1).unwrap().push_back(edge.0.clone());
            self.nodes.push(edge.1.clone());
        }
    }
}

async fn get_graph(url: &str) -> Graph {
    let mut g = Graph::new();

    let body = reqwest::get(url)
        .await
        .expect("Failed to GET reqwest")
        .text()
        .await
        .unwrap()
        .to_string();

    let json_data: serde_json::Value = serde_json::from_str(&body)
        .expect("Can't parse json");

    for i in json_data["symbols"].as_array().unwrap() {
        g.add_edge((i["baseAsset"].as_str().unwrap().to_string(), i["quoteAsset"].as_str().unwrap().to_string()));
    }
    
    return g;
}

fn get_cycles(cycles: &mut HashMap<Vec<String>, Vec<String>>, start_iterators: &HashMap<String, Iter<String>>, graph: &Graph, len_of_cycles: usize){
    let mut used_nodes: HashMap<String, String> = HashMap::new();
    let mut line: Vec<String> = Vec::new();
    let mut cnters: HashMap<String, usize> = HashMap::new();
    let mut nd: String = String::new();
    let mut nn: String = String::new();
    let mut next_node: Option<&String> = None;
    let mut iterators: HashMap<String, Iter<String>> = HashMap::new();
    let mut way: Vec<String> = Vec::new();

    for start_node in graph.nodes.clone() {

        line.clear();
        line.push(start_node.clone());

        iterators.clear();
        for node in graph.nodes.clone() {
            iterators.insert(node.clone(), start_iterators[&node].clone());
        }

        cnters.clear();
        cnters.insert(start_node.clone(), 0);

        while line.len() != 0 {
            nd = line.last().unwrap().clone();

            loop {
                next_node = iterators.get_mut(&nd).unwrap().next();
                if next_node.is_none(){
                    line.pop();
                    cnters.remove(&nd);
                    break;
                }
                nn = next_node.unwrap().to_string();

                if !used_nodes.contains_key(&nn) {
                    if !cnters.contains_key(&nn) {
                        if line.len() < len_of_cycles {
                            cnters.insert(nn.clone(), line.len());
                            line.push(nn.clone());
                            iterators.insert(nn.clone(), start_iterators[&nn].clone());
                            break;
                        }
                    }

                    else {
                        way = line[cnters[&nn]..].to_vec();
                        way.sort();
                        if (!cycles.contains_key(&way)) && line.len() - cnters[&nn] >= len_of_cycles{
                            cycles.insert(way.clone(), way.clone());
                        }
                    }
                }
            }
        }

        used_nodes.insert(start_node.clone(), start_node.clone());
    }
}

#[tokio::main]
async fn main() {

    let url = "https://api.binance.com/api/v3/exchangeInfo?symbolStatus=TRADING&showPermissionSets=false";
    let grp = get_graph(url).await;
    
    let mut cycles: HashMap<Vec<String>, Vec<String>> = HashMap::new();
    let mut start_iterators: HashMap<String, Iter<String>> = HashMap::new();

    for node in grp.nodes.clone() {
        start_iterators.insert(node.clone(), grp.neighbors[&node].iter());
    }

    let start = Instant::now();
    
    get_cycles(&mut cycles, &start_iterators, &grp, 3);
    get_cycles(&mut cycles, &start_iterators, &grp, 4);
    //get_cycles(&mut cycles, &start_iterators, &grp, 5);
    //get_cycles(&mut cycles, &start_iterators, &grp, 6);
    
    println!("{}", cycles.len());
    println!("{:?}", start.elapsed());
}
