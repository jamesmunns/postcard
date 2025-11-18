//! Implementations of the [`Schema`] trait for the `log` crate v0.4

use crate::{
    schema::{DataModelType, DataModelVariant, NamedType, NamedVariant},
    Schema,
};

impl Schema for log_v0_4::Level {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "Level",
        ty: &DataModelType::Enum(&[
            &NamedVariant {
                name: "ERROR",
                ty: &DataModelVariant::UnitVariant,
            },
            &NamedVariant {
                name: "WARN",
                ty: &DataModelVariant::UnitVariant,
            },
            &NamedVariant {
                name: "INFO",
                ty: &DataModelVariant::UnitVariant,
            },
            &NamedVariant {
                name: "DEBUG",
                ty: &DataModelVariant::UnitVariant,
            },
            &NamedVariant {
                name: "TRACE",
                ty: &DataModelVariant::UnitVariant,
            },
        ]),
    };
}

impl Schema for log_v0_4::LevelFilter {
    const SCHEMA: &'static NamedType = &NamedType {
        name: "LevelFilter",
        ty: &DataModelType::Enum(&[
            &NamedVariant {
                name: "OFF",
                ty: &DataModelVariant::UnitVariant,
            },
            &NamedVariant {
                name: "ERROR",
                ty: &DataModelVariant::UnitVariant,
            },
            &NamedVariant {
                name: "WARN",
                ty: &DataModelVariant::UnitVariant,
            },
            &NamedVariant {
                name: "INFO",
                ty: &DataModelVariant::UnitVariant,
            },
            &NamedVariant {
                name: "DEBUG",
                ty: &DataModelVariant::UnitVariant,
            },
            &NamedVariant {
                name: "TRACE",
                ty: &DataModelVariant::UnitVariant,
            },
        ]),
    };
}
