//! Mutator for binary operation `+`.

use std::convert::TryFrom;
use std::ops::Deref;
use std::ops::Not;

use quote::quote_spanned;
use syn::Expr;

use crate::comm::Mutation;
use crate::transformer::ast_inspect::ExprUnopNot;
use crate::transformer::transform_info::SharedTransformInfo;
use crate::transformer::TransformContext;

use crate::optimistic::NotToNone;
use crate::MutagenRuntimeConfig;

pub struct MutatorUnopNot {}

impl MutatorUnopNot {
    pub fn run<T: Not>(
        mutator_id: usize,
        val: T,
        runtime: impl Deref<Target = MutagenRuntimeConfig>,
    ) -> <T as Not>::Output {
        runtime.covered(mutator_id);
        if runtime.is_mutation_active(mutator_id) {
            val.may_none()
        } else {
            !val
        }
    }

    pub fn run_native_num<I: Not<Output = I>>(
        mutator_id: usize,
        val: I,
        runtime: impl Deref<Target = MutagenRuntimeConfig>,
    ) -> I {
        runtime.covered(mutator_id);
        if runtime.is_mutation_active(mutator_id) {
            val
        } else {
            !val
        }
    }

    pub fn transform(
        e: Expr,
        transform_info: &SharedTransformInfo,
        context: &TransformContext,
    ) -> Expr {
        let e = match ExprUnopNot::try_from(e) {
            Ok(e) => e,
            Err(e) => return e,
        };

        let mutator_id = transform_info.add_mutation(Mutation::new_spanned(
            &context,
            "unop_not".to_owned(),
            "!".to_owned(),
            "".to_owned(),
            e.span,
        ));

        let expr = &e.expr;

        // if the current expression is based on numbers, use the function `run_native_num` instead
        let run_fn = if context.is_num_expr() {
            quote_spanned! {e.span=> run_native_num}
        } else {
            quote_spanned! {e.span=> run}
        };

        syn::parse2(quote_spanned! {e.span=>
            ::mutagen::mutator::MutatorUnopNot::#run_fn(
                    #mutator_id,
                    #expr,
                    ::mutagen::MutagenRuntimeConfig::get_default()
                )
        })
        .expect("transformed code invalid")
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn boolnot_inactive() {
        // input is true, but will be negated by non-active mutator
        let result = MutatorUnopNot::run(1, true, &MutagenRuntimeConfig::without_mutation());
        assert_eq!(result, false);
    }
    #[test]
    fn boolnot_active() {
        let result = MutatorUnopNot::run(1, true, &MutagenRuntimeConfig::with_mutation_id(1));
        assert_eq!(result, true);
    }
    #[test]
    fn intnot_active() {
        let result =
            MutatorUnopNot::run_native_num(1, 1, &MutagenRuntimeConfig::with_mutation_id(1));
        assert_eq!(result, 1);
    }

    pub use crate::optimistic::{TypeWithNotOtherOutput, TypeWithNotTarget};

    #[test]
    fn optimistic_incorrect_inactive() {
        let result = MutatorUnopNot::run(
            1,
            TypeWithNotOtherOutput(),
            &MutagenRuntimeConfig::without_mutation(),
        );
        assert_eq!(result, TypeWithNotTarget());
    }
    #[test]
    #[should_panic]
    fn optimistic_incorrect_active() {
        MutatorUnopNot::run(
            1,
            TypeWithNotOtherOutput(),
            &MutagenRuntimeConfig::with_mutation_id(1),
        );
    }
}
