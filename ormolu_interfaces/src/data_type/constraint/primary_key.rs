use crate::{HasPrimaryKey, Key};
use sqlx::{Database, Decode, prelude::Type};
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::{fmt::Display, marker::PhantomData};

#[repr(transparent)]
pub struct PrimaryKey<Entity, T> {
    value: T,
    _entity: PhantomData<fn() -> Entity>,
}

impl<E, T> Hash for PrimaryKey<E, T>
where
    T: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<E, T> AsRef<T> for PrimaryKey<E, T> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

impl<E, T> Clone for PrimaryKey<E, T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            _entity: PhantomData,
            value: self.value.clone(),
        }
    }
}

impl<E, T> Copy for PrimaryKey<E, T> where T: Copy {}

impl<E, T> PartialEq for PrimaryKey<E, T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<E, T> std::fmt::Debug for PrimaryKey<E, T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrimaryKey")
            .field("value", &self.value)
            .finish()
    }
}

impl<E, T> From<T> for PrimaryKey<E, T> {
    fn from(value: T) -> Self {
        Self {
            _entity: PhantomData,
            value,
        }
    }
}

impl<DB: Database, E, T> Type<DB> for PrimaryKey<E, T>
where
    String: Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <String as Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <String as Type<DB>>::compatible(ty)
    }
}

impl<'r, DB: Database, E, T> Decode<'r, DB> for PrimaryKey<E, T>
where
    String: Decode<'r, DB>,
{
    fn decode(
        value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        todo!()
    }
}

impl<E, T> Deref for PrimaryKey<E, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<E, T> DerefMut for PrimaryKey<E, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<Entity, T> Key<Entity, T> for PrimaryKey<Entity, T>
where
    Entity: HasPrimaryKey<T>,
{
    fn get_entity(&self) -> impl Future<Output = Result<Option<Entity>, crate::OrmoluError>> {
        Entity::get_by_primary_key(self)
    }
}

impl<E, T> std::fmt::Display for PrimaryKey<E, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}
