//! Implementations of the [`Schema`] trait for the `chrono` crate v0.4

use crate::{schema::DataModelType, Schema};

#[cfg_attr(docsrs, doc(cfg(feature = "chrono-v0_4")))]
impl<Tz: chrono_v0_4::TimeZone> Schema for chrono_v0_4::DateTime<Tz> {
    // Chrono serializes as an rfc3339 string, which is typically about 27 bytes:
    //
    // "2026-09-06T13:24:04.659820Z"
    //  000000000011111111112222222
    //  012345678901234567890123456
    //
    // But you can also make it spit out somewhat silly output with custom precision
    // and time zones, see `longest_rfc3339` below. If you can make chrono output a
    // longer serialized output, please open an issue.
    //
    // "+262142-12-31T23:59:59.999999999+23:59"
    //  00000000001111111111222222222233333333
    //  01234567890123456789012345678901234567
    const SCHEMA: &'static DataModelType = &DataModelType::String { max_len: Some(38) };
}

#[cfg(test)]
mod test {
    use crate::Schema;
    use chrono_v0_4::{DateTime, FixedOffset, NaiveDate, TimeZone, Utc};

    fn longest_rfc3339() -> DateTime<FixedOffset> {
        let offset = FixedOffset::east_opt(23 * 3600 + 59 * 60).unwrap();
        let local = NaiveDate::MAX
            .and_hms_nano_opt(23, 59, 59, 999_999_999)
            .unwrap();
        offset.from_local_datetime(&local).single().unwrap()
    }

    #[test]
    fn chrono_longest_check() {
        let dt = longest_rfc3339();
        let mut buf = [0u8; 64];
        let used = postcard::to_slice(&dt, &mut buf).unwrap();
        assert_eq!(
            used.len(),
            DateTime::<FixedOffset>::SCHEMA.max_size().unwrap()
        );
        assert_eq!(
            postcard::from_bytes::<DateTime<FixedOffset>>(used).unwrap(),
            dt
        );
    }

    #[test]
    fn chrono_basic_check() {
        let dt = Utc::now();
        let mut buf = [0u8; 64];
        let used = postcard::to_slice(&dt, &mut buf).unwrap();
        assert!(used.len() <= DateTime::<FixedOffset>::SCHEMA.max_size().unwrap());
        assert_eq!(postcard::from_bytes::<DateTime<Utc>>(used).unwrap(), dt);
    }
}
