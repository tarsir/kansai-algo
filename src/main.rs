use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufReader, Write};
use std::option::Option;
use std::process::exit;

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
            matching_prefixes: HashMap::from([(1, vec![]), (2, vec![]), (3, vec![]), (4, vec![])]),
        }
    }

    fn prefix_exists<S: Into<String> + Copy + Eq + Hash>(&self, prefix: S) -> Option<u32> {
        for (id, p_list) in self.matching_prefixes.iter() {
            if p_list.contains(&prefix.into()) {
                return Some(*id);
            }
        }
        None
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let file = File::open("./data/words2k.json")?;
    let reader = BufReader::new(file);
    let list_words: WordList = serde_json::from_reader(reader)?;
    // let list_words = WordList {
    //     words: reader.lines().filter_map(|l| l.ok()).collect(),
    // };
    println!("Loaded {} words", list_words.words.len());
    let words = list_words.words;
    let words: Vec<String> = words
        .into_iter()
        .filter(|w| w.len() >= 3 && w.len() <= 9 && w.as_bytes()[w.len() - 1] != b's')
        .collect();
    let count = words.len();
    println!("{} words between 3-9 characters not ending in 's'", count);
    let mut results: Vec<Combinations> = find_combinations(&words).unwrap_or_default();
    for combo in results.iter_mut() {
        combo.matching_prefixes.remove(&1).unwrap();
        combo.suffixes.sort();
    }
    results.sort_by_cached_key(|c| c.suffixes.join(""));
    results.dedup_by_key(|a| a.suffixes.join(""));

    let mut out_file = File::create("output2k.json").unwrap();
    out_file.write_all(&serde_json::to_string(&results).unwrap().into_bytes())?;
    println!("Found {} combinations", results.len());

    Ok(())
}

fn find_combinations(words: &Vec<String>) -> Option<Vec<Combinations>> {
    let mut suffix_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut prefix_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut prefix_frequencies: HashMap<String, u32> = HashMap::new();
    let mut starting_suffixes: HashSet<String> = HashSet::new();

    for w in words.iter() {
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
                    suffix_map.insert(suffix.clone(), vec![prefix.clone()]);
                }
            }

            match prefix_map.get_mut(&prefix) {
                None => {
                    prefix_map.insert(prefix.clone(), vec![suffix.clone()]);
                }
                Some(suffixes) => suffixes.push(suffix.clone()),
            }

            match prefix_frequencies.get_mut(&prefix) {
                Some(count) => {
                    *count += 1;
                    if *count >= 4 {
                        starting_suffixes.insert(suffix.clone());
                    }
                }
                None => {
                    prefix_frequencies.insert(prefix, 1);
                }
            }
        }
    }
    println!("suffixes pre-filter: {:?}", suffix_map.len());
    println!("prefixes pre-filter: {:?}", prefix_frequencies.len());
    for (_, prefixes) in suffix_map.iter_mut() {
        // remove all prefixes that only occur once or are longer than 3 characters
        prefixes.retain(|p| prefix_frequencies[p] > 1 && p.len() <= 3)
    }
    suffix_map.retain(|_k, v| v.len() >= 6);
    prefix_map.retain(|_k, v| v.len() >= 2);

    for (_, suffixes) in prefix_map.iter_mut() {
        // remove all suffixes that don't appear in the filtered suffix list
        suffixes.retain(|s| suffix_map.contains_key(s))
    }
    prefix_frequencies.retain(|_k, v| v > &mut 1);
    // suffix_map.retain(|k, _v| starting_suffixes.contains(k));

    println!("suffixes post-filter: {:?}", suffix_map.len());
    println!("prefixes post-filter: {:?}", prefix_map.len());
    println!("prefix frequencies: {:?}", prefix_frequencies.len());
    let mut all_suffixes: Vec<String> = suffix_map.clone().into_keys().collect();
    all_suffixes.sort();
    let mut counter = 0;
    let mut index = 0;

    println!("special suffixes pre-filter: {:?}", starting_suffixes.len());

    let suffixes_as_set = all_suffixes
        .clone()
        .into_iter()
        .collect::<HashSet<String>>();
    let starting_suffixes = starting_suffixes
        .intersection(&suffixes_as_set)
        .map(|x| x.to_owned());
    let starting_suffixes: Vec<String> = starting_suffixes.collect();

    println!(
        "special suffixes post-filter: {:?}",
        starting_suffixes.len()
    );

    let center_prefixes = prefix_map
        .iter()
        .filter_map(|(p, s_list)| {
            if s_list.len() >= 4 {
                Some(p.clone())
            } else {
                None
            }
        })
        .unique()
        .collect::<Vec<String>>();
    println!("center prefix candidates: {:?}", center_prefixes.len());

    return Some(
        center_prefixes
            .iter()
            .filter_map(|p| {
                // println!("Searching from prefix {}", p);
                let result = search_from_center_candidates(
                    p,
                    &mut Combinations::new(),
                    &suffix_map,
                    &prefix_map,
                    &mut counter,
                );
                // println!("Searched {} nodes", counter);
                counter = 0;
                result
            })
            .flatten()
            .collect(),
    );

    exit(0);

    let suffix_one = starting_suffixes.first().unwrap();
    build_results_nah(
        "ar".to_string(),
        &suffix_map,
        &prefix_map,
        &prefix_frequencies,
    );

    Some(
        starting_suffixes
            .iter()
            .filter_map(|s| {
                println!("Searching from {}", &s);
                let res = search_combinations(
                    &mut Combinations::new(),
                    s.to_string(),
                    &suffix_map,
                    &mut counter,
                );
                println!("Searched {} nodes", counter);
                counter = 0;
                index += 1;
                res
            })
            .flatten()
            .filter(validate_combination)
            .collect(),
    )
}

