//! The `obscur-devtools` CLI entry point (spec §7.6): a thin argument parser over
//! [`obscur_devtools::cli`] (the client interface) and [`obscur_devtools::host`]
//! (a self-contained demo host to attach to).
//!
//! ```text
//! obscur-devtools serve [--tcp | --unix <path>] [--token <t>] [--scopes <list>]
//! obscur-devtools inspect <target|all> --tcp <addr> | --unix <path> [--token <t>]
//! obscur-devtools subscribe <topic>    --tcp <addr> | --unix <path> [--token <t>]
//! obscur-devtools ping                 --tcp <addr> | --unix <path> [--token <t>]
//! ```
#![forbid(unsafe_code)]

use std::process::ExitCode;

use obscur_devtools::cli;
use obscur_devtools::host;
use obscur_devtools_protocol::message::{
    HandshakeResponse, InjectedEvent, PermissionScope, ScopeSet,
};
use obscur_devtools_server::transport::{ClientBound, SocketClient, SocketListener};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("obscur-devtools: {message}");
            ExitCode::FAILURE
        }
    }
}

/// Parses `args` and runs the selected command, returning a human error string.
fn run(args: &[String]) -> Result<(), String> {
    let Some((command, rest)) = args.split_first() else {
        print_usage();
        return Ok(());
    };
    match command.as_str() {
        "serve" => run_serve(rest),
        "inspect" => run_inspect(rest),
        "subscribe" => run_subscribe(rest),
        "inject" => run_inject(rest),
        "ping" => run_ping(rest),
        "-h" | "--help" | "help" => {
            print_usage();
            Ok(())
        }
        other => Err(format!("unknown command '{other}' (try --help)")),
    }
}

/// Where a connection binds/attaches: a loopback TCP address or a Unix path.
enum Endpoint {
    /// Loopback TCP. For `serve`, `None` binds an ephemeral port.
    Tcp(Option<String>),
    /// A Unix domain socket path.
    Unix(String),
}

/// The connection + auth options common to every client command.
struct Options {
    endpoint: Endpoint,
    token: String,
    scopes: ScopeSet,
    positionals: Vec<String>,
}

/// Parses `--tcp`/`--unix`/`--token`/`--scopes` plus one optional positional.
fn parse_options(args: &[String]) -> Result<Options, String> {
    let mut endpoint = Endpoint::Tcp(None);
    let mut token = "dev".to_owned();
    let mut scopes = ScopeSet::all();
    let mut positionals = Vec::new();

    let mut index = 0;
    while index < args.len() {
        let arg = args[index].as_str();
        match arg {
            "--tcp" => {
                // `--tcp` may stand alone (serve, ephemeral) or take an address.
                let next = args.get(index + 1).filter(|next| !next.starts_with("--"));
                endpoint = Endpoint::Tcp(next.map(|addr| {
                    index += 1;
                    addr.clone()
                }));
            }
            "--unix" => {
                let path = args.get(index + 1).ok_or("--unix needs a path")?;
                endpoint = Endpoint::Unix(path.clone());
                index += 1;
            }
            "--token" => {
                token = args.get(index + 1).ok_or("--token needs a value")?.clone();
                index += 1;
            }
            "--scopes" => {
                scopes = parse_scopes(args.get(index + 1).ok_or("--scopes needs a value")?)?;
                index += 1;
            }
            other if other.starts_with("--") => {
                return Err(format!("unknown option '{other}'"));
            }
            value => positionals.push(value.to_owned()),
        }
        index += 1;
    }
    Ok(Options {
        endpoint,
        token,
        scopes,
        positionals,
    })
}

/// Parses a comma-separated scope list (`read,subscribe`, `all`, `observer`).
fn parse_scopes(spec: &str) -> Result<ScopeSet, String> {
    match spec {
        "all" => return Ok(ScopeSet::all()),
        "observer" => return Ok(ScopeSet::observer()),
        _ => {}
    }
    let mut set = ScopeSet::empty();
    for name in spec.split(',') {
        let scope = match name.trim() {
            "read" => PermissionScope::Read,
            "subscribe" => PermissionScope::Subscribe,
            "control" => PermissionScope::Control,
            "inject" => PermissionScope::Inject,
            other => return Err(format!("unknown scope '{other}'")),
        };
        set = set.with(scope);
    }
    Ok(set)
}

