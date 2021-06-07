# Goals:
- Fully support Source and GoldSource packets
- Support The Ship
- Good test coverage
- DOCS!

# How to use
  // TODO: Clean up / better explaination / link to use example / write example code
1. Send request to server with the default challenge value of -1
2. Parse response and check for challenge
  // TODO: link to response parsing
3. If the response contained a challenge resend the request with the challenge
4. Parse the response(s) using [`packet`]
  If you don't know what game engine the server your are querying is using attempt to send a ping request. The response differs between Source and Gold Source
5. If the payload is split across several packets use [`parse_split_payload()`](packet::parse_split_payload) and wait to recieve them all and combine the payloads.
  Payload can be compressed with bz2, decompress.
6. Parse the payload using the appropriate parser, determine by using [`parse_payload_header()`](packet::parse_payload_header)
7. ???
8. Profit

# Credits:
Amos and his wonderful writeups on rust. Specifically: https://fasterthanli.me/articles/rust-modules-vs-files
LogRocket for their blog post on nom: https://blog.logrocket.com/parsing-in-rust-with-nom/#errorhandlinginnom
benkay86 for and introduction to nom 5+: https://github.com/benkay86/nom-tutorial/#chap3