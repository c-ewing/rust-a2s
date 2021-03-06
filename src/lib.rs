/*!
This crate provides methods for sending [`A2S`] queries to source servers.

# Goals:
- One shot methods get_players(server ip)
- Repeated access through object creation server.get_players() to avoid repeated socket setup and deconstruction
- Fully support GoldSource packets
- Full testing coverage
- nom for message parsing

```rust

//code block
let a = 0;

```

[`A2S`]: https://developer.valvesoftware.com/wiki/Server_queries

End of Doc

# Credits:
Amos and his wonderful writeups on rust. Specifically: https://fasterthanli.me/articles/rust-modules-vs-files
LogRocket for their blog post on nom: https://blog.logrocket.com/parsing-in-rust-with-nom/#errorhandlinginnom
benkay86 for and introduction to nom 5+: https://github.com/benkay86/nom-tutorial/#chap3
*/

extern crate nom;

pub mod parse;
