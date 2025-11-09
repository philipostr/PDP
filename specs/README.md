[TPBA (Top-down Parsing, Bottom-up Abstraction)](#tpba-top-down-parsing-bottom-up-abstraction)  
&emsp;[Process](#process)  
&emsp;[TPG (Token Parse Grammar)](#tpg-token-parse-grammar)  
&emsp;&emsp;[Syntax](#syntax)  
&emsp;&emsp;[Example](#example)  
&emsp;[PTAG (Parse Tree Abstraction Grammar)](#ptag-parse-tree-abstraction-grammar)  
&emsp;&emsp;[Syntax](#syntax-1)  
&emsp;&emsp;[Building a PTAG](#building-a-ptag)  
&emsp;&emsp;[Example](#example-1)  

# TPBA (Top-down Parsing, Bottom-up Abstraction)

To be clear, I have no idea what the standard way of parsing tokens and generating abstract syntax trees is, but I have come up with my own way that may or may not be different to varying degrees. The purpose of this README is to explain the process, and describe the syntax used for TPG and PTAG.

## Process

In my trial-and-error stages of coming up with a concrete syntax grammar, I often attempted to use it to manually parse the tokens of some random small script by hand, both to see if the logic was sound, and to determine whether it actually made unambiguous unvague sense from a computer's perspective. While doing this, I noticed that the trees described the concrete syntax while traversing down from the root node, but it started to form a clear abstraction path if traversing up from the leaf nodes.

For example, the code snippet
```
x = 10
```
resulted in a parse tree containing the subtree
```
                Expr
                 |
             ExprUnary
                 |
              ExprUnit
                 |
              NUMBER(4)
```

I realized that, clearly, all of the nodes in this subtree were ubiquitously equivalent to a single node, in this case the token `NUMBER(4)`. Using this logic, I derived a tree-parsing grammar that transforms nodes based on their child subtrees. One would think that the sheer number of possible subtrees that a node may have would make this impossible or at least nearly so, but once you consider that you also inductively did the same to all the child nodes, you're not dealing with actual subtrees, but purely direct children. These direct children encode within themselves the subtrees that they contained, and thus we can propagate this up to the root node of the entire parse tree, having generated an AST in the process.

I have called this down-up scan method TPBA, as it parses top-down, and abstracts bottom-up on its way back. This can be programmed quite easily because of the recursive nature of the compiler: determine what arm of the TPG (Token Parse Grammar) the next token indicates, parse each of the children appropriately which also returns their abstraction result, and use these abstractions to determine what the current node should be abstracted to using the PTAG (Parse Tree Abstraction Grammar). Like this, you get both the concrete syntax tree and AST in one fell swoop!

## TPG (Token Parse Grammar)

The TPG allows for context-free grammar definitions, but also context-sensitive grammars like in the case of PDP. In the latter cases, the TPG must start with the definition of the context object that will be passed down through the subtrees. This should contain all the contexts to be used, their types,, starting values, optionally a shorter identifying name for a cleaner grammar, and any extra mutation rules if necessary (valid regex for strings, maximum value for numbers, etc).

The grammar itself is of course comprised of node (nonterminal) definitions: sets of match arms describing the allowed patterns of tokens (terminals) and child nodes. For the grammar to be more easily usable, the arms should follow the standard rules of *LL(n)* grammars:

1. There MUST be no left recursion: If a nonterminal has arms that start with another nonterminal, the latter should in no possible way expand back to the former as the start of an arm. This is to prevent infinite parse trees that never manage to advance through the token stream. Thereby making the compiler hang forever.
2. Arms that start with terminals must have that terminal be UNIQUE among the other starting terminals in a node definition: Otherwise, if two or more arms start with the same terminal, the compiler will have to do backtracking to try each possibility if the previous ones failed.
3. There can only be at most ONE arm starting with a nonterminal per node definition, and it must be placed last: By doing this, the compiler has no backtracking and therefore a more simple path. If each arm starts with a terminal, this instantly advances through the token stream and leaves no room for backtracking or ambiguity, and leaves the only nonterminal-starting arm as the "final resort" or "else" clause.

The first *LL(n)* rule must be followed at all costs, but the second and third have some more leniency. The second is not 100% imperative, but can always be completely eliminated by factoring out the common parts of the bad arms. The third is the only one that may be necessary to break, such as with the `BracExpr` in PDP having two nonterminal-starting arms, but that is the ONLY time that this happens in all of PDP.

### Syntax

* Tokens are ALL CAPS and may have internal values specified if applicable.
* Nodes are PascalCase.
* Sections enclosed with `[]` are "meta", and are used to describe the role of contexts fields.
* A `*`, `+`, `?`, `{amount}` can be used to denote varying amounts of what it describes.
* The first nonterminal is the root node of the resulting parse tree.

### Example:

```
Context: {
    pyramidLevel: int = 1 // denoted with a 'p'. Cannot go above 5.
}
```

```
HeadOfThePyramid: CHAR('*') NEWLINE StarPyramid   [p += 1]
                | CHAR('#') NEWLINE HashPyramid   [p += 1]
```

```
StarPyramid: END
           | CHAR('*'){p} Comment StarPyramid
```

```
HashPyramid: END
           | CHAR('#'){p} Comment HashPyramid
```

```
Comment:    NEWLINE          [p += 1]
 [p even] | STRING NEWLINE   [p += 1]
```

The above grammar would allow the following example inputs (notice the ending `NEWLINE`s):

```
*

```

```
#

```

```
*
** hello i am a comment
***
```

But it would not allow the following example inputs:

```

```

```
*
##

```

```
*
**
***
```

```
# this is a comment
##

```

```
*
** there are
***
**** more than
*****
****** 5 lines
```

## PTAG (Parse Tree Abstraction Grammar)

Given a parse tree generated by a TPG as input, the role of a PTAG is to identify and extract the abstract constructs hidden behind the concrete verbosity of terminals and nonterminals by doing a post-order tree traversal. This is designed, as described previously, to allow the PTAG to be used at the same time as the parse tree generation itself.

This is meant to be a more syntactically simple grammar as opposed to its concrete cousin, in part due to its inherent abstract nature. The main idea behind it is to describe the **propagation** of values up the tree, instead of always creating new node types. An example of this propagation is shown in [this section](#process).

Basically, the idea of a PTAG is to describe an abstraction for every possible (syntactically possible by the TPG) combination of a node and its children in the parse tree. This is only meant to be a high-level logical representation of abstraction, but the implementation is up to the programmer. Specifically, one will want to embed abstractions within other abstractions instead of keeping them as literal children in the tree, cleaning up the AST and simplifying the code.

### Syntax

* The LHS of an abstraction relationship is the concrete syntax nonterminal, since it has not yet been abstracted. This should, of course, be in PascalCase.
* Terminals are implicitly abstracted to an abstract version of themselves, which is the same but in snake_case. They should not be listed explicitly, it would only add unnecessary noise.
* Every arm in the TPG should generally be translated to a single abstraction relationship. Even if multiple arms of a single TPG nonterminal look almost identical, having the same amount and similar types of terminals and nonterminals, the corresponding PTAG should still have an explicit abstraction relationship for each.
* An abstraction arm within an abstraction relationship has the abstract children on the LHS, and the resultant abstraction on the RHS.
* If an arm's LHS and RHS are both the same singular abstraction, it can be written as just the abstraction; no need for LHS ⟶ RHS.
* If an arm's LHS is empty, it should have `empty`. If the arm doesn't result in anything meaningful, it should have `empty` as the RHS.
* A `*`, `+`, `?`, `{amount}` can be used to denote varying amounts of what it describes.

### Building a PTAG

Normally, in the same way that the abstraction is done backwards from the concrete parsing, abstraction relationships for nonterminals should be written starting from the last nonterminal in the TPG, and ending on the first nonterminal. This is because, in most cases, the creation of the nonterminals followed a logical process that implicitly describes a topological sorting. Since we care about defining abstraction relationships by the possible abstracted children of the nonterminal in question, we want to start from the end of the topological sorting and go up as much as possible.

Writing the first few abstraction relationships will be simple. However, as you go further, you will have more and more possible arms for each relationship. Lets say you have a nonterminal `Foo` which abstracts to different things depending on the abstractions of `Bar` and `Baz`, and `Bar` and `Baz` each can abstract to 2 different results, `Foo` will have to define 4 abstraction arms, each of which could possibly have different results... So on so forth. In the worst case, this means that every subsequent nonterminal can abstract to an exponentially larger amount of results. If you're smart about it though, you can minimize this quite easily by having many arms yield the same result but with different values inside their fields.

### Example

```
Comment.1: empty
```

```
Comment.2: string ⟶ comment
```

```
HashPyramid.1: empty
```

```
HashPyramid.2: char+ comment empty  ⟶ hashes
               char+ empty empty    ⟶ hashes
               char+ comment hashes ⟶ hashes
               char+ empty hashes   ⟶ hashes
```

```
StarPyramid.1: empty
```

```
StarPyramid.2: char+ comment empty ⟶ stars
               char+ empty empty   ⟶ stars
               char+ comment stars ⟶ stars
               char+ empty stars   ⟶ stars
```

```
HeadOfThePyramid.1: char stars ⟶ stars
                    char empty ⟶ stars
```

```
HeadOfThePyramid.2: char hashes ⟶ hashes
                    char empty  ⟶ hashes
```
