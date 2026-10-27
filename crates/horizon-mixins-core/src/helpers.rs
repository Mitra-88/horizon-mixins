use crate::{
    Diagnostic, Result,
    index::{self, Function, tokens},
    manifest::{Binding, Kind},
};
use quote::quote;
use serde::{Deserialize, Serialize};
use syn::{
    Expr, FnArg, Item, ReturnType, Type,
    visit::{self, Visit},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Operation {
    pub inputs: Vec<String>,
    pub output: String,
    pub method: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Contract {
    pub output: String,
    pub value: Option<String>,
    pub operation: Option<Operation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle: Option<ArgumentBundle>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArgumentBundle {
    pub inputs: Vec<String>,
    pub output: String,
    pub receiver: Option<String>,
}
pub fn receiver_type(f: &Function, expr: &Expr) -> Result<Type> {
    let Expr::Path(path) = expr else {
        return Err(Diagnostic::new(
            "E_UNSUPPORTED_SITE",
            "receiver must be a target reference parameter",
        ));
    };
    if path.path.segments.len() != 1 {
        return Err(Diagnostic::new(
            "E_UNSUPPORTED_SITE",
            "receiver must be one parameter name",
        ));
    }
    let name = path.path.segments[0].ident.to_string();
    f.signature
        .inputs
        .iter()
        .find_map(|arg| match arg {
            FnArg::Typed(arg) if matches!(&*arg.pat,syn::Pat::Ident(p) if p.ident==name) => {
                Some(*arg.ty.clone())
            }
            FnArg::Receiver(r) if name == "self" && r.reference.is_some() => {
                f.self_type.as_ref().map(|ty| {
                    if r.mutability.is_some() {
                        syn::parse_quote!(&mut #ty)
                    } else {
                        syn::parse_quote!(&#ty)
                    }
                })
            }
            _ => None,
        })
        .filter(|ty| matches!(ty, Type::Reference(_)))
        .ok_or_else(|| {
            Diagnostic::new(
                "E_UNSUPPORTED_SITE",
                "receiver must have an explicit shared/mutable parameter reference type",
            )
        })
}

pub fn return_type(sig: &syn::Signature) -> Type {
    match &sig.output {
        ReturnType::Default => syn::parse_quote!(()),
        ReturnType::Type(_, ty) => *ty.clone(),
    }
}
pub fn normalize_type(ty: &Type) -> String {
    tokens(ty)
        .replace(' ', "")
        .replace("::core::ops::", "")
        .replace("::std::ops::", "")
        .replace("core::ops::", "")
        .replace("std::ops::", "")
}
fn parse_type(text: &str) -> Result<Type> {
    syn::parse_str(text).map_err(|e| Diagnostic::new("E_INVALID_BINDING", e.to_string()))
}
fn unsupported_type(ty: &Type, references: bool) -> bool {
    struct Check {
        bad: bool,
        references: bool,
    }
    impl<'ast> Visit<'ast> for Check {
        fn visit_type(&mut self, ty: &'ast Type) {
            self.bad |= matches!(
                ty,
                Type::ImplTrait(_) | Type::Never(_) | Type::Infer(_) | Type::Macro(_)
            ) || (self.references && matches!(ty, Type::Reference(_)));
            visit::visit_type(self, ty);
        }
        fn visit_ident(&mut self, id: &'ast syn::Ident) {
            self.bad |= id == "Self";
        }
    }
    let mut check = Check {
        bad: false,
        references,
    };
    check.visit_type(ty);
    check.bad
}
pub fn parse_helper(source: &str) -> Result<syn::ItemFn> {
    let file =
        syn::parse_file(source).map_err(|e| Diagnostic::new("E_INVALID_HELPER", e.to_string()))?;
    if !file.attrs.is_empty() || file.items.len() != 1 {
        return Err(Diagnostic::new(
            "E_INVALID_HELPER",
            "helper file must contain exactly one private function",
        ));
    }
    let Some(Item::Fn(helper)) = file.items.into_iter().next() else {
        return Err(Diagnostic::new(
            "E_INVALID_HELPER",
            "helper must be a function",
        ));
    };
    let sig = &helper.sig;
    if !helper.attrs.is_empty()
        || !matches!(helper.vis, syn::Visibility::Inherited)
        || sig.asyncness.is_some()
        || sig.constness.is_some()
        || sig.unsafety.is_some()
        || sig.abi.is_some()
        || sig.variadic.is_some()
        || !sig.generics.params.is_empty()
        || sig.generics.where_clause.is_some()
    {
        return Err(Diagnostic::new(
            "E_INVALID_HELPER",
            "helper must be private, synchronous, safe, have no explicit generic parameters or attributes",
        ));
    }
    Ok(helper)
}
pub fn contract(
    f: &Function,
    kind: Kind,
    cancellable: bool,
    helper_source: &str,
    bindings: &[Binding],
    call: Option<&Expr>,
    call_result: Option<&str>,
) -> Result<Contract> {
    let helper = parse_helper(helper_source)?;
    let result = return_type(&f.signature);
    let value = if matches!(
        kind,
        Kind::Return
            | Kind::ModifyArg
            | Kind::ModifyConstant
            | Kind::ModifyVariable
            | Kind::ModifyArgs
    ) {
        let special = if kind == Kind::ModifyArgs {
            "$args"
        } else {
            "$value"
        };
        let values: Vec<_> = bindings.iter().filter(|b| b.value == special).collect();
        if values.len() != 1 {
            return Err(Diagnostic::new(
                "E_INVALID_BINDING",
                format!("value modifiers require exactly one {special} binding"),
            ));
        }
        let value_type = parse_type(&values[0].ty)?;
        if unsupported_type(&value_type, false) {
            return Err(Diagnostic::new(
                "E_VALUE_SIGNATURE",
                "value modifiers need a concrete value type; opaque, inferred and never types are unsupported",
            ));
        }
        Some(value_type)
    } else {
        None
    };
    let operation = if kind == Kind::WrapOperation {
        let call =
            call.ok_or_else(|| Diagnostic::new("E_PLAN_SITE", "wrapper has no original call"))?;
        let (method, arguments) = match call {
            Expr::Call(c) => (false, c.args.len()),
            Expr::MethodCall(c) => (true, c.args.len()),
            _ => return Err(Diagnostic::new("E_PLAN_SITE", "wrapper site is not a call")),
        };
        let input_count = arguments + usize::from(method);
        if bindings.len() != input_count + 1 {
            return Err(Diagnostic::new(
                "E_INVALID_BINDING",
                "wrapper must bind every receiver/argument once and $operation last; extra context is unsupported",
            ));
        }
        let mut inputs = Vec::new();
        for (i, binding) in bindings[..input_count].iter().enumerate() {
            let expected = if method && i == 0 {
                "$receiver".into()
            } else {
                format!("$arg{}", i - usize::from(method))
            };
            if binding.value != expected {
                return Err(Diagnostic::new(
                    "E_INVALID_BINDING",
                    format!("wrapper binding {} must use {expected}", binding.name),
                ));
            }
            let ty = parse_type(&binding.ty)?;
            if unsupported_type(&ty, false) {
                return Err(Diagnostic::new(
                    "E_OPERATION_SIGNATURE",
                    "operation inputs require explicit concrete types",
                ));
            }
            inputs.push(tokens(&ty));
        }
        if bindings[input_count].value != "$operation" {
            return Err(Diagnostic::new(
                "E_INVALID_BINDING",
                "$operation must be the last wrapper binding",
            ));
        }
        let output = return_type(&helper.sig);
        if unsupported_type(&output, true) {
            return Err(Diagnostic::new(
                "E_OPERATION_SIGNATURE",
                "operation result must be an explicit owned type or unit; borrowed/opaque results are unsupported",
            ));
        }
        Some(Operation {
            inputs,
            output: tokens(&output),
            method,
        })
    } else {
        None
    };
    let bundle = if kind == Kind::ModifyArgs {
        let Some(Type::Tuple(tuple)) = &value else {
            return Err(Diagnostic::new(
                "E_ARGUMENT_SIGNATURE",
                "MODIFY_ARGS $args must use a concrete tuple type",
            ));
        };
        let call =
            call.ok_or_else(|| Diagnostic::new("E_PLAN_SITE", "missing call for argument bundle"))?;
        let (count, receiver) = match call {
            Expr::Call(c) => (c.args.len(), None),
            Expr::MethodCall(c) => (c.args.len(), Some(tokens(&receiver_type(f, &c.receiver)?))),
            _ => {
                return Err(Diagnostic::new(
                    "E_PLAN_SITE",
                    "argument bundle site is not a call",
                ));
            }
        };
        if tuple.elems.len() != count {
            return Err(Diagnostic::new(
                "E_ARGUMENT_SIGNATURE",
                "$args tuple arity must match all explicit call arguments",
            ));
        }
        let output = parse_type(call_result.ok_or_else(|| {
            Diagnostic::new("E_ARGUMENT_SIGNATURE", "MODIFY_ARGS requires result type")
        })?)?;
        if unsupported_type(&output, true) {
            return Err(Diagnostic::new(
                "E_ARGUMENT_SIGNATURE",
                "MODIFY_ARGS result must be an explicit owned type or unit",
            ));
        }
        Some(ArgumentBundle {
            inputs: tuple.elems.iter().map(tokens).collect(),
            output: tokens(&output),
            receiver,
        })
    } else {
        None
    };
    let output = if let Some(operation) = &operation {
        operation.output.clone()
    } else if cancellable {
        if unsupported_type(&result, false) {
            return Err(Diagnostic::new(
                "E_UNSUPPORTED_SIGNATURE",
                "cancellation needs a concrete target result type",
            ));
        }
        tokens(&syn::parse2::<Type>(quote!(::core::ops::ControlFlow<#result>)).unwrap())
    } else if matches!(kind, Kind::Return | Kind::Overwrite) {
        tokens(&result)
    } else if let Some(value) = &value {
        tokens(value)
    } else {
        "()".into()
    };
    if kind == Kind::Return && unsupported_type(&result, false) {
        return Err(Diagnostic::new(
            "E_UNSUPPORTED_SIGNATURE",
            "RETURN needs a concrete target result type",
        ));
    }
    if kind == Kind::Return && normalize_type(value.as_ref().unwrap()) != normalize_type(&result) {
        return Err(Diagnostic::new(
            "E_INVALID_BINDING",
            "RETURN $value type must match the target result type",
        ));
    }
    Ok(Contract {
        output,
        value: value.as_ref().map(tokens),
        operation,
        bundle,
    })
}
pub fn validate_helper(
    f: &Function,
    kind: Kind,
    cancellable: bool,
    source: &str,
    bindings: &[Binding],
    contract: &Contract,
) -> Result<()> {
    let helper = parse_helper(source)?;
    let parameters = index::parameters(&f.signature)?;
    if parameters.contains(&helper.sig.ident.to_string()) {
        return Err(Diagnostic::new(
            "E_INVALID_HELPER",
            "helper name shadows a target parameter; rename it",
        ));
    }
    if helper.sig.inputs.len() != bindings.len() {
        return Err(Diagnostic::new(
            "E_INVALID_BINDING",
            "one explicit binding is required per helper parameter",
        ));
    }
    // OVERWRITE retains the MVP's compiler-validated return compatibility.
    if kind != Kind::Overwrite
        && normalize_type(&return_type(&helper.sig))
            != normalize_type(&parse_type(&contract.output)?)
    {
        let why = if cancellable {
            "cancellable HEAD helper must return core::ops::ControlFlow<TargetResult>"
        } else {
            "helper return type must match its injection contract"
        };
        return Err(Diagnostic::new("E_INVALID_HELPER", why));
    }
    let mut names = std::collections::BTreeSet::new();
    for (arg, binding) in helper.sig.inputs.iter().zip(bindings) {
        let FnArg::Typed(arg) = arg else {
            return Err(Diagnostic::new(
                "E_INVALID_HELPER",
                "helpers cannot have self receivers",
            ));
        };
        let syn::Pat::Ident(pattern) = &*arg.pat else {
            return Err(Diagnostic::new(
                "E_INVALID_BINDING",
                "helper parameters must be named identifiers",
            ));
        };
        let ty = parse_type(&binding.ty)?;
        if pattern.ident != binding.name
            || pattern.by_ref.is_some()
            || pattern.subpat.is_some()
            || !names.insert(&binding.name)
            || normalize_type(&ty) != normalize_type(&arg.ty)
        {
            return Err(Diagnostic::new(
                "E_INVALID_BINDING",
                format!(
                    "binding '{}' must match helper parameter name and type in declaration order",
                    binding.name
                ),
            ));
        }
        if binding.value.starts_with('$') {
            let valid = if crate::scopes::parse_capture(&binding.value)?.is_some() {
                !matches!(kind, Kind::Head | Kind::Overwrite | Kind::WrapOperation)
            } else if kind == Kind::WrapOperation {
                if binding.value == "$operation" {
                    let op = contract.operation.as_ref().unwrap();
                    let inputs: Vec<Type> = op
                        .inputs
                        .iter()
                        .map(|s| parse_type(s))
                        .collect::<Result<_>>()?;
                    let output = parse_type(&op.output)?;
                    let expected: Type =
                        syn::parse2(quote!(impl FnOnce(#(#inputs),*) -> #output)).unwrap();
                    normalize_type(&ty) == normalize_type(&expected)
                } else {
                    binding.value == "$receiver"
                        || binding
                            .value
                            .strip_prefix("$arg")
                            .is_some_and(|s| s.parse::<usize>().is_ok())
                }
            } else {
                contract.value.is_some()
                    && binding.value
                        == if kind == Kind::ModifyArgs {
                            "$args"
                        } else {
                            "$value"
                        }
            };
            if !valid {
                return Err(Diagnostic::new(
                    "E_INVALID_BINDING",
                    format!(
                        "invalid special binding '{}' or operation signature; use impl FnOnce with exact input/output types",
                        binding.value
                    ),
                ));
            }
        } else {
            if kind == Kind::WrapOperation {
                return Err(Diagnostic::new(
                    "E_INVALID_BINDING",
                    "wrappers accept operation inputs only",
                ));
            }
            let value: Expr = syn::parse_str(&binding.value)
                .map_err(|e| Diagnostic::new("E_INVALID_BINDING", e.to_string()))?;
            if !binding_expr(&value, &parameters) {
                return Err(Diagnostic::new(
                    "E_INVALID_BINDING",
                    format!(
                        "binding '{}' must be a target parameter or its explicit borrow/reborrow; no calls or locals",
                        binding.value
                    ),
                ));
            }
        }
        if tokens(&ty).split_whitespace().any(|s| s == "Self") {
            return Err(Diagnostic::new(
                "E_INVALID_BINDING",
                "use an explicit concrete type instead of Self",
            ));
        }
    }
    let mut reserved = Reserved(false);
    reserved.visit_item_fn(&helper);
    if reserved.0 {
        return Err(Diagnostic::new(
            "E_INVALID_HELPER",
            "function definitions and bindings beginning __horizon_mixins_ are reserved for generated code",
        ));
    }
    Ok(())
}
struct Reserved(bool);
impl<'ast> Visit<'ast> for Reserved {
    fn visit_pat_ident(&mut self, p: &'ast syn::PatIdent) {
        self.0 |= p.ident.to_string().starts_with("__horizon_mixins_");
        visit::visit_pat_ident(self, p);
    }
    fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
        self.0 |= f.sig.ident.to_string().starts_with("__horizon_mixins_");
        visit::visit_item_fn(self, f);
    }
}

fn binding_expr(expr: &Expr, parameters: &std::collections::BTreeSet<String>) -> bool {
    match expr {
        Expr::Path(p) if p.qself.is_none() && p.path.segments.len() == 1 => {
            parameters.contains(&p.path.segments[0].ident.to_string())
        }
        Expr::Reference(r) => binding_expr(&r.expr, parameters),
        Expr::Unary(u) if matches!(u.op, syn::UnOp::Deref(_)) => binding_expr(&u.expr, parameters),
        Expr::Paren(p) => binding_expr(&p.expr, parameters),
        _ => false,
    }
}
