pub enum InfoType {
    SYNC(SyncType),
    COMD,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum SyncType {
    REQ = 0,
    RDY = 1,
    SIG = 2,
    DLT = 3,
}

impl From<u8> for SyncType {
    fn from(value: u8) -> Self {
        assert!(value <= 3);
        match value {
            0 => SyncType::REQ,
            1 => SyncType::RDY,
            2 => SyncType::SIG,
            3 => SyncType::DLT,
            _ => panic!("Invalid SyncType!"),
        }
    }
}

pub struct Header {
    pub info_type: InfoType,
}

impl Header {
    pub fn into_u8(&self) -> Vec<u8> {
        let mut ret = Vec::new();
        match self.info_type {
            InfoType::SYNC(x) => {
                ret.push(0);
                ret.push(x as u8);
            }
            InfoType::COMD => {
                ret.push(1);
                ret.push(0);
            }
        }
        ret
    }

    pub fn parse(buf: &Vec<u8>) -> Self {
        assert_eq!(buf.len(), 2);
        match buf[0] {
            0 => Header {
                info_type: InfoType::SYNC(buf[1].into()),
            },
            1 => Header {
                info_type: InfoType::COMD,
            },
            _ => panic!("Invalid header type!"),
        }
    }
}
