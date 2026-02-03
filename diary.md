## Entry #9 (February 2, 2026) *The prophecy has come to pass*

It is time... the project has come to a close. There have been highs and lows, excitements and restarts, fleeting moments of progress and ceaseless meriads of failures wrought with pain. But we are here now, together, as the looming shadow in th shape of intertwining serpents finally aligns in our view: Python has been completed!!

To be clear, the project itself will be ongoing for a little while longer to support some much-needed features such as:
1. `elif` and `else`
2. Attribute accesses
3. Custom classes
4. f-strings
5. Closures
6. Debugger
7. And more...

However, while there is a lot of missing logic, the original idea of creating a compiler/interpreter from scratch has been achieved. I've learned so much doing this project in the past 7 months, and it has seriously been an incredible journey. I am extremely proud of what I have accomplished, and the fact that I practically did it all myself. The design, the architecture, the algorithms, the code, it all came from me. Were there some moments where I wasn't sure where to go? Yeah, absolutely. Did I get help from Stack Overflow and LLMs in those moments? Without a doubt. But they never told me how to do things, only what approaches would be useful. It was basically rubber-ducking with extra steps. DEFINITELY, never did I use copilot, vibe coding, or any other AI tool to write or even finish a single line of code. I think that it's simple to say, but not that common anymore in this day and age.

