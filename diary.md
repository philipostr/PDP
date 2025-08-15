## Entry #2 (August 15, 2025): *I already have regrets*

Yeah... even the smallest things can cause pretty annoying inconveniences. In my initial grammar, I named anything that could be an identifier OR literal `ident`, which is pretty dumb in retrospect. I also hadn't included it in the `TokenType` enum for reasons I cannot begin to comprehend.

This commit was going to be focused on implementing the grammar into actual code with tangible logic, rather than being a soup of hopes and dreams. I believe I may have committed an error in the grammar though: Each nonterminal's arm, except for optionally the final one, starts with a unique regular string, followed by any string of nonterminals and terminals. It may have instead been a better move to ensure that these arms start with a regular string which are followed by ONLY A SINGLE NONTERMINAL. This would help greatly with ease of parsing, with the tradeoff of more tokens to implement. I'm ignoring this potential problem for now, to see how far I can go without caring. The only change I'm adopting because of this is that the my "iterable grammar" idea won't work anymore, so it will be fully recursive. If I want to make it iterable, it would be pseudo-iterable with `async`/`await` or threads with channels.

Something that did require changes to the grammar was the concept of the `Program` token. It inherently has no meaning by holding an unknowable amount of `Unit` tokens, because it doesn't actually tell the parser how to divide anything up. If each `Unit` was a line, then it would be salvageable, but that is not the case.

So where does that leave us? There's an easy answer: I rewrote the entire grammar, and through the architecture out of the window. You can read more about the grammar (it's actually TWO grammars) in the [specs folder](https://github.com/philipostr/PDP/blob/main/specs), it is quite a lot more complete, and extremely well documented. I guess this is it for now, thanks for waiting for 20+ days for this update!

## Entry #1 (July 24, 2025): *Every great Oddysey has an inception*

Hi there, ye who dares to venture into the darkness of poorly designed compilers, and even more poorly designed architectures.

I am preparing to embark on a dangerous journey, one that I don't see myself returning from without the loss of several limbs. That is to say, I plan on creating an extremely badly engineered piece of software that one as crazy as I could even conceive to call a Python intepreter. Not that I plan on making it badly designed, but rather I plan on designing it from scratch.

My idea is to not do any research on how compilers/interpreters work, nor how bytecode is effectively implemented, nor any hints as to the grammar of the real Python, nor anything else to aid this endevour. It will all come from me, and me alone.

That admittedly means that it will be very inefficient and, as the name suggests of the repository suggests, poorly designed. HOWEVER... my hope is that it works, and that I learn a thing or two along the way.

This first commit contains the initial semblances of an architecture, which I am basing on this [grammar](https://github.com/philipostr/PDP/blob/main/media/grammar_01.png) and [diagram](https://github.com/philipostr/PDP/blob/main/media/architecture_01.png), which are linked instead of attached because they would take up too much space.

These will inevitably change over time, but they may give a good starting point. Either that, or they lead me to architecture hell. 

There be dragons after all.
