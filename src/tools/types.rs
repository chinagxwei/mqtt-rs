use num_enum::TryFromPrimitive;

#[derive(Debug, Copy, Clone, TryFromPrimitive, Ord, PartialOrd, Eq, PartialEq)]
#[repr(u8)]
pub enum TypeKind {
    CONNECT = 1,
    CONNACK,
    PUBLISH,
    PUBACK,
    PUBREC,
    PUBREL,
    PUBCOMP,
    SUBSCRIBE,
    SUBACK,
    UNSUBSCRIBE,
    UNSUBACK,
    PINGREQ,
    PINGRESP,
    DISCONNECT,
    AUTH,
}

impl TypeKind {
    pub fn is_pubrel(&self) -> bool {
        matches!(self,TypeKind::PUBREL)
    }
}

impl TypeKind {
    pub fn as_str(&self) -> &'static str {
        match *self {
            TypeKind::CONNECT => { "connect" }
            TypeKind::CONNACK => { "connack" }
            TypeKind::PUBLISH => { "publish" }
            TypeKind::PUBACK => { "puback" }
            TypeKind::PUBREC => { "pubrec" }
            TypeKind::PUBREL => { "pubrel" }
            TypeKind::PUBCOMP => { "pubcomp" }
            TypeKind::SUBSCRIBE => { "subscribe" }
            TypeKind::SUBACK => { "suback" }
            TypeKind::UNSUBSCRIBE => { "unsubscribe" }
            TypeKind::UNSUBACK => { "unsuback" }
            TypeKind::PINGREQ => { "pingreq" }
            TypeKind::PINGRESP => { "pingresp" }
            TypeKind::DISCONNECT => { "disconnect" }
            TypeKind::AUTH => { "auth" }
        }
    }

    pub fn as_header_byte(&self) -> u8 {
        match *self {
            TypeKind::CONNECT => { (TypeKind::CONNECT as u8) << 4 }
            TypeKind::CONNACK => { (TypeKind::CONNACK as u8) << 4 }
            TypeKind::PUBLISH => { (TypeKind::PUBLISH as u8) << 4 }
            TypeKind::PUBACK => { (TypeKind::PUBACK as u8) << 4 }
            TypeKind::PUBREC => { (TypeKind::PUBREC as u8) << 4 }
            TypeKind::PUBREL => { (TypeKind::PUBREL as u8) << 4 }
            TypeKind::PUBCOMP => { (TypeKind::PUBCOMP as u8) << 4 }
            TypeKind::SUBSCRIBE => { (TypeKind::SUBSCRIBE as u8) << 4 }
            TypeKind::SUBACK => { (TypeKind::SUBACK as u8) << 4 }
            TypeKind::UNSUBSCRIBE => { (TypeKind::UNSUBSCRIBE as u8) << 4 }
            TypeKind::UNSUBACK => { (TypeKind::UNSUBACK as u8) << 4 }
            TypeKind::PINGREQ => { (TypeKind::PINGREQ as u8) << 4 }
            TypeKind::PINGRESP => { (TypeKind::PINGRESP as u8) << 4 }
            TypeKind::DISCONNECT => { (TypeKind::DISCONNECT as u8) << 4 }
            TypeKind::AUTH => { (TypeKind::AUTH as u8) << 4 }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn test() {
        let kind = TypeKind::try_from(1_u8);
        if let Ok(res) = kind {
            dbg!(res);
        }
    }

    #[test]
    fn test2() {
        // assert_eq!(TypeKind::PINGREQ.as_byte(), 192)
        // println!("{:b}", TypeKind::AUTH.as_byte());
    }
}


