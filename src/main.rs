use std::cmp;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::time::Instant;

use petgraph;
use petgraph::dot::{Config, Dot};
use petgraph::graph::NodeIndex;
use petgraph::Direction;

fn main() -> Result<(), std::io::Error> {
    let start = Instant::now();
    let mut found_cycles: Vec<Box<Vec<[char; 2]>>> = Vec::new();
    let mut input_file = File::open("./input-cities-list.txt")?;
    let mut input_string = String::new();
    input_file.read_to_string(&mut input_string)?;
    let mut city_names: Vec<&str> = input_string.split("\n").collect();
    city_names.pop(); // pops last empty value
    city_names.sort();
    let short_names: Vec<(char, char)> = city_names
        .clone()
        .into_iter()
        .map(|x| {
            (
                x.to_lowercase().chars().next().unwrap(),
                x.chars().last().unwrap(),
            )
        })
        .collect();
    let self_cycles: Vec<Vec<[char; 2]>> = short_names
        .clone()
        .into_iter()
        .filter(|x| x.0 == x.1)
        .map(|x| vec![[x.0, x.1]])
        .collect();
    for self_path in self_cycles {
        found_cycles.push(Box::new(self_path));
    }
    let pairs_wo_cycles: Vec<(char, char)> = short_names
        .clone()
        .into_iter()
        .filter(|x| x.0 != x.1)
        .collect();
    let mut pair_freq = HashMap::new();
    for pair in pairs_wo_cycles.clone().into_iter() {
        let mut temp = pair.0.to_string();
        temp.push(pair.1);
        *pair_freq.entry(temp).or_insert(0) += 1;
    }
    let mut two_pairs_cycles = HashMap::new();
    for pair in pairs_wo_cycles.clone().into_iter() {
        let mut str_a = pair.0.to_string();
        str_a.push(pair.1);
        let mut str_b = pair.1.to_string();
        str_b.push(pair.0);
        let str_ab = [&*str_a, &*str_b].concat();
        let str_ba = [&*str_b, &*str_a].concat();
        if !(two_pairs_cycles.contains_key(&*str_ba) ^ two_pairs_cycles.contains_key(&*str_ab)) {
            //pair_already_in hashmap
            let a_count = pair_freq.get(&*str_a).unwrap_or(&0);
            let b_count = pair_freq.get(&*str_b).unwrap_or(&0);
            let str_ab = str_a + &str_b;
            let min = cmp::min(a_count, b_count);
            two_pairs_cycles.insert(
                str_ab,
                (a_count, b_count, min, a_count - min, b_count - min),
            );
        }
    }
    let mut pairs_cleaned_from_2_cycles = Vec::new();
    for (key, value) in two_pairs_cycles.into_iter() {
        let temp: Vec<char> = key.clone().chars().collect();
        for _ in 0..*value.2 {
            found_cycles.push(Box::new(vec![[temp[0], temp[1]], [temp[2], temp[3]]]));
        }
        let temp: Vec<char> = key.clone().chars().collect();
        let a_lasted_num = value.3;
        for _ in 0..a_lasted_num {
            pairs_cleaned_from_2_cycles.push(Box::new((temp[0], temp[1])));
        }
        let b_lasted_num = value.4;
        for _ in 0..b_lasted_num {
            pairs_cleaned_from_2_cycles.push(Box::new((temp[2], temp[3])));
        }
    }

    let mut nodes: Vec<char> = pairs_cleaned_from_2_cycles
        .clone()
        .into_iter()
        .map(|x| x.0)
        .collect();
    nodes.sort();
    nodes.dedup();
    //let mut graph = petgraph::graphmap::DiGraphMap::<char, i32>::new(); //<char, i32>::new();
    let mut graph = petgraph::Graph::<char, i32>::new(); //<char, i32>::new();
    let mut graph_nodes = HashMap::new();
    for node in nodes {
        let temp = graph.add_node(node);
        graph_nodes.insert(node, temp);
    }
    let mut edge_freq = HashMap::new();
    for pair in pairs_cleaned_from_2_cycles.clone().into_iter() {
        let mut temp = pair.0.to_string();
        temp.push(pair.1);
        *edge_freq.entry(temp).or_insert(0) += 1;
    }
    for (key, value) in edge_freq.into_iter() {
        let temp: Vec<char> = key.clone().chars().collect();
        let start = temp[0];
        let end = temp[1];
        //graph.add_edge(start, end, value);
        graph.add_edge(graph_nodes[&start], graph_nodes[&end], value);
    }
    //dbg!(&graph);
    let mut found_any = true;
    let mut search_depth = 3;
    let depth_limit = 10;
    while search_depth <= depth_limit {
        if found_any == false {
            search_depth += 1;
        }
        found_any = false;
        for (_, node_index) in graph_nodes.clone().into_iter() {
            let mut path = Vec::new();
            breadth_search(node_index, node_index, &graph, &mut path, search_depth);
            if path != [] {
                found_any = true;
                path.reverse();
                let mut weights = Vec::new();
                let mut short_path_naming = Vec::new();
                for start in 0..path.len() {
                    let mut end = start + 1;
                    if end == path.len() {
                        end = 0;
                    }
                    weights.push(graph[graph.find_edge(path[start], path[end]).unwrap()]);
                    short_path_naming.push([graph[path[start]], graph[path[end]]]);
                }
                let path_minimum = weights.iter().min().unwrap();
                for start in 0..path.len() {
                    let mut end = start + 1;
                    if end == path.len() {
                        end = 0;
                    }
                    let edge_index = graph.find_edge(path[start], path[end]).unwrap();
                    let edge_weight = graph.edge_weight_mut(edge_index).unwrap();
                    *edge_weight = *edge_weight - path_minimum;
                    if *edge_weight == 0 {
                        graph.remove_edge(edge_index);
                    }
                }
                for _ in 0..*path_minimum {
                    found_cycles.push(Box::new(short_path_naming.clone()));
                }
            }
        }
    }

    let mut found_paths = Vec::new();
    for (_, node_index) in graph_nodes.clone().into_iter() {
        found_paths.extend(depth_first_serach(
            node_index,
            &graph,
            &mut Vec::new(),
            &mut Vec::new(),
        ));
    }
    let longest_base_path = found_paths.iter().max_by_key(|x| x.len()).unwrap();
    let mut result = Vec::new();
    for start in 0..longest_base_path.len() - 1 {
        let end = start + 1;
        result.push([
            graph[longest_base_path[start]],
            graph[longest_base_path[end]],
        ]);
    }
    let mut notfound_counter = 0;
    let mut previous_found_cycles_len = found_cycles.len();
    while found_cycles.len() > 0 {
        for (cycle_pos, cycle) in found_cycles.iter().enumerate() {
            let cycle_letter = cycle[0][0];
            let mut found = false;
            for (position, word) in result.iter().enumerate() {
                if word[1] == cycle_letter {
                    //found insert place
                    result.splice(position + 1..position + 1, cycle.iter().cloned());
                    found = true;
                    break;
                }
            }
            if found {
                found_cycles.remove(cycle_pos);
                break;
            };
        }
        if found_cycles.len() == previous_found_cycles_len {
            notfound_counter += 1;
        }
        previous_found_cycles_len = found_cycles.len();
        if notfound_counter > 100 {
            println!("Couldn\'t colapse all cycles, recommend to restart because result probably isn\'t the best");
            println!("Cycles on merged: {:?}", found_cycles.len());
            break;
        }
    }
    city_names.sort_by(|a, b| a.len().cmp(&b.len()));
    let mut result_string = String::new();
    for pair in &result {
        for (city_num, city) in city_names.iter().enumerate() {
            let city_chars: Vec<char> = city.to_lowercase().chars().collect();
            if city_chars[0] == pair[0] && city_chars[city.len() - 1] == pair[1] {
                result_string.push_str(city);
                city_names.remove(city_num);
                break;
            }
        }
    }
    let mut output_file = File::create("resulting_string.txt")?;
    output_file.write(result_string.as_bytes())?;

    println!(
        "Time elapsed: {:?} milliseconds",
        start.elapsed().as_millis()
    );
    println!("Total path len: {:?}", result.len());
    println!("Total string len: {:?}", result_string.len());
    Ok(())
}

