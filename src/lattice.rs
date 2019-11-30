use crate::lattice::constraints::TypeConstraint;
use ena::unify::{InPlaceUnificationTable, UnificationTable, UnifyKey};
use std::slice::Iter;

//mod lattice_type;
//mod type_constraint;
//mod ena_if;
pub mod constraints;
pub mod reification;

/// The main struct for the type checking procedure.
/// It manages a set of abstract types in a lattice-like structure and perform a union-find procedure to derive
/// the least concrete abstract type that satisfies a defined set of constraints.
/// Each abstract type is referred to with a key assigned by the `TypeChecker` (refer to
/// `TypeChecker::new_key(&mut self)`).
///
/// # Usage
/// Requires two types: `Key` and a type representing an abstract type.
/// `Key` needs to implement `ena::UnifyKey`, which has an associated type `ena::Key::Value` that
/// is the abstract type.  As such, the abstract type needs to implement `ena::UnifyValue` providing an abstract
/// "meet" or "unification" function, and `AbstractType`.  The latter trait grants access to a `AbstractType::top()`
/// function that represents an unconstrained abstract type.
#[derive(Debug, Clone)]
pub struct TypeChecker<Key: UnifyKey>
where
    Key::Value: AbstractType,
{
    store: InPlaceUnificationTable<Key>,
    keys: Vec<TypeCheckKey<Key>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypeCheckKey<Key: UnifyKey>(Key)
where
    Key::Value: AbstractType;

pub trait AbstractType: UpperBounded {}

/// Provides assess to an element representing the lower bound of the type lattice.
/// This usually represents a type error.
pub trait LowerBounded: Eq + Sized {
    fn bot() -> Self;

    /// Determines if an element is the lower bound of the type lattice.
    fn is_bot(&self) -> bool {
        self == &Self::bot()
    }
}

/// Provides assess to an element representing the upper bound of the type lattice.
/// This usually represents an unconstrained type.
pub trait UpperBounded: Eq + Sized {
    fn top() -> Self;

    /// Determines if an element is the upper bound of the type lattice.
    fn is_top(&self) -> bool {
        self == &Self::top()
    }
}
impl<Key: UnifyKey> Default for TypeChecker<Key>
where
    Key::Value: AbstractType,
{
    fn default() -> Self {
        TypeChecker::new()
    }
}

//// %%%%%%%%%%% PUBLIC INTERFACE %%%%%%%%%%%
impl<Key: UnifyKey> TypeChecker<Key>
where
    Key::Value: AbstractType,
{
    /// Creates a new, empty `TypeChecker`.
    pub fn new() -> Self {
        TypeChecker { store: UnificationTable::new(), keys: Vec::new() }
    }
}

impl<Key: UnifyKey> TypeChecker<Key>
where
    Key::Value: AbstractType,
{
    /// Returns a view on the current state of `self`.  Returns a mapping of all keys known to
    /// `self` to the `Key::Value` associated with it.
    pub fn get_state(&mut self) -> Vec<(TypeCheckKey<Key>, Key::Value)> {
        let keys = self.keys.clone();
        keys.into_iter().map(|key| key).map(|key| (key, self.get_type(key))).collect()
    }

    /// Creates a new unconstrained variable that can be referred to using the returned
    /// `TypeCheckKey`.  The current state of it can be accessed using
    /// `TypeChecker::get_type(TypeCheckKey)` and constraints can be imposed using `TypeChecker::impose(TypeConstraint)`.
    /// `TypeCheckKey` provides means to create such `TypeConstraints`.
    pub fn new_key(&mut self) -> TypeCheckKey<Key> {
        let new = TypeCheckKey(self.store.new_key(<Key::Value as UpperBounded>::top()));
        self.keys.push(new);
        new
    }

    /// Imposes `constr` on the current state of the type checking procedure.
    /// This may or may not change the abstract types of some keys.
    pub fn impose(&mut self, constr: TypeConstraint<Key>) {
        use TypeConstraint::*;
        match constr {
            MoreConcreteThanAll { target, args } => {
                // Look-up all constrains of args, bound `target` by each.
                args.iter().for_each(|a| {
                    let bound = self.store.probe_value(a.0);
                    let _ = self.store.unify_var_value(target.0, bound);
                });
            }
            BoundByValue { target, bound } => {
                let _ = self.store.unify_var_value(target.0, bound);
            }
            MoreConcreteThanType { target, args } => {
                args.into_iter().for_each(|bound| {
                    let _ = self.store.unify_var_value(target.0, bound);
                });
            }
        }
    }

    /// Returns the abstract type associated with `key`.
    pub fn get_type(&mut self, key: TypeCheckKey<Key>) -> Key::Value {
        self.store.probe_value(key.0)
    }

    /// Returns an iterator over all keys currently present in the type checking procedure.
    pub fn keys(&self) -> Iter<TypeCheckKey<Key>> {
        self.keys.iter()
    }
}
