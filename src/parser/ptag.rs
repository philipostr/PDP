#![allow(dead_code)]

use super::building_blocks::*;
use super::markers::*;

#[macro_export]
macro_rules! identity_safe_ast {
    () => {
        $crate::parser::ptag::AstNode::function_call { .. }
            | $crate::parser::ptag::AstNode::variable { .. }
            | $crate::parser::ptag::AstNode::list(..)
            | $crate::parser::ptag::AstNode::dictionary(..)
            | $crate::parser::ptag::AstNode::set(..)
            | $crate::parser::ptag::AstNode::string(..)
            | $crate::parser::ptag::AstNode::number(..)
            | $crate::parser::ptag::AstNode::boolean(..)
    };
}

#[macro_export]
macro_rules! non_identity_ast {
    () => {
        AstNode::op(_)
            | AstNode::asop(_)
            | AstNode::keyword(_)
            | AstNode::name(_)
            | AstNode::bracket(_)
            | AstNode::misc(_)
            | AstNode::multiple(_)
            | AstNode::access(_)
            | AstNode::arguments(_)
            | AstNode::assign_op { .. }
            | AstNode::assign_op_rhs { .. }
            | AstNode::binary_op_rhs { .. }
            | AstNode::block(_)
            | AstNode::r#break
            | AstNode::r#continue
            | AstNode::empty
            | AstNode::expr(_)
            | AstNode::for_loop { .. }
            | AstNode::function_def { .. }
            | AstNode::if_stmt { .. }
            | AstNode::parameters(_)
            | AstNode::return_stmt(_)
            | AstNode::while_loop { .. }
    };
}

#[derive(Debug)]
pub enum OperationTree {
    Unary {
        operation: MarkedOp,
        value: Box<MarkedOperationTree>,
    },
    Binary {
        operation: MarkedOp,
        left: Box<MarkedOperationTree>,
        right: Box<MarkedOperationTree>,
    },
    Identity(MarkedAstNode),
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AstNode {
    // token nodes
    op(MarkedOp),
    asop(MarkedAsop),
    keyword(MarkedKeyword),
    name(MarkedString),
    bracket(MarkedComponent<char>),
    string(MarkedString),
    number(MarkedNumber),
    boolean(MarkedBoolean),
    misc(MarkedComponent<char>),

    // meta nodes
    multiple(Vec<MarkedAstNode>),

    // meaningful nodes
    access(Vec<MarkedOperationTree>),
    arguments(Vec<MarkedOperationTree>),
    assign_op {
        variable: MarkedString,
        accesses: Vec<MarkedOperationTree>,
        asop: MarkedAsop,
        value: Box<MarkedOperationTree>,
    },
    assign_op_rhs {
        accesses: Vec<MarkedOperationTree>,
        asop: MarkedAsop,
        rhs: Box<MarkedOperationTree>,
    },
    binary_op_rhs {
        operation: MarkedOp,
        rhs: Box<MarkedOperationTree>,
    },
    block(Vec<MarkedAstNode>),
    r#break,
    r#continue,
    dictionary(Vec<(MarkedString, MarkedOperationTree)>),
    empty,
    expr(Box<MarkedOperationTree>),
    for_loop {
        loop_variable: MarkedString,
        iterator: Box<MarkedOperationTree>,
        body: Box<MarkedAstNode>,
    },
    function_call {
        function: MarkedString,
        arguments: Vec<MarkedOperationTree>,
    },
    function_def {
        identifier: MarkedString,
        parameters: Vec<MarkedString>,
        body: Box<MarkedAstNode>,
    },
    if_stmt {
        condition: Box<MarkedOperationTree>,
        then: Box<MarkedAstNode>,
    },
    list(Vec<MarkedOperationTree>),
    parameters(Vec<MarkedString>),
    return_stmt(Option<Box<MarkedOperationTree>>),
    set(Vec<MarkedOperationTree>),
    variable {
        identifier: MarkedString,
        accesses: Vec<MarkedOperationTree>,
    },
    while_loop {
        condition: Box<MarkedOperationTree>,
        body: Box<MarkedAstNode>,
    },
}

macro_rules! tuplify {
    ($node:expr, $variant:ident) => {
        match $node.comp {
            AstNode::$variant(x) => x,
            bad => panic!("Tried reading {bad:?} as {}", stringify!($variant))
        }
    };
    ($node:expr, $variant:ident{$( $field:ident ),+}) => {
        match $node.comp {
            AstNode::$variant{$( $field ),+} => ($( $field ),+),
            bad => panic!("Tried reading {bad:?} as {}{{{}}}", stringify!($variant), stringify!($pattern))
        }
    };
}

impl AstNode {
    /// ```
    /// Index: expr ⟶ access
    /// ```
    pub fn from_index_node(first: MarkedAstNode) -> MarkedAstNode {
        MarkedAstNode::new(Self::access(vec![*tuplify!(first, expr)]), first.mark)
    }

