# Clausewitz Parser

A tool used to parse and generate databases from Paradox Script gamefiles.
This tool programmatically parses Paradox Script and inserts its contents into a Postgres database, keeping track of each item's parent scope.

## Getting Started

### Dependencies

- Rust
- Postgres

### Running this Project

1. Clone this project locally.

2. Set your DATABASE_URL to any existing database in a running Postgres server. This can be done in the shell, but the preferred way to do this is in a .env file in the project directory:

```bash
DATABASE_URL='<url>'
```

3. Initialize the gamefiles table:

```bash
cargo run --release -- --initialize
```

Note: Initializing the table will clean up ALL previous entries in the gamefiles table.

4. Add entries to the gamefiles table, along with a provided game name:

```bash
cargo run --release -- --game '<name>' --files <gamefiles>/**/*.txt
```

Note: Adding entries to the table will also clean up previous entries matching the game name. The provided game name is also what will show up if you decide to use this database with [Clausewitz Manifest](https://github.com/aw-andre/clausewitz-manifest).
Note: Some .txt files may have to be manually removed as they are not valid Paradox Script. For example, Clausewitz Parser will attempt to parse license files and will panic as it was given invalid input.
Note: The \*\* option may not always work. If it doesn't work, check your shell's support for recursive globbing.

5. The files should finish parsing in a few minutes. For reference, EU4 gamefiles (which provide over one million rows after parsing) takes less than five minutes to parse on my machine.

6. Verify in Postgres that each key's parent_id matches the primary_id of the key in its outer scope. If this is not the case, something has gone wrong.

## Contributing

If you can make any improvements to this project, please don't hesitate to send a pull request. Any contribution is welcome!
Some areas of possible improvement include:

- compilation for release builds on various CPU architectures and operating systems
- speeding up database insertion time (the bulk of runtime can be attributed to database insertion as this program works by letting Postgres return primary keys upon each single-row insertion; this most likely can be done much faster in memory before doing a large multi-row insertion, though program logic would likely have to be implemented for this)
- testing on other games (I do not currently have the files for other games and this is only tested for EU4; as there is no known public documentation on Paradox Script, I had to account for some syntax edge-cases like quote-delimited lists after testing)
