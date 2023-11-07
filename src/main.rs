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

struct Combinations {
    suffixes: Vec<String>,
    matching_prefixes: HashMap<u32, Vec<String>>,
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
    find_combinations(2, words);

    Ok(())
}

fn find_combinations(suffix_len: usize, words: Vec<String>) -> Option<Combinations> {
    let mut suffix_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut prefix_frequencies: HashMap<String, u32> = HashMap::new();

    for w in words.iter() {
        if w.len() < 3 || w.len() > 6 {
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
        prefixes.retain(|p| prefix_frequencies[p] > 1)
    }
    prefix_frequencies.retain(|_k, v| v > &mut 1);

    println!("{:?}", suffix_map.len());
    println!("{:?}", prefix_frequencies.len());
    None
}

fn search_combinations(
    mut combinations: Combinations,
    next_suffix: String,
    all_suffixes: HashMap<String, Vec<String>>,
    prefix_frequencies: HashMap<String, u32>,
) -> Option<Combinations> {
    if combinations.suffixes.contains(&next_suffix.to_string()) {
        return None;
    }
    let next_prefixes = all_suffixes.get(&next_suffix).unwrap();
    let mut buffers: [Vec<&String>; 4] = [vec![], vec![], vec![], vec![]];
    for (count, mut prefix_list) in combinations.matching_prefixes.iter_mut() {
        let (mut stays, shifts): (Vec<String>, Vec<String>) = prefix_list
            .into_iter()
            .partition(|p| next_prefixes.contains(p));
        buffers[*count as usize - 1] = shifts;
        prefix_list = stays;
    }
    None
}