/// Connects a [`SocketClient`] to the resolved endpoint.
fn connect(endpoint: &Endpoint) -> Result<SocketClient, String> {
    match endpoint {
        Endpoint::Tcp(Some(addr)) => {
            let parsed = addr
                .parse()
                .map_err(|_| format!("invalid address '{addr}'"))?;
            SocketClient::connect_loopback(parsed).map_err(|e| format!("connect failed: {e}"))
        }
        Endpoint::Tcp(None) => {
            Err("a client command needs --tcp <addr> or --unix <path>".to_owned())
        }
        #[cfg(unix)]
        Endpoint::Unix(path) => {
            SocketClient::connect_unix(path).map_err(|e| format!("connect failed: {e}"))
        }
        #[cfg(not(unix))]
        Endpoint::Unix(_) => Err("Unix sockets are not available on this platform".to_owned()),
    }
}

/// Connects and handshakes, returning the client on acceptance or a rejection
/// error message.
fn connect_and_handshake(options: &Options) -> Result<SocketClient, String> {
    let mut client = connect(&options.endpoint)?;
    let response = cli::handshake(&mut client, &options.token).map_err(|e| e.to_string())?;
    println!("{}", cli::format_handshake(&response));
    match response {
        HandshakeResponse::Accepted { .. } => Ok(client),
        HandshakeResponse::Rejected { reason } => Err(format!("handshake rejected: {reason:?}")),
    }
}

/// `inspect <target|all>`: connect, handshake, and print each target's snapshot.
fn run_inspect(args: &[String]) -> Result<(), String> {
    let options = parse_options(args)?;
    let target_arg = options
        .positionals
        .first()
        .map(String::as_str)
        .ok_or("inspect needs a target (a name or 'all')")?;
    let targets = if target_arg == "all" {
        cli::all_targets()
    } else {
        vec![
            cli::parse_target(target_arg)
                .ok_or_else(|| format!("unknown target '{target_arg}'"))?,
        ]
    };

    let mut client = connect_and_handshake(&options)?;
    for (index, target) in targets.into_iter().enumerate() {
        let id = index as u64 + 1;
        let reply = cli::inspect(&mut client, id, target).map_err(|e| e.to_string())?;
        println!("{}", cli::format_client_bound(&reply));
    }
    Ok(())
}

/// `subscribe <topic>`: connect, handshake, and print the subscription grant.
fn run_subscribe(args: &[String]) -> Result<(), String> {
    let options = parse_options(args)?;
    let topic_arg = options
        .positionals
        .first()
        .map(String::as_str)
        .ok_or("subscribe needs a topic")?;
    let topic =
        cli::parse_topic(topic_arg).ok_or_else(|| format!("unknown topic '{topic_arg}'"))?;

    let mut client = connect_and_handshake(&options)?;
    let reply = cli::subscribe(&mut client, 1, topic).map_err(|e| e.to_string())?;
    println!("{}", cli::format_client_bound(&reply));
    match &reply {
        // A denial is the server saying it will never push on this session, so
        // say so rather than leaving the user watching a silent socket.
        ClientBound::Subscription(subscription) if !subscription.granted => {
            println!(
                "note: this host serves request/reply and pushes no events, so nothing would arrive.\n      \
                 The demo host (`obscur-devtools serve`) has no runtime to report on; attach to a\n      \
                 host that publishes, such as obscur-kit's showcase_window example."
            );
        }
        ClientBound::Subscription(_) => {
            print_pushed_events(&mut client);
        }
        _ => {}
    }
    Ok(())
}

/// Streams pushed events to stdout until the connection ends or the user
/// interrupts — a granted subscription now delivers, so print what arrives.
fn print_pushed_events(client: &mut SocketClient) {
    println!("subscribed · streaming events (^C to stop)");
    loop {
        match client.receive() {
            Ok(Some(reply)) => println!("{}", cli::format_client_bound(&reply)),
            // A clean close: the host ended the session.
            Ok(None) => return,
            Err(error) => {
                eprintln!("stream ended: {error}");
                return;
            }
        }
    }
}

