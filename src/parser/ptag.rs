#![allow(dead_code)]

use super::building_blocks::*;

#[derive(Debug)]
pub enum OperationTree {
    Unary {
        operation: Op,
        value: Box<OperationTree>,
    },
    Binary {
        operation: Op,
        left: Box<OperationTree>,
        right: Box<OperationTree>,
    },
    Identity(AstNode),
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AstNode {
    // token nodes
    op(Op),
    asop(Asop),
    keyword(Keyword),
    name(String),
    bracket(char),
    string(String),
    number(f64),
    boolean(bool),
    misc(char),

    // meta nodes
    multiple(Vec<Self>),

    // meaningful nodes
    access(Vec<OperationTree>),
    arguments(Vec<OperationTree>),
    assign_op {
        variable: String,
        accesses: Vec<OperationTree>,
        asop: Asop,
        value: Box<OperationTree>,
    },
    assign_op_rhs {
        accesses: Vec<OperationTree>,
        asop: Asop,
        rhs: Box<OperationTree>,
    },
    binary_op_rhs {
        operation: Op,
        rhs: Box<OperationTree>,
    },
    block(Vec<AstNode>),
    r#break,
    r#continue,
    dictionary(Vec<(String, OperationTree)>),
    empty,
    expr(Box<OperationTree>),
    for_loop {
        loop_variable: String,
        iterator: Box<OperationTree>,
        body: Box<AstNode>,
    },
    function_call {
        function: String,
        arguments: Vec<OperationTree>,
    },
    function_def {
        identifier: String,
        parameters: Vec<String>,
        body: Box<AstNode>,
    },
    if_stmt {
        condition: Box<OperationTree>,
        then: Box<AstNode>,
    },
    list(Vec<OperationTree>),
    parameters(Vec<String>),
    return_stmt(Option<Box<OperationTree>>),
    set(Vec<OperationTree>),
    variable {
        identifier: String,
        accesses: Vec<OperationTree>,
    },
    while_loop {
        condition: Box<OperationTree>,
        body: Box<AstNode>,
    },
}

macro_rules! tuplify {
    ($node:expr, $variant:ident) => {
        match $node {
            AstNode::$variant(x) => x,
            bad => panic!("Tried reading {bad:?} as {}", stringify!($variant))
        }
    };
    ($node:expr, $variant:ident{$( $field:ident ),+}) => {
        match $node {
            AstNode::$variant{$( $field ),+} => ($( $field ),+),
            bad => panic!("Tried reading {bad:?} as {}{{{}}}", stringify!($variant), stringify!($pattern))
        }
    };
}

impl AstNode {
    /// ```
    /// Index: expr ⟶ access
    /// ```
    pub fn from_index_node(first: Self) -> Self {
        Self::access(vec![*tuplify!(first, expr)])
    }

    /// ```
    /// DictTail: string expr ⟶ dictionary
    /// ```
    pub fn from_dict_tail(first: Self, second: Self) -> Self {
        Self::dictionary(vec![(tuplify!(first, string), *tuplify!(second, expr))])
    }

    /// ```
    /// Dict: string expr dictionary* ⟶ dictionary
    /// ```
    pub fn from_dict(first: Self, second: Self, third: Self) -> Self {
        let mut pairs = vec![(tuplify!(first, string), *tuplify!(second, expr))];

        for rest in tuplify!(third, multiple).into_iter() {
            pairs.push(tuplify!(rest, dictionary).into_iter().next().unwrap());
        }

        Self::dictionary(pairs)
    }

    /// ```
    /// ParamsTail: name ⟶ parameters
    /// ```
    pub fn from_params_tail(first: Self) -> Self {
        Self::parameters(vec![tuplify!(first, name)])
    }

    /// ```
    /// Params: name parameters* ⟶ parameters
    /// ```
    pub fn from_params(first: Self, second: Self) -> Self {
        let mut names = vec![tuplify!(first, name)];

        for rest in tuplify!(second, multiple).into_iter() {
            names.push(tuplify!(rest, parameters).into_iter().next().unwrap());
        }

        Self::parameters(names)
    }

