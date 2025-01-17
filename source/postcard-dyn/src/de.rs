use postcard_schema::schema::owned::OwnedNamedType;
use serde_json::Value;

use crate::Error;

pub fn from_slice_dyn(
    schema: &OwnedNamedType,
    data: &[u8],
) -> Result<Value, Error<postcard::Error, serde_json::Error>> {
    // Matches current value type (`serde_json::Value`)'s representation
    crate::reserialize::lossy::reserialize_with_structs_and_enums_as_maps(
        schema,
        &mut postcard::Deserializer::from_bytes(data),
        serde_json::value::Serializer,
    )
}

#[cfg(test)]
mod test {
    use postcard_schema::Schema;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    use crate::{from_slice_dyn, to_stdvec_dyn};

    #[derive(Serialize, Deserialize, Schema)]
    struct UnitStruct;

    #[derive(Serialize, Deserialize, Schema)]
    struct TupStruct1(u8);

    #[derive(Serialize, Deserialize, Schema)]
    struct TupStruct2(u8, u8);

    #[derive(Serialize, Deserialize, Schema)]
    struct Struct1 {
        pub x: bool,
        pub y: u16,
        pub z: f64,
    }

    #[derive(Serialize, Deserialize, Schema)]
    enum Enum1 {
        Alpha,
        Beta(u8),
        Gamma(Vec<u8>),
        Delta(Struct1),
        Epsilon(u8, bool),
        // TODO: struct variants are broken in the Schema derive in
        // stable postcard, tho it is fixed on the main branch.
        // Zeta { a: u8, b: bool },
    }

    #[test]
    fn smoke() {
        let bye = serde_json::to_value(Enum1::Alpha).unwrap();
        let t = to_stdvec_dyn(&Enum1::SCHEMA.into(), &bye).unwrap();
        assert_eq!(vec![0], t);
        let de = from_slice_dyn(&Enum1::SCHEMA.into(), &t).unwrap();
        assert_eq!(
            de,
            json! {
                "Alpha"
            }
        );

        let bye = serde_json::to_value(Enum1::Beta(4)).unwrap();
        let t = to_stdvec_dyn(&Enum1::SCHEMA.into(), &bye).unwrap();
        assert_eq!(vec![1, 4], t);
        let de = from_slice_dyn(&Enum1::SCHEMA.into(), &t).unwrap();
        assert_eq!(
            de,
            json! {
                {"Beta": 4}
            }
        );

        let bye = serde_json::to_value(Enum1::Gamma(vec![1, 2, 3])).unwrap();
        let t = to_stdvec_dyn(&Enum1::SCHEMA.into(), &bye).unwrap();
        assert_eq!(vec![2, 3, 1, 2, 3], t);
        let de = from_slice_dyn(&Enum1::SCHEMA.into(), &t).unwrap();
        assert_eq!(
            de,
            json! {
                {"Gamma": [1, 2, 3]}
            }
        );

        let bye = serde_json::to_value(Enum1::Delta(Struct1 {
            x: false,
            y: 1000,
            z: 4.0,
        }))
        .unwrap();
        let t = to_stdvec_dyn(&Enum1::SCHEMA.into(), &bye).unwrap();
        assert_eq!(vec![3, 0, 232, 7, 0, 0, 0, 0, 0, 0, 16, 64], t);
        let de = from_slice_dyn(&Enum1::SCHEMA.into(), &t).unwrap();
        assert_eq!(
            de,
            json! {
                {"Delta": {
                    "x": false,
                    "y": 1000,
                    "z": 4.0
                }}
            }
        );

        let bye = serde_json::to_value(Enum1::Epsilon(8, false)).unwrap();
        let t = to_stdvec_dyn(&Enum1::SCHEMA.into(), &bye).unwrap();
        assert_eq!(vec![4, 8, 0], t);
        let de = from_slice_dyn(&Enum1::SCHEMA.into(), &t).unwrap();
        assert_eq!(
            de,
            json! {
                {"Epsilon": [8, false]}
            }
        );
    }
}
