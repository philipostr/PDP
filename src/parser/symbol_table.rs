use indexmap::IndexMap;
use log::{debug, trace};

use crate::non_identity_ast;
use crate::parser::ParseError;
use crate::parser::building_blocks::Asop;
use crate::parser::markers::*;
use crate::parser::ptag::{AstNode, OperationTree};

use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone)]
pub struct SymbolTable {
    local_vars: Vec<MarkedString>,
    cell_vars: Vec<MarkedString>,
    free_vars: Vec<MarkedString>,
    /// Just here for verification and debugging
    #[allow(dead_code)]
    global_accesses: Vec<MarkedString>,
    children: Vec<Self>,
}

#[derive(Debug)]
enum VarClassification {
    Local,
    Free,
    Cell,
    Global,

    /* Not the final classification evaluation */
    Read,
}

struct ScopeEnv {
    vars: Rc<RefCell<IndexMap<MarkedString, VarClassification>>>,
    parent: Option<Rc<Self>>,
}

impl ScopeEnv {
    fn new(
        vars: Rc<RefCell<IndexMap<MarkedString, VarClassification>>>,
        parent: Option<Rc<Self>>,
    ) -> Self {
        Self { vars, parent }
    }

    fn find_and_promote(&self, var: &MarkedString) -> bool {
        let Some(parent) = &self.parent else {
            // If we're at module level, we consider all variables (even if they exist) as "not found"
            // so the caller can assume the variable as a global access
            return false;
        };

        let mut vars = self.vars.borrow_mut();
        if let Some(var_in_scope) = vars.get_mut(var) {
            if matches!(var_in_scope, VarClassification::Local) {
                *var_in_scope = VarClassification::Cell;
            }
            return true;
        }

        parent.find_and_promote(var)
    }
}

impl SymbolTable {
    pub fn from_root_ast(scope: &MarkedAstNode) -> Result<Self, ParseError> {
        Self::from_scope_ast(scope, None)
    }

    fn from_scope_ast(
        scope: &MarkedAstNode,
        parent_env: Option<Rc<ScopeEnv>>,
    ) -> Result<Self, ParseError> {
        debug!("SymbolTable::from_scope_ast() started");

        // Find and classify all vars/functions in the direct scope
        let mut inner_scopes = Vec::new();
        let vars = Rc::new(RefCell::new(Self::find_vars(scope, &mut inner_scopes)?));

        // Clarify initial reads and locals
        for (identifier, classification) in vars.borrow_mut().iter_mut() {
            match classification {
                VarClassification::Read => {
                    if let Some(parent_env) = &parent_env {
                        *classification = if parent_env.find_and_promote(identifier) {
                            // The variable was found in a higher scope, and was promoted to cell there
                            VarClassification::Free
                        } else {
                            // The variable was not found, so it must be assumed global (dynamic at runtime)
                            VarClassification::Global
                        };
                    } else {
                        // If we're at the module level, all reads are global
                        *classification = VarClassification::Global;
                    }
                }
                VarClassification::Local if parent_env.is_none() => {
                    // We're in the module level, so locals are actually globals
                    *classification = VarClassification::Global;
                }
                _ => {}
            }
        }

        // Recursively build the inner scopes' symbol tables
        let env = Rc::new(ScopeEnv::new(vars.clone(), parent_env));
        let mut child_tables = Vec::new();
        for inner_scope in inner_scopes {
            child_tables.push(Self::from_scope_ast(inner_scope, Some(env.clone()))?);
        }

        // Compile the found variables into the symbol table
        let mut local_vars = Vec::new();
        let mut cell_vars = Vec::new();
        let mut free_vars = Vec::new();
        let mut global_accesses = Vec::new();
        for (identifier, classification) in vars.take() {
            match classification {
                VarClassification::Local => local_vars.push(identifier),
                VarClassification::Free => free_vars.push(identifier),
                VarClassification::Cell => cell_vars.push(identifier),
                VarClassification::Global => global_accesses.push(identifier),
                _ => panic!(
                    "Non-finalized variable classification: {identifier} ({classification:?})"
                ),
            }
        }

        debug!("SymbolTable::from_scope_ast() ended");
        Ok(SymbolTable {
            local_vars,
            cell_vars,
            free_vars,
            global_accesses,
            children: child_tables,
        })
    }

