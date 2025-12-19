Outside from this README, the files you see in this folder (renamed `pdp_out/`) are the exact outputs from running
```bash
RUST_LOG=trace RUSTFLAGS="-Awarnings" cargo run
```
on the following sample Python script:
```python
x = 10

def outer(n):
    total = 0

    def inner(i):
        return i + n / x

    for i in range(n):
        total += inner(i)

    return total

outer(5)
```

The explanation of each file's contents is found in the [repo README](https://github.com/philipostr/PDP?tab=readme-ov-file#development).
