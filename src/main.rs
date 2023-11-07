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
    let mut suffix_map: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut prefix_frequencies: HashMap<&str, u32> = HashMap::new();

    for w in words.iter() {
        if w.len() < 3 || w.len() > 6 {
            continue;
        }
        let split_index = w.len() - 2;
        let (prefix, suffix) = w.split_at(split_index);
        match suffix_map.get_mut(suffix) {
            Some(prefixes) => prefixes.push(prefix),
            None => {
                suffix_map.insert(suffix, vec![prefix]);
            }
        }

        match prefix_frequencies.get_mut(prefix) {
            Some(count) => *count += 1,
            None => {
                prefix_frequencies.insert(prefix, 1);
            }
        }
    }

    suffix_map.retain(|k, v| v.len() >= 4);
    for (_, prefixes) in suffix_map.iter_mut() {
        prefixes.retain(|&p| prefix_frequencies[p] > 1)
    }
    prefix_frequencies.retain(|k, v| v > &mut 1);

    println!("{:?}", suffix_map.len());
    println!("{:?}", prefix_frequencies.len());
    None
}

fn search_combinations(
    combo_suffixes: &[&str],
    all_suffixes: HashMap<&str, Vec<&str>>,
    prefix_frequencies: HashMap<&str, u32>,
) -> Option<Combinations> {
    None
}
