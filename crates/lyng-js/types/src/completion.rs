use super::Value;
use lyng_js_common::AtomId;

/// Stable Result-shaped completion surface for spec-facing runtime helpers.
pub type Completion<T> = Result<T, AbruptCompletion>;

/// Abrupt completion variants used by spec-facing slow paths.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AbruptCompletion {
    Throw(Value),
    Return(Value),
    Break(Option<AtomId>),
    Continue(Option<AtomId>),
}

impl AbruptCompletion {
    #[inline]
    pub const fn throw(value: Value) -> Self {
        Self::Throw(value)
    }

    #[inline]
    pub const fn return_(value: Value) -> Self {
        Self::Return(value)
    }

    #[inline]
    pub const fn break_(label: Option<AtomId>) -> Self {
        Self::Break(label)
    }

    #[inline]
    pub const fn continue_(label: Option<AtomId>) -> Self {
        Self::Continue(label)
    }

    #[inline]
    pub const fn is_throw(self) -> bool {
        matches!(self, Self::Throw(_))
    }

    #[inline]
    pub const fn is_return(self) -> bool {
        matches!(self, Self::Return(_))
    }

    #[inline]
    pub const fn is_break(self) -> bool {
        matches!(self, Self::Break(_))
    }

    #[inline]
    pub const fn is_continue(self) -> bool {
        matches!(self, Self::Continue(_))
    }

    #[inline]
    pub const fn thrown_value(self) -> Option<Value> {
        match self {
            Self::Throw(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub const fn return_value(self) -> Option<Value> {
        match self {
            Self::Return(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub const fn break_label(self) -> Option<Option<AtomId>> {
        match self {
            Self::Break(label) => Some(label),
            _ => None,
        }
    }

    #[inline]
    pub const fn continue_label(self) -> Option<Option<AtomId>> {
        match self {
            Self::Continue(label) => Some(label),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn maybe_throw(should_throw: bool) -> Completion<()> {
        if should_throw {
            Err(AbruptCompletion::throw(Value::from_smi(7)))
        } else {
            Ok(())
        }
    }

    fn maybe_break(label: Option<AtomId>) -> Completion<()> {
        label.map_or(Ok(()), |label| Err(AbruptCompletion::break_(Some(label))))
    }

    fn propagate_with_question_mark(should_throw: bool, label: Option<AtomId>) -> Completion<u32> {
        maybe_throw(should_throw)?;
        maybe_break(label)?;
        Ok(11)
    }

    #[test]
    fn abrupt_variants_preserve_values_and_labels() {
        let thrown = AbruptCompletion::throw(Value::from_smi(3));
        let returned = AbruptCompletion::return_(Value::undefined());
        let labeled_break = AbruptCompletion::break_(Some(AtomId::from_raw(17)));
        let unlabeled_continue = AbruptCompletion::continue_(None);

        assert!(thrown.is_throw());
        assert_eq!(thrown.thrown_value(), Some(Value::from_smi(3)));
        assert!(returned.is_return());
        assert_eq!(returned.return_value(), Some(Value::undefined()));
        assert!(labeled_break.is_break());
        assert_eq!(
            labeled_break.break_label(),
            Some(Some(AtomId::from_raw(17)))
        );
        assert!(unlabeled_continue.is_continue());
        assert_eq!(unlabeled_continue.continue_label(), Some(None));
    }

    #[test]
    fn completion_alias_preserves_normal_and_abrupt_results() {
        let normal: Completion<u32> = Ok(41);
        let abrupt: Completion<u32> = Err(AbruptCompletion::return_(Value::null()));

        assert_eq!(normal, Ok(41));
        assert_eq!(abrupt, Err(AbruptCompletion::Return(Value::null())));
    }

    #[test]
    fn completion_alias_supports_question_mark_propagation() {
        assert_eq!(propagate_with_question_mark(false, None), Ok(11));
        assert_eq!(
            propagate_with_question_mark(true, None),
            Err(AbruptCompletion::Throw(Value::from_smi(7)))
        );
        assert_eq!(
            propagate_with_question_mark(false, Some(AtomId::from_raw(23))),
            Err(AbruptCompletion::Break(Some(AtomId::from_raw(23))))
        );
    }
}