fn search_from_center_candidates(
    center_prefix: &String,
    combinations: &mut Combinations,
    suffix_map: &HashMap<String, Vec<String>>,
    prefix_map: &HashMap<String, Vec<String>>,
    mut counter: &mut i32,
) -> Option<Vec<Combinations>> {
    if combinations.suffixes.len() == 4 && validate_combination(combinations) {
        return Some(vec![combinations.clone()]);
    }

    if combinations.suffixes.len() >= 4 {
        return None;
    }

    let next_suffixes: Vec<&String> = prefix_map[center_prefix]
        .iter()
        .filter(|s| !combinations.suffixes.contains(s))
        .collect();

    Some(
        next_suffixes
            .into_iter()
            .filter_map(|suffix| {
                let mut new_combinations = combinations.clone();
                new_combinations.suffixes.push(suffix.clone());
                let next_prefixes: &Vec<String> = suffix_map[suffix].as_ref();
                for p in next_prefixes.iter() {
                    if let Some(index) = new_combinations.prefix_exists(p) {
                        if index < 4 {
                            new_combinations
                                .matching_prefixes
                                .get_mut(&(index + 1))
                                .unwrap()
                                .push(p.to_string());
                            new_combinations
                                .matching_prefixes
                                .get_mut(&(index))
                                .unwrap()
                                .retain(|s| !s.eq(p));
                        }
                    } else {
                        new_combinations
                            .matching_prefixes
                            .get_mut(&1)
                            .unwrap()
                            .push(p.to_string());
                    }
                }
                if validate_combination(&new_combinations) {
                    *counter += 1;
                    search_from_center_candidates(
                        center_prefix,
                        &mut new_combinations,
                        suffix_map,
                        prefix_map,
                        &mut counter,
                    )
                } else {
                    None
                }
            })
            .flatten()
            .collect(),
    )
}

fn build_results_nah(
    seed_suffix: String,
    suffix_map: &HashMap<String, Vec<String>>,
    prefix_map: &HashMap<String, Vec<String>>,
    prefix_frequencies: &HashMap<String, u32>,
) {
    let center_prefixes: Vec<String> = suffix_map[&seed_suffix]
        .clone()
        .into_iter()
        .filter(|p| prefix_frequencies[p] >= 4)
        .collect();
    println!("SEED");
    println!("{} -> {:?}", &seed_suffix, center_prefixes);
    let second_suffixes: Vec<String> = center_prefixes
        .iter()
        .filter_map(|p| {
            let mut suffixes = prefix_map[p].clone();
            suffixes.retain(|s| s != &seed_suffix);

            if suffixes.len() >= 2 {
                Some(
                    suffixes
                        .into_iter()
                        .filter(|s| s != &seed_suffix && !suffix_map[s].contains(p))
                        .collect::<Vec<_>>(),
                )
            } else {
                None
            }
        })
        .flatten()
        .unique()
        .collect();

    println!("LEVEL 2");
    for s in &second_suffixes {
        println!("{} -> {:?}", &s, suffix_map[s]);
    }

    let third_suffixes: Vec<String> = center_prefixes
        .iter()
        .filter_map(|p| {
            let overlaps_second = second_suffixes.iter().any(|s| prefix_map[p].contains(&s));
            if overlaps_second {
                Some(
                    prefix_map[p]
                        .clone()
                        .into_iter()
                        .filter(|s| s != &seed_suffix && s != p)
                        .collect::<Vec<_>>(),
                )
            } else {
                None
            }
        })
        .flatten()
        .unique()
        .collect();
    println!("LEVEL 3");

    for s in &third_suffixes {
        println!("third: {:?} -> {:?}", &s, suffix_map[s]);
    }

    let fourth_suffixes: Vec<String> = center_prefixes
        .iter()
        .filter_map(|p| {
            println!("{:?} -> {:?}", p, prefix_map[p]);
            let overlaps_second = second_suffixes.iter().any(|s| prefix_map[p].contains(&s));
            let overlaps_third = third_suffixes.iter().any(|s| prefix_map[p].contains(&s));
            if overlaps_third && overlaps_second {
                Some(
                    prefix_map[p]
                        .clone()
                        .into_iter()
                        .filter(|s| s != &seed_suffix && s != p)
                        .collect::<Vec<_>>(),
                )
            } else {
                None
            }
        })
        .flatten()
        .unique()
        .collect();
    println!("LEVEL 4");
    for s in &fourth_suffixes {
        println!("fourth: {:?} -> {:?}", &s, suffix_map[s]);
    }
}

