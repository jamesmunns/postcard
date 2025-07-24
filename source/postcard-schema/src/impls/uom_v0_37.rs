//! Implementations of the [`Schema`] trait for the `uom` crate v0.37

use crate::Schema;

impl<D, U, V> Schema for uom_v0_37::si::Quantity<D, U, V>
where
    D: uom_v0_37::si::Dimension + ?Sized,
    U: uom_v0_37::si::Units<V> + ?Sized,
    V: uom_v0_37::num::Num + uom_v0_37::Conversion<V> + Schema,
{
    const SCHEMA: &'static crate::schema::NamedType = V::SCHEMA;
}

#[test]
fn f32_schema_check() {
    type T = uom_v0_37::si::f32::Acceleration;
    assert_eq!(T::SCHEMA.ty, f32::SCHEMA.ty);
}

#[test]
fn f64_schema_check() {
    type T = uom_v0_37::si::f64::ThermodynamicTemperature;
    assert_eq!(T::SCHEMA.ty, f64::SCHEMA.ty);
}

#[test]
fn u8_schema_check() {
    type T = uom_v0_37::si::u8::AngularVelocity;
    assert_eq!(T::SCHEMA.ty, u8::SCHEMA.ty);
}

#[test]
fn u8_conversion() {
    let x = uom_v0_37::si::u8::Acceleration::new::<
        uom_v0_37::si::acceleration::meter_per_second_squared,
    >(4);
    let y = postcard::to_stdvec(&x).unwrap();
    assert_eq!(Some(&4), y.first());
}
