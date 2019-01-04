# Extract Markdown Links

This is just an extremely naive and simple tool designed to extract all the Markdown links from a given document. I do this manually pretty regularly, and it was a fun exercise to build it in Rust and play with some different data structures and see how they ended up working for this.

This is *not* something you should look at as any kind of optimal Rust code; it was literally just experimenting with a couple different approaches. In particular, the use of iterators (with `fold` especially) here is *fine* but it's probably more idiomatic to do this particular bit of state management imperatively in Rust.