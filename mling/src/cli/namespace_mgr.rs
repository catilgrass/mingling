use mingling::{
    ShellContext, Suggest, SuggestItem,
    macros::{chain, completion, dispatcher, pack, r_println, renderer, route, suggest},
    parser::{Picker, Yes},
};

use crate::namespace_manager::{list_namespaces, remove_namespace, set_namespace_trusted};

dispatcher!("trust", TrustNamespaceCommand => TrustNamespaceEntry);
dispatcher!("untrust", UntrustNamespaceCommand => UntrustNamespaceEntry);

dispatcher!("set-trust", SetTrustNamespaceCommand => SetTrustNamespaceEntry);

dispatcher!("rm-namespace", RemoveNamespaceCommand => RemoveNamespaceEntry);

pack!(ErrorNamespaceNotProvided = ());
pack!(ResultNamespaceTrustChanged = ());
pack!(ResultNamespaceRemoved = ());

#[completion(TrustNamespaceEntry)]
pub(crate) fn comp_trust(ctx: &ShellContext) -> Suggest {
    if ctx.previous_word == "trust" {
        return Suggest::Suggest(
            list_namespaces(false, true, true)
                .into_iter()
                .map(SuggestItem::new)
                .collect::<std::collections::BTreeSet<_>>(),
        );
    }
    return suggest!();
}

#[completion(UntrustNamespaceEntry)]
pub(crate) fn comp_untrust(ctx: &ShellContext) -> Suggest {
    if ctx.previous_word == "untrust" {
        return Suggest::Suggest(
            list_namespaces(true, false, true)
                .into_iter()
                .map(SuggestItem::new)
                .collect::<std::collections::BTreeSet<_>>(),
        );
    }
    return suggest!();
}

#[completion(SetTrustNamespaceEntry)]
pub(crate) fn comp_set_trust(ctx: &ShellContext) -> Suggest {
    if ctx.typing_argument() {
        return suggest!(
            "-t": "Whether to trust this namespace",
            "--trusted": "Whether to trust this namespace",
        );
    }
    if ctx.filling_argument_first(["-t", "--trusted"]) {
        return suggest!("yes", "no");
    }
    if ctx.previous_word == "set-trust" {
        return Suggest::Suggest(
            list_namespaces(true, true, true)
                .into_iter()
                .map(SuggestItem::new)
                .collect::<std::collections::BTreeSet<_>>(),
        );
    }
    return suggest!();
}

#[completion(RemoveNamespaceEntry)]
pub(crate) fn comp_remove_namespace(ctx: &ShellContext) -> Suggest {
    if ctx.previous_word == "rm-namespace" {
        return Suggest::Suggest(
            list_namespaces(true, true, true)
                .into_iter()
                .map(SuggestItem::new)
                .collect::<std::collections::BTreeSet<_>>(),
        );
    }
    return suggest!();
}

#[chain]
pub(crate) fn handle_set_trust(p: SetTrustNamespaceEntry) -> Next {
    let (trusted, namespace) = route!(
        Picker::new(p.inner)
            .pick::<Yes>(["-t", "--trusted"])
            .pick_or_route((), ErrorNamespaceNotProvided::default().to_render())
            .unpack()
    );
    set_namespace_trusted(namespace, trusted.is_yes());
    ResultNamespaceTrustChanged::default().to_render()
}

#[chain]
pub(crate) fn handle_trust(p: TrustNamespaceEntry) -> Next {
    SetTrustNamespaceEntry::new({
        let mut args = p.inner.clone();
        args.extend(vec!["-t".to_string(), "yes".to_string()]);
        args
    })
}

#[chain]
pub(crate) fn handle_untrust(p: UntrustNamespaceEntry) -> Next {
    SetTrustNamespaceEntry::new({
        let mut args = p.inner.clone();
        args.extend(vec!["-t".to_string(), "no".to_string()]);
        args
    })
}

#[chain]
pub(crate) fn handle_remove_namespace(p: RemoveNamespaceEntry) -> Next {
    let namespace = route!(
        Picker::new(p.inner)
            .pick_or_route((), ErrorNamespaceNotProvided::default().to_render())
            .unpack()
    );
    remove_namespace(namespace);
    ResultNamespaceRemoved::default().to_render()
}

#[renderer]
pub(crate) fn render_error_namespace_not_provided(_prev: ErrorNamespaceNotProvided) {
    r_println!("Error: no namespace was provided!")
}
