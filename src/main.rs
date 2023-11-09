use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::ops::{Range, RangeInclusive};
use std::option::Option;

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
struct WordList {
    words: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Combinations {
    suffixes: Vec<String>,
    matching_prefixes: HashMap<u32, HashSet<String>>,
}

impl Display for Combinations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Suffixes: {:?}\n1-prefixes: {:?}\n2-prefixes: {:?}\n3-prefixes: {:?}\n4-prefixes: {:?}",
            self.suffixes,
            self.matching_prefixes.get(&1),
            self.matching_prefixes.get(&2),
            self.matching_prefixes.get(&3),
            self.matching_prefixes.get(&4)
        )
    }
}

impl Combinations {
    fn new() -> Self {
        Combinations {
            suffixes: vec![],
            matching_prefixes: HashMap::from([
                (1, HashSet::new()),
                (2, HashSet::new()),
                (3, HashSet::new()),
                (4, HashSet::new()),
            ]),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let file = File::open("./data/words10k.json")?;
    let reader = BufReader::new(file);
    let list_words: WordList = serde_json::from_reader(reader)?;
    // let list_words = WordList {
    //     words: reader.lines().filter_map(|l| l.ok()).collect(),
    // };
    println!("Loaded {} words", list_words.words.len());
    let words = list_words.words.clone();
    let count = words
        .iter()
        .filter(|w| w.len() >= 3 && w.len() <= 9 && w.as_bytes()[w.len() - 1] != b's')
        .count();
    println!("{} words between 3-9 characters not ending in 's'", count);
    let results: Vec<Combinations> = find_combinations(&words).unwrap_or_default();
    let mut out_file = File::create("output10k.json").unwrap();
    out_file.write_all(&serde_json::to_string(&results).unwrap().into_bytes())?;
    println!("{}", results.len());

    Ok(())
}

fn find_combinations(words: &Vec<String>) -> Option<Vec<Combinations>> {
    let mut suffix_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut prefix_frequencies: HashMap<String, u32> = HashMap::new();

    for w in words.iter() {
        if w.len() < 3 || w.len() > 9 || w.as_bytes()[w.len() - 1] == b's' {
            continue;
        }
        for suffix_len in 2..=6 {
            if w.len() < suffix_len + 1 {
                break;
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
    }
    // suffix_map.retain(|_k, v| v.len() >= 4);
    for (_, prefixes) in suffix_map.iter_mut() {
        // remove all prefixes that only occur once or are longer than 3 characters
        prefixes.retain(|p| prefix_frequencies[p] > 1 && p.len() <= 3)
    }
    prefix_frequencies.retain(|_k, v| v > &mut 1);

    println!("{:?}", suffix_map.len());
    println!("{:?}", prefix_frequencies.len());
    let mut all_suffixes: Vec<String> = suffix_map.clone().into_keys().collect();
    all_suffixes.sort();

    Some(
        all_suffixes
            .iter()
            .filter_map(|s| {
                search_combinations(
                    &mut Combinations::new(),
                    s.to_string(),
                    &all_suffixes,
                    &suffix_map,
                )
            })
            .flatten()
            .filter(validate_combination)
            .collect(),
    )
}

fn search_combinations(
    mut combinations: &mut Combinations,
    next_suffix: String,
    all_suffixes: &Vec<String>,
    suffix_map: &HashMap<String, Vec<String>>,
) -> Option<Vec<Combinations>> {
    if combinations.suffixes.contains(&next_suffix.to_string()) || combinations.suffixes.len() >= 4
    {
        return None;
    }
    let next_prefixes = suffix_map.get(&next_suffix).unwrap();
    // println!(
    //     "Add {} to suffixes: {:?}",
    //     &next_suffix, &combinations.suffixes
    / );
    combinations.suffixes.push(next_suffix.clone());
    let mut buffers: [HashSet<String>; 4] = [
        HashSet::from_iter(next_prefixes.clone().into_iter()),
        HashSet::new(),
        HashSet::new(),
        HashSet::new(),
    ];
    // println!("suffix: {:?}", &next_suffix);
    // println!("next: {:?}", &next_prefixes);

    // get the prefixes that need to move up one spot and collect into buffers
    // can't directly move them to the next prefix_list because then they'll keep getting carried
    // up
    for (count, prefix_list) in combinations.matching_prefixes.iter_mut() {
        if count == &4 {
            continue;
        };
        let immut_prefix_list = prefix_list.clone();
        let (shifts, stays): (HashSet<String>, HashSet<String>) = immut_prefix_list
            .into_iter()
            .partition(|p| next_prefixes.contains(p));
        // println!("shifts: {:?}", &shifts);
        // println!("stays: {:?}", &stays);
        buffers[*count as usize].extend(shifts.into_iter());
        *prefix_list = stays;
    }

    // println!("buffers: {:?}", &buffers);

    // slam the buffers into the spot they need to go into
    for (count, prefix_list) in combinations.matching_prefixes.iter_mut() {
        if count == &0 {
            continue;
        };
        let buffer_contents = &mut buffers[*count as usize - 1];
        prefix_list.extend(buffer_contents.drain().into_iter());
    }

    if !validate_combination(combinations) {
        if !combinations.matching_prefixes.get(&3).unwrap().is_empty() {
            println!("Goodbye {}", combinations);
        }
        None
    } else {
        match combinations.matching_prefixes.get(&4) {
            // if there are some 4-cardinality prefixes, we must have 4 suffixes, so we can check
            // the combination and discard if it's bad
            Some(vec) if !vec.is_empty() => {
                if validate_combination(combinations) {
                    Some(vec![combinations.clone()])
                } else {
                    println!("Goodbye {}", combinations);
                    None
                }
            }
            // if there aren't any 4-card prefixes, we can move along and continue digging
            Some(_) => Some(
                all_suffixes
                    .iter()
                    .filter_map(|s| {
                        search_combinations(combinations, s.clone(), all_suffixes, suffix_map)
                    })
                    .flatten()
                    .collect(),
            ),
            // shouldn't be any way to get here but it's not important enough to panic
            None => None,
        }
    }
}

fn validate_combination(combinations: &Combinations) -> bool {
    match combinations.suffixes.len() {
        // if there are 4 suffixes, check we match the conditions
        4 => {
            if combinations.matching_prefixes.get(&4).unwrap().is_empty() {
                return false;
            }
            if combinations.matching_prefixes.get(&3).unwrap().len() < 4 {
                return false;
            }
            if combinations.matching_prefixes.get(&2).unwrap().len() < 4 {
                return false;
            }
            true
        }
        // if there are MORE than 4, invalid
        n if n > 4 => false,
        n if n < 4 => !combinations
            .matching_prefixes
            .get(&(n as u32))
            .unwrap()
            .is_empty(),
        // otherwise it's not yet invalid
        _ => true,
    }
}
