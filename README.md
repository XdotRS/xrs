<h1 align="center">X.RS</h1>

An asynchronous X library for Rust.

> **Warning** \
> Both X.RS and this README are thoroughly incomplete.
> Currently, a lot of the work is going into [XRB], which
> provides the basic representations of X protocols which
> this library builds on.

[XRB]: https://github.com/XdotRS/xrb/

X.RS takes a lot of inspiration from XCB - each message
in X.RS directly corresponds to a message in the underlying
protocols. But X.RS does diverge in the representations of
those messages: all the same configuration options are
possible, but the API of messages is restructured in many
cases to be more intuitive, to eliminate a number of errors
with increased type safety, and, in general, to be more rusty.

## `Client`
X.RS' API is centered around one important struct: the
`Client`. If you are familiar with XCB, this is quite similar
to `xcb_connection_t`. `Client` represents your X client and its
connection to the X server. It has the following specific
purposes:
- to initiate a connection to the X server, thus creating a new
  `Client`, via `Client::connect`;
- to contain and allow access to the information returned by the
  X server in response to a successful connection (e.g. the root
  windows) - where appropriate, X.RS should enforce provided minimum
  and maximum bounds;
- to send `Request`s via `Client::send`; and
- to receive incoming `Event`s, `Error`s, and `Reply`s.

# Examples
```rust
use xrs::{Client, ConnectError, Display, EventMask::*, Message, WindowConfig, x11::request as req};

#[tokio::main]
pub async fn main() -> Result<!, ConnectError> {
    let client = Client::connect(Display::Default, None)?;
    let root = client.screens[0].root;

    client.send(req::ConfigureWindow {
        target: root,
        config: WindowConfig::new().event_mask(SUBSTRUCTURE_NOTIFY | SUBSTRUCTURE_REDIRECT),
    }).await?;

    loop {
        if let Some(message) = client.next_message() {
            match message {
                Message::Event(_event) => todo!(),

                Message::Error(_error) => todo!(),
            }
        }
    }
}
```
