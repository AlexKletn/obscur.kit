use obscur_devtools_protocol::message::{PermissionScope, ScopeSet};

use super::endpoint::Endpoint;

/// The connection + auth options common to every client command.
pub(super) struct ConnectionOptions {
    pub(super) endpoint: Endpoint,
    pub(super) token: String,
    pub(super) scopes: ScopeSet,
    pub(super) positionals: Vec<String>,
}

/// Parses `--tcp`/`--unix`/`--token`/`--scopes` plus one optional positional.
pub(super) fn parse_options(args: &[String]) -> Result<ConnectionOptions, String> {
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
    Ok(ConnectionOptions {
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