This will be my final diary entry. It has been an honour in being given the opportunity to entertain the 1 or 2 (and I'm being pretty generous here) of you who may come across this repository and read through this file, and I appreciate you having stopped by and given me your precious time and attention. Until next time :saluting_face:

## Entry #8 (January 13, 2026) *Bytecode did not byte me in the ass today*

Happy new year! And what a happy new year it truly is. This is my greatest triumph in this project; I have built my way from the ground up, without any clue as to how compilers work and how much work compilers would be, to this jubilous point we now find ourselves. I have been raised from the abyss, having been burned and scared by the azure flames of the almighty dragon, but alive I remain, with no less than prevailance at my side!

What is so good about this day? Why, this may have been the biggest moment that shall be observed by PDP. We have crossed the border, through thick and thin, across the yellow brick road, all the way from Parserville to Bytecodetropia. This is no longer a drill folks, we're not longer trying to understand what the programmer wants... now, we DO IT. Designing the whole of the bytecode architecture, including what the VM will look like and its dependencies, and cross referencing it with the bytecode instruction set that I was slowly building up, was such an incredible joy. I truly cannot explain how deeply I loved the experience, to the occassional dismay of my unfortunate girlfriend, who has against all odds continued to deal with my vast depths of nerdiness.

There's just one step left... one more step to take... and that step shall be taken. The bytecode will be executed, and it will be glorious.

## Entry #7 (December 20, 2025) *I save myself, sort of*

That was painful. Who knew that creating a new marking system to add row and column metadata to every bit of information contained within the AST would be so annoyingly tedious? In the end, however, I think that I have done something right in my prior architectural decisions, as this burden was **only** tedious, not difficult. Yes, I had to amend a lot of things, but amending is incredibly different from modifying. With the exception of a handlefull of very small things, all the amendments were simple additions, there was no logic or complicated functionality that was broken or even remotely worsened by the addition of markers. It sounds obvious, but the most simple of things can cause the biggest complications. Let's see where this takes us.

## Entry #6 (December 18, 2025) *Lexical parsing, shmexical parsing*

Why did I not keep row-column data across the concrete-to-abstract barrier? That was so stupid... now error messages in symbol-table creation and beyond (including a minor thing called EXECUTION) will not be able to be marked. How smart of past me, truly. I'm probably going to end up ammending this abomonable lack of forethought as my next step.

Anyway, onto the topic of this entry: lexical analysis and symbol tables. Boy... this is the first step in this project where I have legitimately needed to do some research. I know, I know, the whole point of this project was for me to make a Python interpreter from scratch, which includes all of the algorithms, design concepts, and the general pipeline of it all. However, for this one case, I did do some back-and-forths with ChatGPT to find out what in the hell symbol tables accomplish in such a flexible and dynamically-typed language like Python. Turns out, it will help with making the bytecode more specific, so that's cool. It also helps define closures and the variables that they borrow, which is not something that I had ever thought of in the Python world. The illumination on the differences between local, global, cell, and free variables was instrumental in making me not completely waste my time. So please, allow this betrayal just this once, my dear old pal.

## Entry #5 (November 9, 2025) *Mo' AST nodes, mo' problems*

I have made a severe, and continuous, lapse in my judgement. Specifically, in my creation of the PTAG. I thought I could be all smart and lazy and say "hey, look at me, I can get away with not specifying the patterns I need!". I did, in fact, need to specify the patterns I needed. Or better yet, I didn't **need** to, but it would have saved me 86 of my seemingly 57.1 million years of an eternally suffering existence in building this project.

As such, I needed to rewrite the PTAG almost in its entirety. Not the general concept of course, that's all perfect because I'm a mad genius. But I did write out all the possible input nodes because I was taking an incredibly long time to figure out what to write in each and every `AstNode::from_*()` method. Then, after getting through several of those, I realized that I did something wrong and had to restart. So here I am, seconds after convincing myself to purge it all and recommence from the beninging (shout out if you know the reference), deciding to rewrite the PTAG to make my life easier later.

Also, PDP does not currently have, with a high likelyhood that it will **never** have, order of operations. I could possibly implement it in the grammars, but NO. THAT WOULD BE UGLY AND I DON'T WANT IT.

**Update:** I just finished implementing the PTAG, and therefore the AST generation. I have been experimenting and playing around with it, and it seems like I only had to fix a single very minor logic error and everything else is working 100% correctly on first try. This has been my biggest success yet, as well as my biggest commit. Yippee!

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

I'm not entirely sure how to deliminate tokens once their lexeme has been identified. What I mean is, I check if a `char` slice starts with `"for"` and conclude that I found an `for` keyword, but what if the entire word was meant to be a variable `fortress`? I can of course use a regex to validate that the lexeme is complete, but I feel like that is extremely overkill and not really _graceful_. So my current plan is to simply use helper functions that confirm different types of boundaries, and use the correct one for each token type. I don't know how well that will age since it requires me to guess every possible character that can follow a lexeme in some cases, but I think it's fine at least for now.


## Entry #2 (August 15, 2025): *I already have regrets*

Yeah... even the smallest things can cause pretty annoying inconveniences. In my initial grammar, I named anything that could be an identifier OR literal `ident`, which is pretty dumb in retrospect. I also hadn't included it in the `TokenType` enum for reasons I cannot begin to comprehend.

This commit was going to be focused on implementing the grammar into actual code with tangible logic, rather than being a soup of hopes and dreams. I believe I may have committed an error in the grammar though: Each nonterminal's arm, except for optionally the final one, starts with a unique regular string, followed by any string of nonterminals and terminals. It may have instead been a better move to ensure that these arms start with a regular string which are followed by ONLY A SINGLE NONTERMINAL. This would help greatly with ease of parsing, with the tradeoff of more tokens to implement. I'm ignoring this potential problem for now, to see how far I can go without caring. The only change I'm adopting because of this is that the my "iterable grammar" idea won't work anymore, so it will be fully recursive. If I want to make it iterable, it would be pseudo-iterable with `async`/`await` or threads with channels.

Something that did require changes to the grammar was the concept of the `Program` token. It inherently has no meaning by holding an unknowable amount of `Unit` tokens, because it doesn't actually tell the parser how to divide anything up. If each `Unit` was a line, then it would be salvageable, but that is not the case.

So where does that leave us? There's an easy answer: I rewrote the entire grammar, and threw the architecture out of the window. You can read more about the grammar (it's actually TWO grammars) in the [specs folder](https://github.com/philipostr/PDP/tree/f5cc208a2af6acffb18525721b1de7a84333c217/specs), it is quite a lot more complete, and extremely well documented. I guess this is it for now, thanks for waiting for 20+ days for this update!

## Entry #1 (July 24, 2025): *Every great Oddysey has an inception*

Hi there, ye who dares to venture into the darkness of poorly designed compilers, and even more poorly designed architectures.

I am preparing to embark on a dangerous journey, one that I don't see myself returning from without the loss of several limbs. That is to say, I plan on creating an extremely badly engineered piece of software that one as crazy as I could even conceive to call a Python intepreter. Not that I plan on making it badly designed, but rather I plan on designing it from scratch.

My idea is to not do any research on how compilers/interpreters work, nor how bytecode is effectively implemented, nor any hints as to the grammar of the real Python, nor anything else to aid this endevour. It will all come from me, and me alone.

That admittedly means that it will be very inefficient and, as the name suggests of the repository suggests, poorly designed. HOWEVER... my hope is that it works, and that I learn a thing or two along the way.

This first commit contains the initial semblances of an architecture, which I am basing on this [grammar](https://github.com/philipostr/PDP/blob/main/media/grammar_01.png) and [diagram](https://github.com/philipostr/PDP/blob/main/media/architecture_01.png), which are linked instead of attached because they would take up too much space.

These will inevitably change over time, but they may give a good starting point. Either that, or they lead me to architecture hell. 

There be dragons after all.
