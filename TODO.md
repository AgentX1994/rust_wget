rust_wget TODOs

- Implement all HTTP status codes
- HTTPS
    - TLS
- Recursive fetching
    - Requires parsing HTML/XHTML/CSS according to wget's man page
    - Also respect robots.txt
- HTTP2 frames instead of only HTTP1 messages
- Progress bar on receiving data 
    - async Rust, or just a periodic timeout?
- Retry on network problems
    - How to test?
- Unit tests
- Integration tests
- Make options closer to wget's
- Error handling
- TCP