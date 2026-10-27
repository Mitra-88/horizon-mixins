use crate::{
    Diagnostic, Result,
    manifest::{Anchor, Slice},
    scopes::Scopes,
    sites::{self, Sites},
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Boundary {
    pub call: usize,
    pub expression: usize,
    pub fingerprint: String,
    pub selector: String,
    pub ordinal: usize,
    pub require: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedSlice {
    pub from: Option<Boundary>,
    pub to: Option<Boundary>,
    pub start: usize,
    pub end: usize,
}
fn boundary(anchor: &Anchor, sites: &Sites) -> Result<Boundary> {
    if anchor.require == 0 {
        return Err(Diagnostic::new(
            "E_SLICE",
            "boundary require must be positive",
        ));
    }
    let selector = sites::parse_call_selector(&anchor.selector)?;
    let matches: Vec<_> = sites
        .calls
        .iter()
        .filter(|s| sites::call_matches(&s.expr, &selector))
        .collect();
    if matches.len() != anchor.require {
        return Err(Diagnostic::new(
            "E_SLICE_COUNT",
            format!(
                "slice boundary '{}' requires {}, found {}",
                anchor.selector,
                anchor.require,
                matches.len()
            ),
        ));
    }
    let site = matches.get(anchor.ordinal).ok_or_else(|| {
        Diagnostic::new("E_SLICE_ORDINAL", "slice boundary ordinal is out of range")
    })?;
    if !site.whole_expression || site.const_context {
        return Err(Diagnostic::new(
            "E_SLICE_SCOPE",
            "slice boundaries must be whole-expression runtime calls",
        ));
    }
    Ok(Boundary {
        call: site.index,
        expression: site.expression,
        fingerprint: site.fingerprint.clone(),
        selector: anchor.selector.clone(),
        ordinal: anchor.ordinal,
        require: anchor.require,
    })
}
pub fn resolve(slice: &Slice, sites: &Sites, scopes: &Scopes) -> Result<ResolvedSlice> {
    if slice.from.is_none() && slice.to.is_none() {
        return Err(Diagnostic::new("E_SLICE", "slice needs from or to"));
    }
    let from = slice
        .from
        .as_ref()
        .map(|a| boundary(a, sites))
        .transpose()?;
    let to = slice.to.as_ref().map(|a| boundary(a, sites)).transpose()?;
    match (&from, &to) {
        (Some(a), Some(b))
            if scopes.contexts[a.expression].block != scopes.contexts[b.expression].block =>
        {
            return Err(Diagnostic::new(
                "E_SLICE_SCOPE",
                "slice boundaries must be in the same lexical block",
            ));
        }
        (Some(a), None) | (None, Some(a)) if scopes.contexts[a.expression].depth != 1 => {
            return Err(Diagnostic::new(
                "E_SLICE_SCOPE",
                "open slice boundary must be in the function root block",
            ));
        }
        _ => {}
    }
    let start = from.as_ref().map_or(0, |b| b.expression);
    let end = to.as_ref().map_or(sites.expressions, |b| b.expression);
    if start >= end {
        return Err(Diagnostic::new(
            "E_SLICE_ORDER",
            "slice from must precede to",
        ));
    }
    Ok(ResolvedSlice {
        from,
        to,
        start,
        end,
    })
}