    /// ```
    /// ListTail: expr
    /// ```
    pub fn from_list_tail(first: Self) -> Self {
        first
    }

    /// ```
    /// List: expr expr* ⟶ expr+
    /// ```
    pub fn from_list(first: Self, second: Self) -> Self {
        let mut items = vec![first];
        items.extend(tuplify!(second, multiple));
        Self::multiple(items)
    }

    /// ```
    /// BracExpr.1: dictionary
    /// ```
    pub fn from_brac_expr_1(first: Self) -> Self {
        first
    }

    /// ```
    /// BracExpr.2: expr+ ⟶ set
    /// ```
    pub fn from_brac_expr_2(first: Self) -> Self {
        Self::set(
            tuplify!(first, multiple)
                .into_iter()
                .map(|e| *tuplify!(e, expr))
                .collect(),
        )
    }

    /// ```
    /// NameExpr.1: empty ⟶ arguments
    ///             expr+ ⟶ arguments
    /// ```
    pub fn from_name_expr_1(first: Self) -> Self {
        match first {
            Self::empty => Self::arguments(Vec::new()),
            Self::multiple(exprs) => {
                Self::arguments(exprs.into_iter().map(|e| *tuplify!(e, expr)).collect())
            }
            bad => panic!("Tried calling from_name_expr_1() with {bad:?}"),
        }
    }

    /// ```
    /// NameExpr.2: empty   ⟶ empty
    ///             access+ ⟶ access
    /// ```
    pub fn from_name_expr_2(first: Self) -> Self {
        match first {
            Self::empty => first,
            Self::multiple(accesses) => Self::access(
                accesses
                    .into_iter()
                    .map(|a| tuplify!(a, access).into_iter().next().unwrap())
                    .collect(),
            ),
            bad => panic!("Tried calling from_name_expr_2() with {bad:?}"),
        }
    }

    /// ```
    /// ExprUnit.1: name arguments ⟶ function_call
    ///             name empty     ⟶ variable
    ///             name access    ⟶ variable
    /// ```
    pub fn from_expr_unit_1(first: Self, second: Self) -> Self {
        match second {
            Self::arguments(args) => Self::function_call {
                function: tuplify!(first, name),
                arguments: args,
            },
            Self::empty => Self::variable {
                identifier: tuplify!(first, name),
                accesses: Vec::new(),
            },
            Self::access(accesses) => Self::variable {
                identifier: tuplify!(first, name),
                accesses,
            },
            bad => panic!("Tried calling from_expr_unit_1() with {bad:?}"),
        }
    }

    /// ```
    /// ExprUnit.2: expr
    /// ```
    pub fn from_expr_unit_2(first: Self) -> Self {
        first
    }

    /// ```
    /// ExprUnit.3: empty ⟶ list
    ///             expr+ ⟶ list
    /// ```
    pub fn from_expr_unit_3(first: Self) -> Self {
        match first {
            Self::empty => Self::list(Vec::new()),
            Self::multiple(exprs) => {
                Self::list(exprs.into_iter().map(|e| *tuplify!(e, expr)).collect())
            }
            bad => panic!("Tried calling from_expr_unit_3() with {bad:?}"),
        }
    }

    /// ```
    /// ExprUnit.4: empty      ⟶ dictionary
    ///             dictionary ⟶ dictionary
    ///             set        ⟶ set
    /// ```
    pub fn from_expr_unit_4(first: Self) -> Self {
        match first {
            Self::empty => Self::dictionary(Vec::new()),
            Self::dictionary(..) => first,
            Self::set(..) => first,
            bad => panic!("Tried calling from_name_expr_4() with {bad:?}"),
        }
    }

    /// ```
    /// ExprUnit.5: string
    /// ```
    pub fn from_expr_unit_5(first: Self) -> Self {
        first
    }

    /// ```
    /// ExprUnit.6: number
    /// ```
    pub fn from_expr_unit_6(first: Self) -> Self {
        first
    }

    /// ```
    /// ExprUnit.7: boolean
    /// ```
    pub fn from_expr_unit_7(first: Self) -> Self {
        first
    }