/// `inject <down|up|click> <x> <y>`: connect, handshake, and inject synthetic
/// pointer events at logical `(x, y)` — the write half of the agent loop.
fn run_inject(args: &[String]) -> Result<(), String> {
    let options = parse_options(args)?;
    let kind = options
        .positionals
        .first()
        .map(String::as_str)
        .ok_or("inject needs: <down|up|click> <x> <y>")?;
    let coord = |index: usize, axis: &str| {
        options
            .positionals
            .get(index)
            .and_then(|value| value.parse::<i32>().ok())
            .ok_or_else(|| format!("inject needs an integer {axis}"))
    };
    let x = coord(1, "x")?;
    let y = coord(2, "y")?;
    let events = match kind {
        "down" => vec![InjectedEvent::PointerDown { x, y }],
        "up" => vec![InjectedEvent::PointerUp { x, y }],
        "click" => {
            vec![
                InjectedEvent::PointerDown { x, y },
                InjectedEvent::PointerUp { x, y },
            ]
        }
        other => return Err(format!("unknown inject kind '{other}' (down|up|click)")),
    };

    let mut client = connect_and_handshake(&options)?;
    for (index, event) in events.into_iter().enumerate() {
        let reply = cli::inject(&mut client, index as u64 + 1, event).map_err(|e| e.to_string())?;
        println!("{}", cli::format_client_bound(&reply));
    }
    Ok(())
}

/// `ping`: connect and handshake only.
fn run_ping(args: &[String]) -> Result<(), String> {
    let options = parse_options(args)?;
    connect_and_handshake(&options)?;
    Ok(())
}

/// `serve`: stand up the demo host and serve client sessions.
fn run_serve(args: &[String]) -> Result<(), String> {
    let options = parse_options(args)?;
    let listener = match &options.endpoint {
        Endpoint::Tcp(_) => {
            SocketListener::bind_loopback().map_err(|e| format!("bind failed: {e}"))?
        }
        #[cfg(unix)]
        Endpoint::Unix(path) => {
            SocketListener::bind_unix(path).map_err(|e| format!("bind failed: {e}"))?
        }
        #[cfg(not(unix))]
        Endpoint::Unix(_) => {
            return Err("Unix sockets are not available on this platform".to_owned());
        }
    };
    match listener.local_addr().ok().flatten() {
        Some(addr) => println!(
            "obscur-devtools demo host listening on {addr}\n\
             connect: obscur-devtools inspect all --tcp {addr} --token {}",
            options.token
        ),
        None => println!("obscur-devtools demo host listening on a Unix socket"),
    }
    println!(
        "token '{}' granted: {}",
        options.token,
        cli::format_scopes(options.scopes)
    );
    host::serve_forever(&listener, &options.token, options.scopes).map_err(|e| e.to_string())
}

/// Prints the command summary.
fn print_usage() {
    println!(
        "obscur-devtools — external DevTools client (spec §7.6)\n\n\
         USAGE:\n  \
         obscur-devtools serve [--tcp | --unix <path>] [--token <t>] [--scopes <list>]\n  \
         obscur-devtools inspect <target|all> --tcp <addr> | --unix <path> [--token <t>]\n  \
         obscur-devtools subscribe <topic>    --tcp <addr> | --unix <path> [--token <t>]\n  \
         obscur-devtools inject <down|up|click> <x> <y>  --tcp <addr> | --unix <path> [--token <t>]\n  \
         obscur-devtools ping                 --tcp <addr> | --unix <path> [--token <t>]\n\n\
         TARGETS: tree styles layout event-flow focus animations resources\n           \
         accessibility scene rendering caches plugin-state  (or 'all')\n\
         TOPICS:  diagnostics lifecycle frames focus\n\
         SCOPES:  read subscribe control inject  (comma-separated, or 'all'/'observer')"
    );
}