    /// ```
    /// DictTail: string expr ⟶ dictionary
    /// ```
    pub fn from_dict_tail(first: MarkedAstNode, second: MarkedAstNode) -> MarkedAstNode {
        MarkedAstNode::new(
            Self::dictionary(vec![(tuplify!(first, string), *tuplify!(second, expr))]),
            first.mark,
        )
    }

    /// ```
    /// Dict: string expr dictionary* ⟶ dictionary
    /// ```
    pub fn from_dict(
        first: MarkedAstNode,
        second: MarkedAstNode,
        third: MarkedAstNode,
    ) -> MarkedAstNode {
        let mut pairs = vec![(tuplify!(first, string), *tuplify!(second, expr))];

        for rest in tuplify!(third, multiple).into_iter() {
            pairs.push(tuplify!(rest, dictionary).into_iter().next().unwrap());
        }

        MarkedAstNode::new(Self::dictionary(pairs), first.mark)
    }

    /// ```
    /// ParamsTail: name ⟶ parameters
    /// ```
    pub fn from_params_tail(first: MarkedAstNode) -> MarkedAstNode {
        MarkedAstNode::new(Self::parameters(vec![tuplify!(first, name)]), first.mark)
    }

    /// ```
    /// Params: name parameters* ⟶ parameters
    /// ```
    pub fn from_params(first: MarkedAstNode, second: MarkedAstNode) -> MarkedAstNode {
        let mut names = vec![tuplify!(first, name)];

        for rest in tuplify!(second, multiple).into_iter() {
            names.push(tuplify!(rest, parameters).into_iter().next().unwrap());
        }

        MarkedAstNode::new(Self::parameters(names), first.mark)
    }

    /// ```
    /// ListTail: expr
    /// ```
    pub fn from_list_tail(first: MarkedAstNode) -> MarkedAstNode {
        first
    }

    /// ```
    /// List: expr expr* ⟶ expr+
    /// ```
    pub fn from_list(first: MarkedAstNode, second: MarkedAstNode) -> MarkedAstNode {
        let mark = first.mark;
        let mut items = vec![first];
        items.extend(tuplify!(second, multiple));
        MarkedAstNode::new(Self::multiple(items), mark)
    }

    /// ```
    /// BracExpr.1: dictionary
    /// ```
    pub fn from_brac_expr_1(first: MarkedAstNode) -> MarkedAstNode {
        first
    }

    /// ```
    /// BracExpr.2: expr+ ⟶ set
    /// ```
    pub fn from_brac_expr_2(first: MarkedAstNode) -> MarkedAstNode {
        MarkedAstNode::new(
            Self::set(
                tuplify!(first, multiple)
                    .into_iter()
                    .map(|e| *tuplify!(e, expr))
                    .collect(),
            ),
            first.mark,
        )
    }

