# PTAG (Parse Tree Abstraction Grammar)

*The leaf nodes of the parse tree, which are of course tokens, get abstracted into their own respective abstraction nodes. For example, ASOP becomes asop, and KEYWORD becomes keyword. The only exceptions to this are that INDENT, NEWLINE, and END become empty.*

*This is the grammar used for the bottom-up abstraction stage of TPBA.*

```
Index: expr ⟶ access
```

```
DictTail: string expr ⟶ dictionary
```

```
Dict: string expr dictionary* ⟶ dictionary
```

```
ParamsTail: name ⟶ parameters
```

```
Params: name parameters* ⟶ parameters
```

```
ListTail: expr
```

```
List: expr expr*  ⟶ expr+
```

```
BracExpr.1: dictionary
```

```
BracExpr.2: expr+ ⟶ set
```

```
NameExpr.1: empty ⟶ arguments
            expr+ ⟶ arguments
```

```
NameExpr.2: empty   ⟶ empty
            access+ ⟶ access
```

```
ExprUnit.1: name arguments ⟶ function_call
            name empty     ⟶ variable
            name access    ⟶ variable
```

```
ExprUnit.2: expr
```

```
ExprUnit.3: empty ⟶ list
            expr+ ⟶ list
```

```
ExprUnit.4: empty      ⟶ dictionary
            dictionary ⟶ dictionary
            set        ⟶ set
```

```
ExprUnit.5: string
```

```
ExprUnit.6: number
```

```
ExprUnit.7: boolean
```

```
ExprBinary: op function_call ⟶ binary_op_rhs
            op variable      ⟶ binary_op_rhs
            op expr          ⟶ binary_op_rhs
            op list          ⟶ binary_op_rhs
            op dictionary    ⟶ binary_op_rhs
            op set           ⟶ binary_op_rhs
            op string        ⟶ binary_op_rhs
            op number        ⟶ binary_op_rhs
            op boolean       ⟶ binary_op_rhs
```

```
ExprUnary.1: function_call ⟶ expr
             variable      ⟶ expr
             expr          ⟶ expr
             list          ⟶ expr
             dictionary    ⟶ expr
             set           ⟶ expr
             string        ⟶ expr
             number        ⟶ expr
             boolean       ⟶ expr
```

```
ExprUnary.2: function_call ⟶ expr
             variable      ⟶ expr
             expr          ⟶ expr
             list          ⟶ expr
             dictionary    ⟶ expr
             set           ⟶ expr
             string        ⟶ expr
             number        ⟶ expr
             boolean       ⟶ expr
```

```
ExprUnary.3: function_call ⟶ function_call
             variable      ⟶ variable
             expr          ⟶ expr
             list          ⟶ list
             dictionary    ⟶ dictionary
             set           ⟶ set
             string        ⟶ string
             number        ⟶ number
             boolean       ⟶ boolean
```

```
Expr: function_call binary_op_rhs* ⟶ expr
      variable binary_op_rhs*      ⟶ expr
      expr binary_op_rhs*          ⟶ expr
      list binary_op_rhs*          ⟶ expr
      dictionary binary_op_rhs*    ⟶ expr
      set binary_op_rhs*           ⟶ expr
      string binary_op_rhs*        ⟶ expr
      number binary_op_rhs*        ⟶ expr
      boolean binary_op_rhs*       ⟶ expr
```

```
SideEffect.1: empty ⟶ arguments
              expr+ ⟶ arguments
```

```
SideEffect.2: access* asop expr ⟶ assign_op_rhs
```

```
Body.1: (empty|if_stmt|while_loop|for_loop|continue|break|return_stmt|function_def|function_call|assign_op)* ⟶ block
```

```
Body.2: expr ⟶ return_stmt
```

```
Result.1: (empty|if_stmt|while_loop|for_loop|continue|break|return_stmt|function_def|function_call|assign_op)+ ⟶ block
```

```
Result.2: name arguments     ⟶ function_call
          name assign_op_rhs ⟶ assign_op
```

```
Unit.1: expr function_call ⟶ if_stmt
        expr assign_op     ⟶ if_stmt
        expr block         ⟶ if_stmt
```

```
Unit.2: expr function_call ⟶ while_loop
        expr assign_op     ⟶ while_loop
        expr block         ⟶ while_loop
```

```
Unit.3: name expr function_call ⟶ for_loop
        name expr assign_op     ⟶ for_loop
        name expr block         ⟶ for_loop
```

```
Unit.4: continue
```

```
Unit.5: break
```

```
Unit.6: empty ⟶ return_stmt
        expr  ⟶ return_stmt
```

```
Unit.7: name parameters block       ⟶ function_def
        name parameters return_stmt ⟶ function_def
```

```
Unit.8: name arguments     ⟶ function_call
        name assign_op_rhs ⟶ assign_op
```

```
Scoped.1: empty
```

```
Scoped.2: if_stmt
          while_loop
          for_loop
          continue
          break
          return_stmt
          function_def
          function_call
          assign_op
```

```
Program.1: empty
```

```
Program.2: (empty|if_stmt|while_loop|for_loop|continue|break|return_stmt|function_def|function_call|assign_op)* ⟶ block
```
