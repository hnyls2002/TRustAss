pub enum InfoType {
    VALID,
    SYNC(SyncType),
    COMMAND,
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
    pub length: u16,
}

impl InfoType {
    pub fn into_u8(&self) -> Vec<u8> {
        match self {
            InfoType::VALID => vec!['Y' as u8, 'L' as u8, 'S' as u8],
            InfoType::SYNC(x) => vec![0, *x as u8, 0],
            InfoType::COMMAND => vec![1, 0, 0],
        }
    }

    // using "YLS" as the magic numbers to check
    // whether the connections from machines are legal
    pub fn check_valid(buf: &Vec<u8>) -> bool {
        assert_eq!(buf.len(), 3);
        return buf[0] == 'Y' as u8 && buf[1] == 'L' as u8 && buf[2] == 'S' as u8;
    }

    pub fn parse(buf: &Vec<u8>) -> Self {
        assert_eq!(buf.len(), 3);

        if InfoType::check_valid(buf) {
            return InfoType::VALID;
        }

        match buf[0] {
            0 => InfoType::SYNC(SyncType::from(buf[1])),
            1 => InfoType::COMMAND,
            _ => panic!("Invalid header!"),
        }
    }
}