    /// ```
    /// ExprBinary: op function_call ⟶ binary_op_rhs
    ///             op variable      ⟶ binary_op_rhs
    ///             op expr          ⟶ binary_op_rhs
    ///             op list          ⟶ binary_op_rhs
    ///             op dictionary    ⟶ binary_op_rhs
    ///             op set           ⟶ binary_op_rhs
    ///             op string        ⟶ binary_op_rhs
    ///             op number        ⟶ binary_op_rhs
    ///             op boolean       ⟶ binary_op_rhs
    /// ```
    pub fn from_expr_binary(first: Self, second: Self) -> Self {
        Self::binary_op_rhs {
            operation: tuplify!(first, op),
            rhs: match second {
                Self::function_call { .. }
                | Self::variable { .. }
                | Self::list(..)
                | Self::dictionary(..)
                | Self::set(..)
                | Self::string(..)
                | Self::number(..)
                | Self::boolean(..) => Box::new(OperationTree::Identity(second)),
                Self::expr(op_tree) => op_tree,
                bad => panic!("Tried calling from_expr_binary() with {bad:?}"),
            },
        }
    }

    /// ```
    /// ExprUnary.1: function_call ⟶ expr
    ///              variable      ⟶ expr
    ///              expr          ⟶ expr
    ///              list          ⟶ expr
    ///              dictionary    ⟶ expr
    ///              set           ⟶ expr
    ///              string        ⟶ expr
    ///              number        ⟶ expr
    ///              boolean       ⟶ expr
    /// ```
    pub fn from_expr_unary_1(first: Self) -> Self {
        Self::expr(Box::new(OperationTree::Unary {
            operation: Op::Minus,
            value: match first {
                Self::function_call { .. }
                | Self::variable { .. }
                | Self::list(..)
                | Self::dictionary(..)
                | Self::set(..)
                | Self::string(..)
                | Self::number(..)
                | Self::boolean(..) => Box::new(OperationTree::Identity(first)),
                Self::expr(op_tree) => op_tree,
                bad => panic!("Tried calling from_expr_unary_1() with {bad:?}"),
            },
        }))
    }

    /// ```
    /// ExprUnary.2: function_call ⟶ expr
    ///              variable      ⟶ expr
    ///              expr          ⟶ expr
    ///              list          ⟶ expr
    ///              dictionary    ⟶ expr
    ///              set           ⟶ expr
    ///              string        ⟶ expr
    ///              number        ⟶ expr
    ///              boolean       ⟶ expr
    /// ```
    pub fn from_expr_unary_2(first: Self) -> Self {
        Self::expr(Box::new(OperationTree::Unary {
            operation: Op::Not,
            value: match first {
                Self::function_call { .. }
                | Self::variable { .. }
                | Self::list(..)
                | Self::dictionary(..)
                | Self::set(..)
                | Self::string(..)
                | Self::number(..)
                | Self::boolean(..) => Box::new(OperationTree::Identity(first)),
                Self::expr(op_tree) => op_tree,
                bad => panic!("Tried calling from_expr_unary_2() with {bad:?}"),
            },
        }))
    }

    /// ```
    /// ExprUnary.3: function_call ⟶ expr
    ///              variable      ⟶ expr
    ///              expr          ⟶ expr
    ///              list          ⟶ expr
    ///              dictionary    ⟶ expr
    ///              set           ⟶ expr
    ///              string        ⟶ expr
    ///              number        ⟶ expr
    ///              boolean       ⟶ expr
    /// ```
    pub fn from_expr_unary_3(first: Self) -> Self {
        Self::expr(match first {
            Self::function_call { .. }
            | Self::variable { .. }
            | Self::list(..)
            | Self::dictionary(..)
            | Self::set(..)
            | Self::string(..)
            | Self::number(..)
            | Self::boolean(..) => Box::new(OperationTree::Identity(first)),
            Self::expr(op_tree) => op_tree,
            bad => panic!("Tried calling from_expr_unary_3() with {bad:?}"),
        })
    }

