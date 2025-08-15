# Token types

*Any token that falls under multiple token types, such as `and` in `KEYWORD` and `OP`, will be categorized into the first of these token types found in this list, starting from the top.*

- `INDENT`: An indentation (4 spaces).
- `OP(Op)`: A non-assignment operation such as `+`, `//`, and `not`.
- `ASOP(Asop)`: An assignment operation such as `=`, `+=`, and `**=`.
- `KEYWORD(Keyword)`: A reserved keyword of the Python language such as `for`, `def`, and `return`.
- `NAME(String)`: An identifier of a variable, function, or anything else.
- `BRACKET(char)`: A `{`, `[`, or `(` that must be paired with `}`, `]`, `)` respectively.
- `STRING(String)`: A string literal specified with `""`, `''`, or other types such as `f''`. These specifying characters are removed in the token.
- `NUMBER(f64)`: A floating-point number. This also includes integers, as Python does not distinguish between the two.
- `BOOL(bool)`: A boolean `true` or `false`.
- `NEWLINE`: A newline character.
- `MISC(String)`: Any miscellaneous sequence of characters that are not included in the above tokens. This includes characters such as `:`, `,`, and `.`.
- `END`: The marker for the end of a script. This should only be generated as the final token in the stream.
