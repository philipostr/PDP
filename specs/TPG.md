# TPG (Token Parsing Grammar)

## Context Object

*When parsing the tokens into a concrete syntax tree, the following pieces of context are carried down the tree, being modified for subtrees when necessary.*

```
Context = {
    indentation: int  = 0,     // denoted with `n`
    in_loop:     bool = false, // denoted with `l`
    in_function: bool = false  // denoted with `f`
}
```

## Grammar

*This is the grammar used for the top-down parsing stage of TPBA.*

```
// The entire script.

Program: END
       | Scoped* END
```

```
// A line that is scoped with `n` indents and ends with a NEWLINE.

Scoped: NEWLINE
      | INDENT{n} Unit
```

```
// The contents of a line, including the NEWLINE.

Unit:  KEYWORD(If) Expr MISC(':') Result
     | KEYWORD(While) Expr MISC(':') Result   [l = true]
     | KEYWORD(For) NAME KEYWORD(in) Expr MISC(:) Result   [l = true]
 [l] | KEYWORD(Continue) NEWLINE
 [l] | KEYWORD(Break) NEWLINE
 [f] | KEYWORD(Return) Expr? NEWLINE
     | KEYWORD(Def) NAME BRACKET('(') Params? BRACKET(')') MISC(':') Body   [f = true]
     | NAME SideEffect NEWLINE
```

```
// A helper node to give blocks the option to be a single in-line statement.

Result: NEWLINE Scoped+   [n += 1]
      | NAME SideEffect NEWLINE
```

```
// A helper node to give function bodies the option to be a single in-line return statement.

Body: NEWLINE Scoped+   [n += 1]
    | KEYWORD(Return) Expr NEWLINE
```

```
// To call NAME as a function, or assign to it a value as a variable or indexed object.

SideEffect: BRACKET('(') List? BRACKET(')')
          | Index* ASOP Expr
```

```
// Any expression that can return a value.

Expr: ExprUnary ExprBinary*
```

```
// An expression potentially starting with a unary operation.

ExprUnary: OP(Minus) ExprUnit
         | OP(Not) ExprUnit
         | ExprUnit
```

```
// The main container of any kind of expression.

ExprUnit: NAME NameExpr
        | BRACKET('(') Expr BRACKET(')')
        | BRACKET('[') List? BRACKET(']')
        | BRACKET('{') BracExpr? BRACKET('}')
        | STRING
        | NUMBER
        | BOOLEAN
```

```
// Helper node for Expr to have multiple subexpressions joined through binary operations.

ExprBinary: OP ExprUnit
```

```
// Helper node for ExprUnit to access a NAME in ways outside of basic value-retrieval.

NameExpr: BRACKET('(') List? BRACKET(')')
        | Index*
```

```
// Helper node for ExprUnit to create sets and dictionaries.

BracExpr: Dict
        | List
```

```
// A comma-separated list of expressions.

List: Expr ListTail*
```

```
// Helper node for List to have multiple values.

ListTail: MISC(',') Expr
```

```
// List but only allowing identifiers.

Params: NAME ParamsTail*
```

```
// Helper node for Params to have multiple values.

ParamsTail: MISC(',') NAME
```

```
// A comma-separated list of key-value pairs.

Dict: STRING MISC(':') Expr DictTail*
```

```
// Helper node for Dict to have multiple key-value pairs.

DictTail: MISC(',') STRING MISC(':') Expr
```

```
// The index of an indexable NAME.

Index: BRACKET('[') Expr BRACKET(']')
```
