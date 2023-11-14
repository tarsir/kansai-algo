# Kansai algo contest

[Contest description](https://docs.google.com/document/d/1_8gVOE6tU7chN40PgAPCEdZFAI5Zr6lPxqK88lYttKY/edit)

## Notes

- 1 prefix must match 4 suffixes
- 4 prefixes must match 3 suffixes
- 4 prefixes must match 2 suffixes
- prefixes are 1-3 characters long
- suffixes are 2-6 characters long
- ignore plural words; use "ends with 's'" as a heuristic for this
- can initially filter out words less than 3 characters or more than 9
  characters or ending in 's'
- this reduces the 10k word set to 6615 words!

## The Algorithm

The code effectively builds a graph of suffixes and the prefixes that occur for each suffix, and
then does a search within that colossal structure to find suffix combinations with appropriate
prefix-suffix-match cardinalities, starting from the possible center prefixes. The search turns
out to be a DFS, but BFS could probably also work? Though that would mean most pruning couldn't
happen until the final suffix so that could result in larger memory usage.

At the end of each run, the program will write its output to a file. That output will be a series
of unique objects like:

```json
{
  "suffixes": [
    "an",
    "ar",
    "ell",
    "est"
  ],
  "matching_prefixes": {
    "2": [
      "s",
      "m",
      "cle",
      "r"
    ],
    "3": [
      "t",
      "w",
      "f",
      "c"
    ],
    "4": [
      "b"
    ]
  }
}
```

Each combination of 4 suffixes will have _all_ usable prefixes based on the number of suffixes
each prefix matches with. This means it's possible to have multiple valid game grids for one of
these objects.

## Results

I ran the algorithm on the 2k, 5k, and 10k word lists with the following results:

| Word list | Time      | Words after filter | Combinations found |
------------|-----------|--------------------|--------------------|
| 2k        | 0.561s    | 1535               | 3                  |
| 5k        | 1.511s    | 3506               | 229                |
| 10k       | 107.250s  | 6616               | 72053              |

## Notes about main.rs

For posterity's sake, I preserved the failed algorithm attempts in the form of their entrypoint
functions in `src/main.rs`. They are not named well, but I am lazy. Sorry.

### `search_combinations`

This was the first attempt I made. This algorithm is close to what I ended up doing, but with some
differences:

- The working algorithm starts its search from a possible center prefix candidate, where this
  algorithm only works off of suffixes. This makes the search space far larger than necessary
- The "prefix management" that pushes repeated prefixes into a higher match list uses the
  [partition](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.partition)
  method. The impact on performance is unclear, but it was taking a lot of time when observed
  in flamegraph, so the working algorithm does away with that.
- The prefix management also uses two loops and a set of buffers, which may not be necessary
- It also has a step to deduplicate prefixes across the buffers, which also may not be necessary
- This algorithm also doesn't make use of a prefix-to-suffixes map

I abandoned this algorithm mostly because it was very slow, but it was also initially not
providing correct results.

### `build_results_nah`

This one is smaller, but still didn't work. The idea was to start from a suffix and then identify
the possible center prefixes, then build the suffix-prefix list based on the overlap as you
iterate over the possibilities of suffix -> prefixes -> suffixes and so on. There might be
something workable here, but I couldn't figure out how to get this purely-iterative solution
to work, which led me back to doing a graph search.

### The working algorithm

After I got the working algorithm running, I made a few optimizations that may be interesting
to note.

- I was initially doing a lot of cloning of the intermediate data structures (vectors,
  hash maps, and hash sets which are just hash maps under the hood). Some of them are
  unavoidable, but I think the unnecessary ones are gone and this cut the time by about
  30% (the 5k list went from ~8s to ~6s)
- Since hash sets don't provide much if any performance gains for small lists, I also
  replaced many of those with vectors. This helped a lot and got another couple seconds
  from the 5k list, down to about 4s
- The last and biggest optimization was in the `validate_combinations` method, where I
  added the `n == 2 or n == 3` case. Before, I only checked that the n-th prefix list
  was non-empty, but this check only applies to the highest-level prefix list (which will
  eventually become the 4 prefix list). By also checking in the middle cases if everything
  below the highest-level prefix has at least 4 elements, we also eliminate a lot of bad
  options. This was the biggest speed up, bringing the 5k list from ~4s to ~1.5s and the
  10k list below 2 minutes.