    /// ```
    /// NameExpr.1: empty ⟶ arguments
    ///             expr+ ⟶ arguments
    /// ```
    pub fn from_name_expr_1(first: MarkedAstNode) -> MarkedAstNode {
        match first.comp {
            Self::empty => MarkedAstNode::new(Self::arguments(Vec::new()), first.mark),
            Self::multiple(exprs) => MarkedAstNode::new(
                Self::arguments(exprs.into_iter().map(|e| *tuplify!(e, expr)).collect()),
                first.mark,
            ),
            bad => panic!("Tried calling from_name_expr_1() with {bad:?}"),
        }
    }

    /// ```
    /// NameExpr.2: empty   ⟶ empty
    ///             access+ ⟶ access
    /// ```
    pub fn from_name_expr_2(first: MarkedAstNode) -> MarkedAstNode {
        match first.comp {
            Self::empty => first,
            Self::multiple(accesses) => MarkedAstNode::new(
                Self::access(
                    accesses
                        .into_iter()
                        .map(|a| tuplify!(a, access).into_iter().next().unwrap())
                        .collect(),
                ),
                first.mark,
            ),
            bad => panic!("Tried calling from_name_expr_2() with {bad:?}"),
        }
    }

    /// ```
    /// ExprUnit.1: name arguments ⟶ function_call
    ///             name empty     ⟶ variable
    ///             name access    ⟶ variable
    /// ```
    pub fn from_expr_unit_1(first: MarkedAstNode, second: MarkedAstNode) -> MarkedAstNode {
        match second.comp {
            Self::arguments(args) => MarkedAstNode::new(
                Self::function_call {
                    function: tuplify!(first, name),
                    arguments: args,
                },
                first.mark,
            ),
            Self::empty => MarkedAstNode::new(
                Self::variable {
                    identifier: tuplify!(first, name),
                    accesses: Vec::new(),
                },
                first.mark,
            ),
            Self::access(accesses) => MarkedAstNode::new(
                Self::variable {
                    identifier: tuplify!(first, name),
                    accesses,
                },
                first.mark,
            ),
            bad => panic!("Tried calling from_expr_unit_1() with {bad:?}"),
        }
    }

    /// ```
    /// ExprUnit.2: expr
    /// ```
    pub fn from_expr_unit_2(first: MarkedAstNode) -> MarkedAstNode {
        first
    }

    /// ```
    /// ExprUnit.3: empty ⟶ list
    ///             expr+ ⟶ list
    /// ```
    pub fn from_expr_unit_3(first: MarkedAstNode) -> MarkedAstNode {
        match first.comp {
            Self::empty => MarkedAstNode::new(Self::list(Vec::new()), first.mark),
            Self::multiple(exprs) => MarkedAstNode::new(
                Self::list(exprs.into_iter().map(|e| *tuplify!(e, expr)).collect()),
                first.mark,
            ),
            bad => panic!("Tried calling from_expr_unit_3() with {bad:?}"),
        }
    }

    /// ```
    /// ExprUnit.4: empty      ⟶ dictionary
    ///             dictionary ⟶ dictionary
    ///             set        ⟶ set
    /// ```
    pub fn from_expr_unit_4(first: MarkedAstNode) -> MarkedAstNode {
        match first.comp {
            Self::empty => MarkedAstNode::new(Self::dictionary(Vec::new()), first.mark),
            Self::dictionary(..) => first,
            Self::set(..) => first,
            bad => panic!("Tried calling from_name_expr_4() with {bad:?}"),
        }
    }

    /// ```
    /// ExprUnit.5: string
    /// ```
    pub fn from_expr_unit_5(first: MarkedAstNode) -> MarkedAstNode {
        first
    }

    /// ```
    /// ExprUnit.6: number
    /// ```
    pub fn from_expr_unit_6(first: MarkedAstNode) -> MarkedAstNode {
        first
    }

