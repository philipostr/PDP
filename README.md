# PDP
Poorly Designed Python. I try to (badly) build a Python-like interpreter from scratch in Rust

## Documentation

**Diary**  
_A compilation of entries accompanying most major commits. These entries detail my thoughts, difficulties, and reasonings for a lot of the things I did._
https://github.com/philipostr/PDP/blob/main/diary.md

**Specs**
* _An explanation of the TPBA parsing process_  
https://github.com/philipostr/PDP/blob/main/specs/README.md
* _List of tokens that will be extracted from a Python script_  
https://github.com/philipostr/PDP/blob/main/specs/Tokens.md
* _Formal TPG for this project_  
https://github.com/philipostr/PDP/blob/main/specs/TPG.md
* _Formal PTAG for this project_  
https://github.com/philipostr/PDP/blob/main/specs/TPG.md

## Development

To run unoptimized with trace logging, run (modified as necessary):
```
RUST_LOG=trace RUSTFLAGS="-Awarnings" cargo run
```

The following files will be created under the `pdp_out/` directory:
- `pdp.log`: The logs, if any, that were generated during execution.
- `token_stream.txt`: A pretty-print of the token-stream that was taken from the Python code.
- `parse_tree.txt`: A pretty-print of the concrete parse tree that was constructed from the token stream.
- `ast.txt`: A pretty-print of the abstract syntax tree that was extracted from the parse tree.
