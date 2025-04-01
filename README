
# Usage

To generate a basic report comparing your performance to the other person's performance:

```
cargo run acotis JayXon --lang rust
```

To print a report based on how things stood at a particular moment in time (defaults to the moment you run the script — see warnings below):

```
cargo run acotis JayXon --lang rust --cutoff 2025-03-31
```

To use chars scoring (defaults to bytes otherwise):

```
cargo run acotis JayXon --lang rust --scoring chars
```

To make the score bars wider or thinner (defaults to 20 or 21 units, whichever makes it possible to center-align everything perfectly):

```
cargo run acotis JayXon --lang rust --score-bar-width 30
```

To leave more or less room for the hole names on the left side (defaults to 34 characters, which is just enough to accomodate the longest hole name while leaving a margin of 2 characters):

```
cargo run acotis JayXon --lang rust --hole-name-width 50
```

To include a third golfer's performance in the score bars (can only include one additional golfer beyond the two being compared, and stats for that golfer are not printed beyond their appearance in the score bar):

```
cargo run acotis JayXon --lang rust --reference xnor-gate
```

# Warnings

This script is poorly-written and I feel bad :)

Generally speaking, you are on your own in terms of getting things right. If you specify a golfer that doesnsn't exist, you'll get an empty report instead of an enlightening error message. If you specify a `--hole-name-width` that's too small, the script will crash. If you specify a `--score-bar-width` that's too small, the script will crash. If you specify a `--lang` that doesn't exist, the script will hang, and then crash.

**The `--cutoff` date that you specify is not parsed into an actual timestamp; it is string-compared with dates given to me via code.golf's API.** That means that you need to be really careful or you will just get garbage. Here are some examples of meaningful strings you can pass as the cutoff date:

- `2025` — this considers all solutions that were submitted before the turn of the year 2025, because the string "2024-12-29T13:29:41.774046Z" compares as being less than the string "2025", but "2025-01-01T02:06:59.015992Z" copares as being greater.
- `2025-03` — this considers all solutions that were submitted before the turn of the month of March 2025.
- `2025-03-01` — this has the same meaning as "2025-03".
- `2025-03-31` — this considers all solutions that were submitted before the turn of midnight when it became 2025 March 31st.
- `current moment` (the default value) — this considers all solutions that have ever been submitted, because all numerical dates compare as being less than the string "current moment" due to the fact that this string starts with a "c" which is greater than any ASCII digit.

# Known bug

This script has a known bug where it doesn't filter out solutions that have been deleted. Sometimes, a hole's scoring judge will be made more strict after a cheese is discovered, and previously-existing solutions will be invalidated and removed from the site's leaderboard. **This script still presents those solutions as existing**, because I couldn't figure out how to tell via the API whether a solution is still valid or not. If you know how, feel free to make a PR.

