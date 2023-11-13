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

At the end of each run, 

## Results

I ran the algorithm on the 2k, 5k, 10k, and 370k word lists with the following results:

| Word list | Time   | Words after filter | Combinations found |
----------------------------------------------------------------
| 2k        | 0.617s | 1535               | 3                  |
| 5k        | 8.201s | 3506               | 229                |
| 10k       | 4m28s  | 6616               | 72035              |
| 370k      |        | 160302             | |


