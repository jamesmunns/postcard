//! Implementations of the [`Schema`] trait for the `log` crate v0.4

use crate::{
    schema::{Data, DataModelType, Variant},
    Schema,
};

impl Schema for log_v0_4::Level {
    const SCHEMA: &'static DataModelType = &DataModelType::Enum {
        name: "Level",
        variants: &[
            &Variant {
                name: "ERROR",
                data: Data::Unit,
            },
            &Variant {
                name: "WARN",
                data: Data::Unit,
            },
            &Variant {
                name: "INFO",
                data: Data::Unit,
            },
            &Variant {
                name: "DEBUG",
                data: Data::Unit,
            },
            &Variant {
                name: "TRACE",
                data: Data::Unit,
            },
        ],
    };
}

impl Schema for log_v0_4::LevelFilter {
    const SCHEMA: &'static DataModelType = &DataModelType::Enum {
        name: "Level",
        variants: &[
            &Variant {
                name: "OFF",
                data: Data::Unit,
            },
            &Variant {
                name: "ERROR",
                data: Data::Unit,
            },
            &Variant {
                name: "WARN",
                data: Data::Unit,
            },
            &Variant {
                name: "INFO",
                data: Data::Unit,
            },
            &Variant {
                name: "DEBUG",
                data: Data::Unit,
            },
            &Variant {
                name: "TRACE",
                data: Data::Unit,
            },
        ],
    };
}