    /// ```
    /// Expr: function_call binary_op_rhs* ⟶ expr
    ///       variable binary_op_rhs*      ⟶ expr
    ///       expr binary_op_rhs*          ⟶ expr
    ///       list binary_op_rhs*          ⟶ expr
    ///       dictionary binary_op_rhs*    ⟶ expr
    ///       set binary_op_rhs*           ⟶ expr
    ///       string binary_op_rhs*        ⟶ expr
    ///       number binary_op_rhs*        ⟶ expr
    ///       boolean binary_op_rhs*       ⟶ expr
    /// ```
    pub fn from_expr(first: Self, second: Self) -> Self {
        fn populate_op_tree(
            top_value: OperationTree,
            mut rhs_chain: std::iter::Rev<<Vec<AstNode> as IntoIterator>::IntoIter>,
        ) -> OperationTree {
            match rhs_chain.next() {
                None => top_value,
                Some(rhs) => {
                    let (operation, right) = tuplify!(rhs, binary_op_rhs { operation, rhs });
                    OperationTree::Binary {
                        operation,
                        left: Box::new(populate_op_tree(top_value, rhs_chain)),
                        right,
                    }
                }
            }
        }

        let chain = tuplify!(second, multiple);

        Self::expr(if chain.is_empty() {
            match first {
                Self::function_call { .. }
                | Self::variable { .. }
                | Self::list(..)
                | Self::dictionary(..)
                | Self::set(..)
                | Self::string(..)
                | Self::number(..)
                | Self::boolean(..) => Box::new(OperationTree::Identity(first)),
                Self::expr(op_tree) => op_tree,
                bad => panic!("Tried calling from_expr() with {bad:?}"),
            }
        } else {
            let root_value = match first {
                Self::function_call { .. }
                | Self::variable { .. }
                | Self::list(..)
                | Self::dictionary(..)
                | Self::set(..)
                | Self::string(..)
                | Self::number(..)
                | Self::boolean(..) => OperationTree::Identity(first),
                Self::expr(op_tree) => *op_tree,
                bad => panic!("Tried calling from_expr() with {bad:?}"),
            };

            Box::new(populate_op_tree(root_value, chain.into_iter().rev()))
        })
    }

    /// ```
    /// SideEffect.1: empty ⟶ arguments
    ///               expr+ ⟶ arguments
    /// ```
    pub fn from_side_effect_1(first: Self) -> Self {
        match first {
            Self::empty => Self::arguments(Vec::new()),
            Self::multiple(exprs) => {
                Self::arguments(exprs.into_iter().map(|e| *tuplify!(e, expr)).collect())
            }
            bad => panic!("Tried calling from_side_effect_1() with {bad:?}"),
        }
    }

    /// ```
    /// SideEffect.2: access* asop expr ⟶ assign_op_rhs
    /// ```
    pub fn from_side_effect_2(first: Self, second: Self, third: Self) -> Self {
        let accesses = tuplify!(first, multiple)
            .into_iter()
            .map(|a| tuplify!(a, access).into_iter().next().unwrap())
            .collect();
        Self::assign_op_rhs {
            accesses,
            asop: tuplify!(second, asop),
            rhs: tuplify!(third, expr),
        }
    }

    /// ```
    /// Body.1: (empty|if_stmt|while_loop|for_loop|continue|break|return_stmt|function_def|function_call|assign_op)* ⟶ block
    /// ```
    pub fn from_body_1(first: Self) -> Self {
        Self::block(tuplify!(first, multiple))
    }

    /// ```
    /// Body.2: expr ⟶ return_stmt
    /// ```
    pub fn from_body_2(first: Self) -> Self {
        Self::return_stmt(Some(tuplify!(first, expr)))
    }

    /// ```
    /// Result.1: (empty|if_stmt|while_loop|for_loop|continue|break|return_stmt|function_def|function_call|assign_op)+ ⟶ block
    /// ```
    pub fn from_result_1(first: Self) -> Self {
        Self::block(tuplify!(first, multiple))
    }