fn search_combinations(
    combinations: &mut Combinations,
    next_suffix: String,
    suffix_map: &HashMap<String, Vec<String>>,
    mut counter: &mut i32,
) -> Option<Vec<Combinations>> {
    if combinations.suffixes.contains(&next_suffix.to_string()) || combinations.suffixes.len() >= 4
    {
        return None;
    }
    let next_prefixes = suffix_map.get(&next_suffix).unwrap();
    combinations.suffixes.push(next_suffix.clone());
    let mut buffers: [HashSet<String>; 4] = [
        HashSet::from_iter(next_prefixes.clone().into_iter()),
        HashSet::new(),
        HashSet::new(),
        HashSet::new(),
    ];

    // get the prefixes that need to move up one spot and collect into buffers
    // can't directly move them to the next prefix_list because then they'll keep getting carried
    // up
    for (count, prefix_list) in combinations.matching_prefixes.iter_mut() {
        if count == &4 {
            continue;
        };
        let immut_prefix_list = prefix_list.clone();
        let (shifts, stays): (Vec<String>, Vec<String>) = immut_prefix_list
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
        prefix_list.extend(buffer_contents.drain().into_iter());
    }

    deduplicate_prefix_sets(combinations);

    if !validate_combination(combinations) {
        None
    } else {
        *counter += 1;
        match combinations.matching_prefixes.get(&4) {
            // if there are some 4-cardinality prefixes, we must have 4 suffixes, so we can check
            // the combination and discard if it's bad
            Some(vec) if !vec.is_empty() => {
                if validate_combination(combinations) {
                    Some(vec![combinations.clone()])
                } else {
                    None
                }
            }
            // if there aren't any 4-card prefixes, we can move along and continue digging
            Some(_) => Some(
                suffix_map
                    .keys()
                    .filter_map(|s| {
                        search_combinations(
                            &mut combinations.clone(),
                            s.clone(),
                            suffix_map,
                            &mut counter,
                        )
                    })
                    .flatten()
                    .collect(),
            ),
            // shouldn't be any way to get here but it's not important enough to panic
            None => None,
        }
    }
}

fn deduplicate_prefix_sets(combinations: &mut Combinations) {
    let mut total_prefix_set = combinations.matching_prefixes.get(&4).unwrap().clone();
    let three_prefixes: &mut Vec<String> = combinations.matching_prefixes.get_mut(&3).unwrap();
    three_prefixes.retain(|p| !total_prefix_set.contains(p));
    three_prefixes.iter().for_each(|p| {
        total_prefix_set.push(p.clone());
    });
    let two_prefixes = combinations.matching_prefixes.get_mut(&2).unwrap();
    two_prefixes.retain(|p| !total_prefix_set.contains(p));
    two_prefixes.iter().for_each(|p| {
        total_prefix_set.push(p.clone());
    });
    let one_prefixes = combinations.matching_prefixes.get_mut(&1).unwrap();
    one_prefixes.retain(|p| !total_prefix_set.contains(p));
    one_prefixes.iter().for_each(|p| {
        total_prefix_set.push(p.clone());
    });
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
        // if only one, the
        1 => !combinations
            .matching_prefixes
            .get(&(1 as u32))
            .unwrap()
            .is_empty(),
        // if there are MORE than 4, invalid
        n if n > 4 => false,
        n if n == 2 || n == 3 => {
            if !(1..=(n - 1)).all(|i| {
                combinations
                    .matching_prefixes
                    .get(&(i as u32))
                    .unwrap()
                    .len()
                    >= 4
            }) {
                false
            } else {
                true
            }
        }
        // otherwise it's not yet invalid
        _ => true,
    }
}
