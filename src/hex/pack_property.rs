use crate::hex::{PropertyItem, Property};
use crate::message::v5::ConnectMessage;

pub fn connect(data: &Vec<PropertyItem>) -> Vec<u8> {
    let (length, mut body) = Property::pack_property_handle(data);
    body.insert(0, length as u8);
    body
}