fn breadth_search(
    searched_index: NodeIndex,
    current_node: NodeIndex,
    graph: &petgraph::graph::Graph<char, i32>,
    path: &mut Vec<NodeIndex>,
    max_depth: i32,
) -> Option<bool> {
    if max_depth == 0 {
        return None;
    }
    for edge in graph.neighbors_directed(current_node, Direction::Outgoing) {
        if edge == searched_index {
            path.push(edge);
            return Some(true);
        }
        let search_result = breadth_search(searched_index, edge, graph, path, max_depth - 1);
        match search_result {
            Some(_) => {
                path.push(edge);
                return Some(true);
            }
            _ => {}
        }
    }
    return None;
}

fn depth_first_serach(
    current_node: NodeIndex,
    graph: &petgraph::graph::Graph<char, i32>,
    path: &mut Vec<NodeIndex>,
    seen: &mut Vec<NodeIndex>,
) -> Vec<Vec<NodeIndex>> {
    if path.is_empty() {
        path.push(current_node);
    }
    seen.push(current_node);

    let mut paths = Vec::new();
    for t in graph.neighbors_directed(current_node, Direction::Outgoing) {
        if !seen.contains(&t) {
            let mut t_path = path.clone();
            t_path.push(t);
            paths.push(t_path.clone());
            paths.extend(depth_first_serach(t, graph, &mut t_path, seen));
        }
    }
    return paths;
}
