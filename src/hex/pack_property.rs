use crate::hex::{PropertyItem, Property};

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

pub fn subscribe(data: &Vec<PropertyItem>) -> Vec<u8> {
    let mut length = 0_usize;
    let mut body = vec![];

    for item in data {
        if item.0.is_subscribe_property() {
            Property::pack_property_handle(item, &mut length, &mut body);
        }
    }
    body.insert(0, length as u8);
    body
}

pub fn unsubscribe(data: &Vec<PropertyItem>) -> Vec<u8> {
    let mut length = 0_usize;
    let mut body = vec![];

    for item in data {
        if item.0.is_unsubscribe_property() {
            Property::pack_property_handle(item, &mut length, &mut body);
        }
    }
    body.insert(0, length as u8);
    body
}

pub fn suback(data: &Vec<PropertyItem>) -> Vec<u8> {
    let mut length = 0_usize;
    let mut body = vec![];

    for item in data {
        if item.0.is_pub_and_sub_property() {
            Property::pack_property_handle(item, &mut length, &mut body);
        }
    }
    body.insert(0, length as u8);
    body
}

pub fn disconnect(data: &Vec<PropertyItem>) -> Vec<u8> {
    let mut length = 0_usize;
    let mut body = vec![];

    for item in data {
        if item.0.is_disconnect_property() {
            Property::pack_property_handle(item, &mut length, &mut body);
        }
    }
    body.insert(0, length as u8);
    body
}

pub fn auth(data: &Vec<PropertyItem>) -> Vec<u8>{
    let mut length = 0_usize;
    let mut body = vec![];

    for item in data {
        if item.0.is_auth_property() {
            Property::pack_property_handle(item, &mut length, &mut body);
        }
    }
    body.insert(0, length as u8);
    body
}

pub fn publish(data: &Vec<PropertyItem>) -> Vec<u8>{
    let mut length = 0_usize;
    let mut body = vec![];

    for item in data {
        if item.0.is_publish_property() {
            Property::pack_property_handle(item, &mut length, &mut body);
        }
    }
    body.insert(0, length as u8);
    body
}



