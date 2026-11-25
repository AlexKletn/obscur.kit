//! Sibling ordering rules for authored conditional chains.

/// Checks a sibling tag and returns whether the current chain can accept another
/// branch. Pass an empty tag for content that interrupts a chain.
pub fn check_conditional_branch(tag: &str, chain_open: bool) -> Result<bool, &'static str> {
    match tag {
        "If" => Ok(true),
        "ElseIf" if chain_open => Ok(true),
        "Else" if chain_open => Ok(false),
        "ElseIf" => Err("`<ElseIf>` must immediately follow an `<If>` or `<ElseIf>`"),
        "Else" => Err("`<Else>` must immediately follow an `<If>` or `<ElseIf>`"),
        _ => Ok(false),
    }
}

/// Checks the number of authored `cond` attributes on a conditional branch.
pub fn check_condition_attribute(tag: &str, count: usize) -> Result<(), &'static str> {
    match (tag, count) {
        ("If", 0) => Err("`<If>` requires a `cond` attribute"),
        ("ElseIf", 0) => Err("`<ElseIf>` requires a `cond` attribute"),
        ("If" | "ElseIf", 2..) => Err("a conditional branch accepts only one `cond` attribute"),
        ("Else", 1..) => Err("`<Else>` cannot have a `cond` attribute; use `<ElseIf>`"),
        _ => Ok(()),
    }
}