    /// ```
    /// Result.2: name arguments     ⟶ function_call
    ///           name assign_op_rhs ⟶ assign_op
    /// ```
    pub fn from_result_2(first: Self, second: Self) -> Self {
        match second {
            Self::arguments(args) => Self::function_call {
                function: tuplify!(first, name),
                arguments: args,
            },
            Self::assign_op_rhs {
                accesses,
                asop,
                rhs,
            } => Self::assign_op {
                variable: tuplify!(first, name),
                accesses,
                asop,
                value: rhs,
            },
            bad => panic!("Tried calling from_result_2() with {bad:?}"),
        }
    }

    /// ```
    /// Unit.1: expr function_call ⟶ if_stmt
    ///         expr assign_op     ⟶ if_stmt
    ///         expr block         ⟶ if_stmt
    /// ```
    pub fn from_unit_1(first: Self, second: Self) -> Self {
        Self::if_stmt {
            condition: tuplify!(first, expr),
            then: Box::new(second),
        }
    }

    /// ```
    /// Unit.2: expr function_call ⟶ while_loop
    ///         expr assign_op     ⟶ while_loop
    ///         expr block         ⟶ while_loop
    /// ```
    pub fn from_unit_2(first: Self, second: Self) -> Self {
        Self::while_loop {
            condition: tuplify!(first, expr),
            body: Box::new(second),
        }
    }

    /// ```
    /// Unit.3: name expr function_call ⟶ for_loop
    ///         name expr assign_op     ⟶ for_loop
    ///         name expr block         ⟶ for_loop
    /// ```
    pub fn from_unit_3(first: Self, second: Self, third: Self) -> Self {
        Self::for_loop {
            loop_variable: tuplify!(first, name),
            iterator: tuplify!(second, expr),
            body: Box::new(third),
        }
    }

    /// ```
    /// Unit.4: continue
    /// ```
    pub fn from_unit_4(first: Self) -> Self {
        first
    }

    /// ```
    /// Unit.5: break
    /// ```
    pub fn from_unit_5(first: Self) -> Self {
        first
    }

    /// ```
    /// Unit.6: empty ⟶ return_stmt
    ///         expr  ⟶ return_stmt
    /// ```
    pub fn from_unit_6(first: Self) -> Self {
        match first {
            Self::empty => Self::return_stmt(None),
            Self::expr(op_tree) => Self::return_stmt(Some(op_tree)),
            bad => panic!("Tried calling from_unit_6() with {bad:?}"),
        }
    }

    /// ```
    /// Unit.7: name parameters block       ⟶ function_def
    ///         name parameters return_stmt ⟶ function_def
    /// ```
    pub fn from_unit_7(first: Self, second: Self, third: Self) -> Self {
        Self::function_def {
            identifier: tuplify!(first, name),
            parameters: tuplify!(second, parameters),
            body: Box::new(third),
        }
    }

    /// ```
    /// Unit.8: name arguments     ⟶ function_call
    ///         name assign_op_rhs ⟶ assign_op
    /// ```
    pub fn from_unit_8(first: Self, second: Self) -> Self {
        match second {
            Self::arguments(args) => Self::function_call {
                function: tuplify!(first, name),
                arguments: args,
            },
            Self::assign_op_rhs {
                accesses,
                asop,
                rhs,
            } => Self::assign_op {
                variable: tuplify!(first, name),
                accesses,
                asop,
                value: rhs,
            },
            bad => panic!("Tried calling from_unit_8() with {bad:?}"),
        }
    }

    /// ```
    /// Scoped.1: empty
    /// ```
    pub fn from_scoped_1(first: Self) -> Self {
        first
    }

    /// ```
    /// Scoped.2: if_stmt
    ///           while_loop
    ///           for_loop
    ///           continue
    ///           break
    ///           return_stmt
    ///           function_def
    ///           function_call
    ///           assign_op
    /// ```
    pub fn from_scoped_2(first: Self) -> Self {
        first
    }

    /// ```
    /// Program.1: empty
    /// ```
    pub fn from_program_1(first: Self) -> Self {
        first
    }

    /// ```
    /// Program.2: (empty|if_stmt|while_loop|for_loop|continue|break|return_stmt|function_def|function_call|assign_op)* ⟶ block
    /// ```
    pub fn from_program_2(first: Self) -> Self {
        Self::block(tuplify!(first, multiple))
    }
}