    /// ```
    /// ExprUnit.7: boolean
    /// ```
    pub fn from_expr_unit_7(first: MarkedAstNode) -> MarkedAstNode {
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
    pub fn from_expr_binary(first: MarkedAstNode, second: MarkedAstNode) -> MarkedAstNode {
        let second_mark = second.mark;
        MarkedAstNode::new(
            Self::binary_op_rhs {
                operation: tuplify!(first, op),
                rhs: match second.comp {
                    identity_safe_ast!() => Box::new(MarkedOperationTree::new(
                        OperationTree::Identity(second),
                        second_mark,
                    )),
                    Self::expr(op_tree) => op_tree,
                    bad => panic!("Tried calling from_expr_binary() with {bad:?}"),
                },
            },
            first.mark,
        )
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
    pub fn from_expr_unary_1(first: MarkedAstNode) -> MarkedAstNode {
        let mark = first.mark;
        MarkedAstNode::new(
            Self::expr(Box::new(MarkedOperationTree::new(
                OperationTree::Unary {
                    operation: MarkedOp::new(Op::Minus, mark),
                    value: match first.comp {
                        identity_safe_ast!() => Box::new(MarkedOperationTree::new(
                            OperationTree::Identity(first),
                            mark,
                        )),
                        Self::expr(op_tree) => op_tree,
                        bad => panic!("Tried calling from_expr_unary_1() with {bad:?}"),
                    },
                },
                mark,
            ))),
            mark,
        )
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
    pub fn from_expr_unary_2(first: MarkedAstNode) -> MarkedAstNode {
        let mark = first.mark;
        MarkedAstNode::new(
            Self::expr(Box::new(MarkedOperationTree::new(
                OperationTree::Unary {
                    operation: MarkedOp::new(Op::Not, mark),
                    value: match first.comp {
                        identity_safe_ast!() => Box::new(MarkedOperationTree::new(
                            OperationTree::Identity(first),
                            mark,
                        )),
                        Self::expr(op_tree) => op_tree,
                        bad => panic!("Tried calling from_expr_unary_2() with {bad:?}"),
                    },
                },
                mark,
            ))),
            mark,
        )
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
    pub fn from_expr_unary_3(first: MarkedAstNode) -> MarkedAstNode {
        let mark = first.mark;
        MarkedAstNode::new(
            Self::expr(match first.comp {
                identity_safe_ast!() => Box::new(MarkedOperationTree::new(
                    OperationTree::Identity(first),
                    mark,
                )),
                Self::expr(op_tree) => op_tree,
                bad => panic!("Tried calling from_expr_unary_3() with {bad:?}"),
            }),
            mark,
        )
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
    pub fn from_expr(first: MarkedAstNode, second: MarkedAstNode) -> MarkedAstNode {
        fn populate_op_tree(
            top_value: MarkedOperationTree,
            mut rhs_chain: std::iter::Rev<<Vec<MarkedAstNode> as IntoIterator>::IntoIter>,
        ) -> MarkedOperationTree {
            let mark = top_value.mark;
            match rhs_chain.next() {
                None => top_value,
                Some(rhs) => {
                    let (operation, right) = tuplify!(rhs, binary_op_rhs { operation, rhs });
                    MarkedOperationTree::new(
                        OperationTree::Binary {
                            operation,
                            left: Box::new(populate_op_tree(top_value, rhs_chain)),
                            right,
                        },
                        mark,
                    )
                }
            }
        }

        let chain = tuplify!(second, multiple);
        let first_mark = first.mark;

        MarkedAstNode::new(
            Self::expr(if chain.is_empty() {
                match first.comp {
                    identity_safe_ast!() => Box::new(MarkedOperationTree::new(
                        OperationTree::Identity(first),
                        first_mark,
                    )),
                    Self::expr(op_tree) => op_tree,
                    bad => panic!("Tried calling from_expr() with {bad:?}"),
                }
            } else {
                let root_value = match first.comp {
                    identity_safe_ast!() => {
                        MarkedOperationTree::new(OperationTree::Identity(first), first_mark)
                    }
                    Self::expr(op_tree) => *op_tree,
                    bad => panic!("Tried calling from_expr() with {bad:?}"),
                };

                Box::new(populate_op_tree(root_value, chain.into_iter().rev()))
            }),
            first_mark,
        )
    }

    /// ```
    /// SideEffect.1: empty ⟶ arguments
    ///               expr+ ⟶ arguments
    /// ```
    pub fn from_side_effect_1(first: MarkedAstNode) -> MarkedAstNode {
        match first.comp {
            Self::empty => MarkedAstNode::new(Self::arguments(Vec::new()), first.mark),
            Self::multiple(exprs) => MarkedAstNode::new(
                Self::arguments(exprs.into_iter().map(|e| *tuplify!(e, expr)).collect()),
                first.mark,
            ),
            bad => panic!("Tried calling from_side_effect_1() with {bad:?}"),
        }
    }

    /// ```
    /// SideEffect.2: access* asop expr ⟶ assign_op_rhs
    /// ```
    pub fn from_side_effect_2(
        first: MarkedAstNode,
        second: MarkedAstNode,
        third: MarkedAstNode,
    ) -> MarkedAstNode {
        let accesses = tuplify!(first, multiple)
            .into_iter()
            .map(|a| tuplify!(a, access).into_iter().next().unwrap())
            .collect();
        MarkedAstNode::new(
            Self::assign_op_rhs {
                accesses,
                asop: tuplify!(second, asop),
                rhs: tuplify!(third, expr),
            },
            first.mark,
        )
    }

    /// ```
    /// Body.1: (empty|if_stmt|while_loop|for_loop|continue|break|return_stmt|function_def|function_call|assign_op)* ⟶ block
    /// ```
    pub fn from_body_1(first: MarkedAstNode) -> MarkedAstNode {
        MarkedAstNode::new(Self::block(tuplify!(first, multiple)), first.mark)
    }

    /// ```
    /// Body.2: expr ⟶ return_stmt
    /// ```
    pub fn from_body_2(first: MarkedAstNode) -> MarkedAstNode {
        MarkedAstNode::new(Self::return_stmt(Some(tuplify!(first, expr))), first.mark)
    }

    /// ```
    /// Result.1: (empty|if_stmt|while_loop|for_loop|continue|break|return_stmt|function_def|function_call|assign_op)+ ⟶ block
    /// ```
    pub fn from_result_1(first: MarkedAstNode) -> MarkedAstNode {
        MarkedAstNode::new(Self::block(tuplify!(first, multiple)), first.mark)
    }

    /// ```
    /// Result.2: name arguments     ⟶ function_call
    ///           name assign_op_rhs ⟶ assign_op
    /// ```
    pub fn from_result_2(first: MarkedAstNode, second: MarkedAstNode) -> MarkedAstNode {
        match second.comp {
            Self::arguments(args) => MarkedAstNode::new(
                Self::function_call {
                    function: tuplify!(first, name),
                    arguments: args,
                },
                first.mark,
            ),
            Self::assign_op_rhs {
                accesses,
                asop,
                rhs,
            } => MarkedAstNode::new(
                Self::assign_op {
                    variable: tuplify!(first, name),
                    accesses,
                    asop,
                    value: rhs,
                },
                first.mark,
            ),
            bad => panic!("Tried calling from_result_2() with {bad:?}"),
        }
    }

    /// ```
    /// Unit.1: expr function_call ⟶ if_stmt
    ///         expr assign_op     ⟶ if_stmt
    ///         expr block         ⟶ if_stmt
    /// ```
    pub fn from_unit_1(first: MarkedAstNode, second: MarkedAstNode) -> MarkedAstNode {
        MarkedAstNode::new(
            Self::if_stmt {
                condition: tuplify!(first, expr),
                then: Box::new(second),
            },
            first.mark,
        )
    }

    /// ```
    /// Unit.2: expr function_call ⟶ while_loop
    ///         expr assign_op     ⟶ while_loop
    ///         expr block         ⟶ while_loop
    /// ```
    pub fn from_unit_2(first: MarkedAstNode, second: MarkedAstNode) -> MarkedAstNode {
        MarkedAstNode::new(
            Self::while_loop {
                condition: tuplify!(first, expr),
                body: Box::new(second),
            },
            first.mark,
        )
    }

    /// ```
    /// Unit.3: name expr function_call ⟶ for_loop
    ///         name expr assign_op     ⟶ for_loop
    ///         name expr block         ⟶ for_loop
    /// ```
    pub fn from_unit_3(
        first: MarkedAstNode,
        second: MarkedAstNode,
        third: MarkedAstNode,
    ) -> MarkedAstNode {
        MarkedAstNode::new(
            Self::for_loop {
                loop_variable: tuplify!(first, name),
                iterator: tuplify!(second, expr),
                body: Box::new(third),
            },
            first.mark,
        )
    }

    /// ```
    /// Unit.4: continue
    /// ```
    pub fn from_unit_4(first: MarkedAstNode) -> MarkedAstNode {
        first
    }

    /// ```
    /// Unit.5: break
    /// ```
    pub fn from_unit_5(first: MarkedAstNode) -> MarkedAstNode {
        first
    }

    /// ```
    /// Unit.6: empty ⟶ return_stmt
    ///         expr  ⟶ return_stmt
    /// ```
    pub fn from_unit_6(first: MarkedAstNode) -> MarkedAstNode {
        match first.comp {
            Self::empty => MarkedAstNode::new(Self::return_stmt(None), first.mark),
            Self::expr(op_tree) => MarkedAstNode::new(Self::return_stmt(Some(op_tree)), first.mark),
            bad => panic!("Tried calling from_unit_6() with {bad:?}"),
        }
    }

    /// ```
    /// Unit.7: name parameters block       ⟶ function_def
    ///         name parameters return_stmt ⟶ function_def
    /// ```
    pub fn from_unit_7(
        first: MarkedAstNode,
        second: MarkedAstNode,
        third: MarkedAstNode,
    ) -> MarkedAstNode {
        MarkedAstNode::new(
            Self::function_def {
                identifier: tuplify!(first, name),
                parameters: match second.comp {
                    Self::parameters(params) => params,
                    Self::empty => Vec::new(),
                    bad => panic!("Tried calling from_unit_7() with {bad:?}"),
                },
                body: Box::new(third),
            },
            first.mark,
        )
    }

    /// ```
    /// Unit.8: name arguments     ⟶ function_call
    ///         name assign_op_rhs ⟶ assign_op
    /// ```
    pub fn from_unit_8(first: MarkedAstNode, second: MarkedAstNode) -> MarkedAstNode {
        match second.comp {
            Self::arguments(args) => MarkedAstNode::new(
                Self::function_call {
                    function: tuplify!(first, name),
                    arguments: args,
                },
                first.mark,
            ),
            Self::assign_op_rhs {
                accesses,
                asop,
                rhs,
            } => MarkedAstNode::new(
                Self::assign_op {
                    variable: tuplify!(first, name),
                    accesses,
                    asop,
                    value: rhs,
                },
                first.mark,
            ),
            bad => panic!("Tried calling from_unit_8() with {bad:?}"),
        }
    }

    /// ```
    /// Scoped.1: empty
    /// ```
    pub fn from_scoped_1(first: MarkedAstNode) -> MarkedAstNode {
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
    pub fn from_scoped_2(first: MarkedAstNode) -> MarkedAstNode {
        first
    }

    /// ```
    /// Program.1: empty
    /// ```
    pub fn from_program_1(first: MarkedAstNode) -> MarkedAstNode {
        first
    }

    /// ```
    /// Program.2: (empty|if_stmt|while_loop|for_loop|continue|break|return_stmt|function_def|function_call|assign_op)* ⟶ block
    /// ```
    pub fn from_program_2(first: MarkedAstNode) -> MarkedAstNode {
        MarkedAstNode::new(Self::block(tuplify!(first, multiple)), first.mark)
    }
}
