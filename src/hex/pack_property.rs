use crate::hex::{PropertyItem, Property};
use crate::message::v5::ConnectMessage;

pub fn connect(data: &Vec<PropertyItem>) -> Vec<u8> {
    let mut length = 0_usize;
    let mut body = vec![];
    for item in data {
        if item.0.is_connect_property() {
            Property::pack_property_handle(item, &mut length, &mut body);
        }
    }
    body.insert(0, length as u8);
    body
}

pub fn connack(data: &Vec<PropertyItem>) -> Vec<u8> {
    let mut length = 0_usize;
    let mut body = vec![];
    for item in data {
        if item.0.is_connack_property() {
            Property::pack_property_handle(item, &mut length, &mut body);
        }
    }
    body.insert(0, length as u8);
    body
}

pub fn will_properties(data: &Vec<PropertyItem>) -> Vec<u8> {
    let mut length = 0_usize;
    let mut body = vec![];

    for item in data {
        if item.0.is_will_property() {
            Property::pack_property_handle(item, &mut length, &mut body);
        }
    }
    body.insert(0, length as u8);
    body
}
