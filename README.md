# PDP
Poorly Designed Python. I try to (badly) build a Python-like Interpreter from scratch in Rust

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
RUST_LOG=<target-module>=trace RUSTFLAGS="-Awarnings" cargo run 2> pdp_out.txt
```

Logs (if any) will be sent to `pdp.log`, and the outputs of every stage (token stream, concrete syntax tree, AST, etc.) will be put into `pdp_out.txt`
