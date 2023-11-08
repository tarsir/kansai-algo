use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::option::Option;

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
struct WordList {
    words: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Combinations {
    suffixes: Vec<String>,
    matching_prefixes: HashMap<u32, Vec<String>>,
}

impl Combinations {
    fn new() -> Self {
        Combinations {
            suffixes: vec![],
            matching_prefixes: HashMap::from([(1, vec![]), (2, vec![]), (3, vec![]), (4, vec![])]),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let file = File::open("./data/words2k.json")?;
    let reader = BufReader::new(file);
    let list_2k: WordList = serde_json::from_reader(reader)?;
    println!("Loaded {} words", list_2k.words.len());
    let words = list_2k.words.clone();
    let count = words
        .iter()
        .filter(|w| w.len() >= 3 && w.len() <= 6)
        .count();
    println!("{} words between 3-6 characters", count);
    let results = find_combinations(2, words);
    // println!("{}", serde_json::to_string_pretty(&results).unwrap());
    println!("{}", results.unwrap().len());

    Ok(())
}

fn find_combinations(suffix_len: usize, words: Vec<String>) -> Option<Vec<Combinations>> {
    let mut suffix_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut prefix_frequencies: HashMap<String, u32> = HashMap::new();

    for w in words.iter() {
        if w.len() < 3 || w.len() > 6 || w.as_bytes()[w.len() - 1] == b's' {
            continue;
        }
        let split_index = w.len() - suffix_len;
        let (prefix, suffix) = w.split_at(split_index);
        let prefix = prefix.to_string();
        let suffix = suffix.to_string();
        match suffix_map.get_mut(&suffix) {
            Some(prefixes) => prefixes.push(prefix.clone()),
            None => {
                suffix_map.insert(suffix, vec![prefix.clone()]);
            }
        }

        match prefix_frequencies.get_mut(&prefix) {
            Some(count) => *count += 1,
            None => {
                prefix_frequencies.insert(prefix, 1);
            }
        }
    }

    suffix_map.retain(|_k, v| v.len() >= 4);
    for (_, prefixes) in suffix_map.iter_mut() {
        prefixes.retain(|p| prefix_frequencies[p] > 1 && p.len() <= 3)
    }
    prefix_frequencies.retain(|_k, v| v > &mut 1);

    println!("{:?}", suffix_map.len());
    println!("{:?}", prefix_frequencies.len());

    Some(
        suffix_map
            .iter()
            .filter_map(|(s, _p)| {
                search_combinations(
                    &mut Combinations::new(),
                    s.clone(),
                    &suffix_map,
                    &prefix_frequencies,
                )
            })
            .flatten()
            .collect(),
    )
}

fn search_combinations(
    mut combinations: &mut Combinations,
    next_suffix: String,
    all_suffixes: &HashMap<String, Vec<String>>,
    prefix_frequencies: &HashMap<String, u32>,
) -> Option<Vec<Combinations>> {
    if combinations.suffixes.contains(&next_suffix.to_string()) {
        return None;
    }

    let next_prefixes = all_suffixes.get(&next_suffix).unwrap();
    combinations.suffixes.push(next_suffix);
    let mut buffers: [Vec<String>; 4] = [next_prefixes.to_vec(), vec![], vec![], vec![]];

    // get the prefixes that need to move up one spot and collect into buffers
    for (count, prefix_list) in combinations.matching_prefixes.iter_mut() {
        if count == &4 {
            continue;
        };
        let immut_prefix_list = prefix_list.clone();
        let (stays, shifts): (Vec<String>, Vec<String>) = immut_prefix_list
            .into_iter()
            .partition(|p| next_prefixes.contains(p));
        buffers[*count as usize].extend(shifts.into_iter());
        *prefix_list = stays;
    }

    // slam the buffers into the spot they need to go into
    for (count, prefix_list) in combinations.matching_prefixes.iter_mut() {
        if count == &0 {
            continue;
        };
        let buffer_contents = &mut buffers[*count as usize - 1];
        prefix_list.append(buffer_contents);
    }

    println!("{:?}", &combinations);
    match combinations.matching_prefixes.get(&4) {
        Some(vec) if !vec.is_empty() => Some(vec![combinations.clone()]),
        Some(_) => Some(
            all_suffixes
                .iter()
                .filter_map(|(s, _p)| {
                    search_combinations(combinations, s.clone(), all_suffixes, prefix_frequencies)
                })
                .flatten()
                .collect(),
        ),
        None => None,
    }
}
