# Kansai algo contest

[Contest description](https://docs.google.com/document/d/1_8gVOE6tU7chN40PgAPCEdZFAI5Zr6lPxqK88lYttKY/edit)

## Notes

- 1 prefix must match 4 suffixes
- 4 prefixes must match 3 suffixes
- 4 prefixes must match 2 suffixes
- prefixes are 1-3 characters long
- suffixes are 2-6 characters long
- ignore plural words; use "ends with 's'" as a heuristic for this
- can initially filter out words less than 3 characters or more than 9 characters or ending in 's'
- this reduces the 10k word set to 6615 words!

## The Algorithm

The code effectively builds a graph of suffixes and the prefixes that occur for each suffix, and
then does a search within that colossal structure to find suffix combinations with appropriate
prefix-suffix-match cardinalities. The search turns out to be a DFS, but BFS could probably also
work? Though that would mean most pruning couldn't happen until the final suffix so that could
result in larger memory usage.

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
----------------------------------------------------------------
| 2k        | 0.561s    | 1535               | 3                  |
| 5k        | 1.511s    | 3506               | 229                |
| 10k       | 107.250s  | 6616               | 72053              |


