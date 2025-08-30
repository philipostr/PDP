## Entry #4 (August 29, 2025) *Digging a hole straight into token-parsing hell*

### Idiomatic Rust architecture

Since I'm using Rust for this project, I want to make my logic as idiomatically Rust as possible. As such, for the implementation of the TPG, I'm using enums to describe every node type, and variants for every possible arm, holding their recursive nodes in a tuple. For the meta-nodes (`*`, `+`, `?`, etc.), I'm defining structs for each one as well.

I'm not sure how the performance will be for doing all of this, nor if it will even make sense as an architecture later, but that's the entire point of the project: to see how far I can make it with my inexperience and lack of knowledge, find the point where my initial decisions made everything impossible, and adapt/pivot that initial decision as best I can.

In fact, I have already found a problem with being Rustic in my Tokens code. My plan, when making the TPG, was to use `Token`s as nodes as well. That's all fine and good, but I just realized that I can't actually specify specific tokens as types. This is a problem because, instead of defining the `Result` node's 2nd arm as having a `Token::NAME`, I can only specify that it has a `Token`. I can obviously work around this by assuming that this token is a `NAME`, but that's not particularly pretty. I could also make a tuple-struct for each `Token` type, but that's not pretty either. I'm probably going with the latter, but I'm not happy about it.

As a side note, because of the immense amount of helper macros and functions I am making, I am heavily considering writing a crate for general TPBA language parsing, that can be used for any language as long as a TPG and PTAG were already built for it. Of course, this idea would only have any semblance of legitimacy AFTER I'm done with this current project.

### Errors from quantified nodes

I have come across an interesting problem; in my current implementation, I programmed the quantified nodes (`*`, `+`, and `?`) to keep matching until an error is found, roll back to the beginning of the current match attempt, and continue with the next node. I am now realizing, rather stupidly, that I will always be throwing out any errors that I find within those quantified nodes. This specifically poses a problem for the `Program` node, which is always the root of the parse tree. Non-trivially, this node is always `Scoped* END`, and because we throw out all errors found within `Scoped*`, the ultimate error will be that the next node doesn't match `END`. That's hardly helpful to the programmer...

My potentially naive solution is to discard the error only if the first token did not match anything, and propagate the error if any valid first token WAS matched. More nuance may or may not be necessary, I will found out by experimenting.

### Debugging is difficult

If there's a single thing I have learned the importance of, it's logging. I made a massive 1.2k line file in one go, without ever having run it a single time to see if what I had so far was working. That isn't necessarily my fault, since for things to work properly, everything needed to be finished. So when I finally sat down and tried running it... well, I didn't have a great time. Of course there were bugs, but I had no way of finding them since errors propagate up the call stack, and so I would never know where they originated from nor how they came to be. I therefore added debug and trace logging everywhere, and was finally able to iron out all the bugs I could get my grubby little hands on.

## Entry #3 (August 22, 2025) *Chewing gum and parsing code, and I'm all out of code*

I'm not entirely sure how to deliminate tokens once they're lexeme has been found. What I mean is, I check if a `char` slice starts with `"for"` and conclude that I found an `for` keyword, but what if the entire word was meant to be a variable `fortress`? I can of course use a regex to validate that the lexeme is complete, but I feel like that is extremely overkill and not really _graceful_. So my current plan is to simply use helper functions that confirm different types of boundaries, and use the correct one for each token type. I don't know how well that will age since it requires me to guess every possible character that can follow a lexeme in some cases, but I think it's fine at least for now.


## Entry #2 (August 15, 2025): *I already have regrets*

Yeah... even the smallest things can cause pretty annoying inconveniences. In my initial grammar, I named anything that could be an identifier OR literal `ident`, which is pretty dumb in retrospect. I also hadn't included it in the `TokenType` enum for reasons I cannot begin to comprehend.

This commit was going to be focused on implementing the grammar into actual code with tangible logic, rather than being a soup of hopes and dreams. I believe I may have committed an error in the grammar though: Each nonterminal's arm, except for optionally the final one, starts with a unique regular string, followed by any string of nonterminals and terminals. It may have instead been a better move to ensure that these arms start with a regular string which are followed by ONLY A SINGLE NONTERMINAL. This would help greatly with ease of parsing, with the tradeoff of more tokens to implement. I'm ignoring this potential problem for now, to see how far I can go without caring. The only change I'm adopting because of this is that the my "iterable grammar" idea won't work anymore, so it will be fully recursive. If I want to make it iterable, it would be pseudo-iterable with `async`/`await` or threads with channels.

Something that did require changes to the grammar was the concept of the `Program` token. It inherently has no meaning by holding an unknowable amount of `Unit` tokens, because it doesn't actually tell the parser how to divide anything up. If each `Unit` was a line, then it would be salvageable, but that is not the case.

So where does that leave us? There's an easy answer: I rewrote the entire grammar, and through the architecture out of the window. You can read more about the grammar (it's actually TWO grammars) in the [specs folder](https://github.com/philipostr/PDP/tree/f5cc208a2af6acffb18525721b1de7a84333c217/specs), it is quite a lot more complete, and extremely well documented. I guess this is it for now, thanks for waiting for 20+ days for this update!

## Entry #1 (July 24, 2025): *Every great Oddysey has an inception*

Hi there, ye who dares to venture into the darkness of poorly designed compilers, and even more poorly designed architectures.

I am preparing to embark on a dangerous journey, one that I don't see myself returning from without the loss of several limbs. That is to say, I plan on creating an extremely badly engineered piece of software that one as crazy as I could even conceive to call a Python intepreter. Not that I plan on making it badly designed, but rather I plan on designing it from scratch.

My idea is to not do any research on how compilers/interpreters work, nor how bytecode is effectively implemented, nor any hints as to the grammar of the real Python, nor anything else to aid this endevour. It will all come from me, and me alone.

That admittedly means that it will be very inefficient and, as the name suggests of the repository suggests, poorly designed. HOWEVER... my hope is that it works, and that I learn a thing or two along the way.

This first commit contains the initial semblances of an architecture, which I am basing on this [grammar](https://github.com/philipostr/PDP/blob/main/media/grammar_01.png) and [diagram](https://github.com/philipostr/PDP/blob/main/media/architecture_01.png), which are linked instead of attached because they would take up too much space.

These will inevitably change over time, but they may give a good starting point. Either that, or they lead me to architecture hell. 

There be dragons after all.
