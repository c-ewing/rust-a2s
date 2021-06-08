/*!
This crate provides methods for parsing [`Source Engine`] and [`Gold Source`] [`A2S`] packets and payloads.

# Overview
Each [`A2S`] response is found in its respective module. Parsers take a slice and return a struct containing the fields defined on the [`A2S`] wiki page
All requests are parsed in [`requests`]

[`Source Engine`]: https://developer.valvesoftware.com/wiki/Source
[`Gold Source`]: https://developer.valvesoftware.com/wiki/Goldsource
[`A2S`]: https://developer.valvesoftware.com/wiki/Server_queries
*/

// This is gonna hurt (at first)
//#![deny(missing_docs)]
// TODO: Add better errors for parsing failures

///Parsing complete responses to [A2S_INFO](https://developer.valvesoftware.com/wiki/Server_queries#A2S_INFO) requests
pub mod info;
/// Parsing [A2S Packets](https://developer.valvesoftware.com/wiki/Server_queries#Protocol)
pub mod packet;
// TODO: Doc
pub mod parser_util;
/// Parsing complete responses to [A2S_PING](https://developer.valvesoftware.com/wiki/Server_queries#A2A_PING) requests for [Gold Source](https://developer.valvesoftware.com/wiki/Goldsource) and [Source](https://developer.valvesoftware.com/wiki/Source)
pub mod ping;
/// Parsing complete responses to [A2S_PLAYER](https://developer.valvesoftware.com/wiki/Server_queries#A2A_PLAYER) requests for [Gold Source](https://developer.valvesoftware.com/wiki/Goldsource) and [Source](https://developer.valvesoftware.com/wiki/Source)
pub mod player;
/// Parsing all complete [A2S](https://developer.valvesoftware.com/wiki/Server_queries#Requests) requests
pub mod requests;
/// Parsing complete responses to [A2S_RULES](https://developer.valvesoftware.com/wiki/Server_queries#A2A_RULES) requests for [Gold Source](https://developer.valvesoftware.com/wiki/Goldsource) and [Source](https://developer.valvesoftware.com/wiki/Source)
pub mod rules;

// TODO: Parse any slice provided and attempt to make a packet out of it
// Need to figure out how to return different packet types from one function call and how to determine
// split gold source from split source