    fn find_vars<'a>(
        root_node: &'a MarkedAstNode,
        inner_scopes: &mut Vec<&'a MarkedAstNode>,
    ) -> Result<IndexMap<MarkedString, VarClassification>, ParseError> {
        let mut analysis_node = root_node;
        let mut result = IndexMap::new();

        // Handle parameters as local variables when dealing with a function definition
        if let AstNode::function_def {
            parameters, body, ..
        } = &root_node.comp
        {
            for param in parameters {
                result.insert(param.clone(), VarClassification::Local);
            }
            analysis_node = body;
        }
        Self::find_vars_ast(analysis_node, &mut result, inner_scopes)?;
        Ok(result)
    }

    fn find_vars_ast<'a>(
        node: &'a MarkedAstNode,
        vars: &mut IndexMap<MarkedString, VarClassification>,
        inner_scopes: &mut Vec<&'a MarkedAstNode>,
    ) -> Result<(), ParseError> {
        match &node.comp {
            AstNode::empty => {
                trace!("Called find_vars_ast() on an empty");
            }
            AstNode::block(children) => {
                trace!("Called find_vars_ast() on a block");
                for child in children {
                    Self::find_vars_ast(child, vars, inner_scopes)?;
                }
            }
            AstNode::if_stmt { condition, then } => {
                trace!("Called find_vars_ast() on an if_stmt");
                Self::find_vars_op(condition, vars, inner_scopes)?;
                Self::find_vars_ast(then, vars, inner_scopes)?;
            }
            AstNode::while_loop { condition, body } => {
                trace!("Called find_vars_ast() on a while_loop");
                Self::find_vars_op(condition, vars, inner_scopes)?;
                Self::find_vars_ast(body, vars, inner_scopes)?;
            }
            AstNode::for_loop {
                loop_variable,
                iterator,
                body,
            } => {
                trace!("Called find_vars_ast() on a for_loop");
                Self::put_local(loop_variable, vars)?;
                Self::find_vars_op(iterator, vars, inner_scopes)?;
                Self::find_vars_ast(body, vars, inner_scopes)?;
            }
            AstNode::r#continue => {
                trace!("Called find_vars_ast() on a continue");
            }
            AstNode::r#break => {
                trace!("Called find_vars_ast() on a break");
            }
            AstNode::return_stmt(value) => {
                trace!("Called find_vars_ast() on a return_stmt");
                if let Some(value) = value {
                    Self::find_vars_op(value, vars, inner_scopes)?;
                }
            }
            AstNode::function_def { identifier, .. } => {
                trace!("Called find_vars_ast() on a function_def");
                inner_scopes.push(node);
                Self::put_local(identifier, vars)?;
            }
            AstNode::function_call {
                function,
                arguments,
            } => {
                trace!("Called find_vars_ast() on a function_call");
                Self::put_read(function, vars);
                for arg in arguments {
                    Self::find_vars_op(arg, vars, inner_scopes)?;
                }
            }
            AstNode::assign_op {
                variable,
                asop,
                value,
                ..
            } => {
                trace!("Called find_vars_ast() on an assign_op");
                // Custom `put_local()` implementation because all untrivial asops are read AND write,
                // so the var must have been evaluated as local ALREADY
                match vars.get(variable) {
                    Some(VarClassification::Read) => {
                        return Err(ParseError::marked(
                            &format!("local variable '{variable}' referenced before assignment"),
                            variable.mark.row,
                            variable.mark.col,
                        ));
                    }
                    Some(VarClassification::Local) => {}
                    Some(_) => unreachable!(),
                    None => {
                        if !matches!(asop.comp, Asop::Assign) {
                            return Err(ParseError::marked(
                                &format!(
                                    "local variable '{variable}' referenced before assignment"
                                ),
                                variable.mark.row,
                                variable.mark.col,
                            ));
                        }
                        vars.insert(variable.clone(), VarClassification::Local);
                    }
                }

                Self::find_vars_op(value, vars, inner_scopes)?;
            }
            _ => {
                // Find vars in all the ast nodes that directly mention them (identity operations)
                match &node.comp {
                    AstNode::function_call { .. } => {
                        // This is above
                        unreachable!()
                    }
                    AstNode::variable {
                        identifier,
                        accesses,
                    } => {
                        trace!("Called find_vars_ast() on a variable");
                        Self::put_read(identifier, vars);
                        for access in accesses {
                            Self::find_vars_op(access, vars, inner_scopes)?;
                        }
                    }
                    AstNode::list(list) => {
                        trace!("Called find_vars_ast() on a list");
                        for item in list {
                            Self::find_vars_op(item, vars, inner_scopes)?;
                        }
                    }
                    AstNode::dictionary(dictionary) => {
                        trace!("Called find_vars_ast() on a dict");
                        for (_, val) in dictionary {
                            Self::find_vars_op(val, vars, inner_scopes)?;
                        }
                    }
                    AstNode::set(set) => {
                        trace!("Called find_vars_ast() on a set");
                        for item in set {
                            Self::find_vars_op(item, vars, inner_scopes)?;
                        }
                    }
                    AstNode::string(_) => {
                        trace!("Called find_vars_ast() on a string");
                        // Do nothing
                    }
                    AstNode::number(_) => {
                        trace!("Called find_vars_ast() on a number");
                        // Do nothing
                    }
                    AstNode::boolean(_) => {
                        trace!("Called find_vars_ast() on a boolean");
                        // Do nothing
                    }
                    non_identity_ast!() => {
                        panic!("Tried calling find_vars_ast() with {node:?}");
                    }
                }
            }
        }

        Ok(())
    }

    fn find_vars_op<'a>(
        node: &'a MarkedOperationTree,
        vars: &mut IndexMap<MarkedString, VarClassification>,
        inner_scopes: &mut Vec<&'a MarkedAstNode>,
    ) -> Result<(), ParseError> {
        match &node.comp {
            OperationTree::Binary {
                operation: _,
                left,
                right,
            } => {
                Self::find_vars_op(left, vars, inner_scopes)?;
                Self::find_vars_op(right, vars, inner_scopes)?;
            }
            OperationTree::Unary {
                operation: _,
                value,
            } => {
                Self::find_vars_op(value, vars, inner_scopes)?;
            }
            OperationTree::Identity(ast) => {
                Self::find_vars_ast(ast, vars, inner_scopes)?;
            }
        }

        Ok(())
    }

    fn put_local(
        identifier: &MarkedString,
        vars: &mut IndexMap<MarkedString, VarClassification>,
    ) -> Result<(), ParseError> {
        match vars.get(identifier) {
            Some(VarClassification::Read) => {
                return Err(ParseError::marked(
                    &format!("local variable '{identifier}' referenced before assignment"),
                    identifier.mark.row,
                    identifier.mark.col,
                ));
            }
            Some(VarClassification::Local) => {}
            Some(_) => unreachable!(),
            None => {
                vars.insert(identifier.clone(), VarClassification::Local);
            }
        }

        Ok(())
    }

    fn put_read(identifier: &MarkedString, vars: &mut IndexMap<MarkedString, VarClassification>) {
        match vars.get(identifier) {
            Some(VarClassification::Read | VarClassification::Local) => {}
            Some(_) => unreachable!(),
            None => {
                vars.insert(identifier.clone(), VarClassification::Read);
            }
        }
    }

    pub fn local_idx(&self, name: &MarkedString) -> Option<usize> {
        self.local_vars.iter().position(|n| n == name)
    }

    pub fn deref_idx(&self, name: &MarkedString) -> Option<usize> {
        if let Some(idx) = self.cell_vars.iter().position(|n| n == name) {
            Some(idx)
        } else if let Some(idx) = self.free_vars.iter().position(|n| n == name) {
            Some(idx + self.cell_vars.len())
        } else {
            None
        }
    }

    pub fn child(&self, c: usize) -> &Self {
        &self.children[c]
    }

    pub fn num_local_vars(&self) -> usize {
        self.local_vars.len()
    }

    pub fn num_deref_vars(&self) -> usize {
        self.cell_vars.len() + self.free_vars.len()
    }
}
