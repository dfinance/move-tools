use libra::account::Identifier;
use move_core_types::language_storage::StructTag;
use libra::prelude::*;
use rv::{AnnotatedMoveStruct, AnnotatedMoveValue};
use serde::Serialize;

#[derive(Serialize)]
pub struct AnnotatedMoveStructHelper(
    #[serde(with = "AnnotatedMoveStructExt")] pub AnnotatedMoveStruct,
);

#[derive(Serialize)]
#[serde(remote = "rv::AnnotatedMoveStruct")]
struct AnnotatedMoveStructExt {
    is_resource: bool,
    type_: StructTag,
    #[serde(with = "vec_annotated_move_value_mapped")]
    value: Vec<(Identifier, AnnotatedMoveValue)>,
}

#[derive(Serialize)]
#[serde(remote = "rv::AnnotatedMoveValue")]
enum AnnotatedMoveValueExt {
    U8(u8),
    U64(u64),
    U128(u128),
    Bool(bool),
    Address(AccountAddress),
    #[serde(with = "vec_annotated_move_value")]
    Vector(Vec<AnnotatedMoveValue>),
    Bytes(Vec<u8>),
    Struct(#[serde(with = "AnnotatedMoveStructExt")] AnnotatedMoveStruct),
}

mod vec_annotated_move_value {
    use super::{AnnotatedMoveValue, AnnotatedMoveValueExt};
    use serde::{Serialize, Serializer};

    pub fn serialize<S>(vec: &Vec<AnnotatedMoveValue>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Helper<'a>(#[serde(with = "AnnotatedMoveValueExt")] &'a AnnotatedMoveValue);

        vec.into_iter()
            .map(Helper)
            .collect::<Vec<_>>()
            .serialize(serializer)
    }
}

mod vec_annotated_move_value_mapped {
    use super::{AnnotatedMoveValue, AnnotatedMoveValueExt};
    use super::Identifier;
    use serde::{Serialize, Serializer};

    pub fn serialize<S>(
        vec: &Vec<(Identifier, AnnotatedMoveValue)>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Helper<'a> {
            id: &'a Identifier,
            #[serde(with = "AnnotatedMoveValueExt")]
            value: &'a AnnotatedMoveValue,
        }

        vec.into_iter()
            .map(|(id, value)| Helper { id, value })
            .collect::<Vec<_>>()
            .serialize(serializer)
    }
}
