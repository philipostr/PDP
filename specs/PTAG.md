# PTAG (Parse Tree Abstraction Grammar)

*The leaf nodes of the parse tree, which are of course tokens, get abstracted into their own respective abstraction nodes. For example, ASOP becomes asop, and KEYWORD becomes keyword. The only exceptions to this are that INDENT, NEWLINE, and END become empty.*

*This is the grammar used for the bottom-up abstraction stage of TPBA.*

```
Index: ... ⟶ index
```

```
DictTail: ... ⟶ dictionary
```

```
Dict: ... ⟶ dictionary
```

```
ParamsTail: ... ⟶ param
```

```
Params: ... ⟶ params_list
```

```
ListTail: (...a) ⟶ a
```

```
List: ... ⟶ list
```

```
BracExpr: dictionary ⟶ dictionary
          list       ⟶ set
```

```
NameExpr: bracket('(') (...) bracket(')') ⟶ params_list
          empty                           ⟶ empty
          index{≥1}                       ⟶ index_chain
```

```
ExprBinary: ... ⟶ binary_op_lhs
```

```
ExprUnit: name params_list                     ⟶ function_call
          name empty                           ⟶ variable
          name index_chain                     ⟶ variable
          bracket('(') (...a) bracket(')')     ⟶ a
          bracket('[') (...) bracket(']')      ⟶ list
          bracket('{') dictionary bracket('}') ⟶ dictionary
          bracket('{') set bracket('}')        ⟶ set
          bracket('{') empty bracket('}')      ⟶ dictionary
          (...a)                               ⟶ a
```

```
ExprUnary: op(Minus) (...) ⟶ unary_op
           op(Not) (...)   ⟶ unary_op
           (...a)          ⟶ a
```

```
Expr: (...a) empty        ⟶ a
      (...) binary_op{≥1} ⟶ binary_op_chain
```

```
SideEffect: bracket('(') (...a) bracket(')') ⟶ list
            (...) asop (...)                 ⟶ assign_op_lhs
```

```
Body: keyword(Return) ... ⟶ return_stmt
      ...                 ⟶ block
```

```
Result: name list empty          ⟶ function_call
        name assign_op_lhs empty ⟶ assign_op
        ...                      ⟶ block
```

```
Unit: keyword(If) ...          ⟶ if_stmt
      keword(For) ...          ⟶ for_loop
      keyword(While) ...       ⟶ while_loop
      keyword(Continue) ...    ⟶ continue_stmt
      keyword(Break) ...       ⟶ break_stmt
      keyword(Return) ...      ⟶ return_stmt
      keyword(def) ...         ⟶ function_def
      name list empty          ⟶ function_call
      name assign_op_lhs empty ⟶ block
```

```
Scoped: empty             ⟶ empty
        empty{...} (...a) ⟶ a
```

```
Program: ... ⟶ script
```
