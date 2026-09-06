//! Implementations of the [`Schema`] trait for the `uuid` crate v1.0

use crate::{schema::DataModelType, Schema};

impl Schema for uuid_v1_0::Uuid {
    const SCHEMA: &'static DataModelType = &DataModelType::Seq {
        element: &DataModelType::U8,
        max_len: Some(16),
    };
}

#[cfg(test)]
mod test {
    use crate::Schema;
    #[test]
    fn basic() {
        let uuid = uuid_v1_0::uuid!("21E764AB-6B44-42D3-BE73-682E5E85C196");
        let mut buf = [0u8; 17];
        let res = postcard::to_slice(&uuid, &mut buf).unwrap();
        assert_eq!(res.len(), uuid_v1_0::Uuid::SCHEMA.max_size().unwrap());
    }
}
