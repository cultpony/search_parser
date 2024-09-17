mod ip_cidr;

use regex::Regex;

use super::FSMStateMatcher;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataValueType {
    Float,
    Integer,
    Boolean,
    IP,
    CIDR,
    RelativeDate,
    AbsoluteDate,
    String,
}

impl FSMStateMatcher for DataValueType {
    fn matches(self, inp: &str) -> Option<u8> {
        thread_local! {
            static FLOAT: Regex = Regex::new(r"(?P<float>^[+-]{0,1}\d+\.\d+)").unwrap();
            static INTEGER: Regex = Regex::new(r"(?P<int>^[+-]{0,1}\d+)").unwrap();
            static FIELD: Regex = Regex::new(r"(?P<field>^[^.\(\),\s]+)(?:\s(AND|OR|\.[gl]te?:|\.n?eq)(\s)){0,1}").unwrap();
            static IP_CIDR: Regex = Regex::new(r"(?P<ip>(\b25[0-5]|\b2[0-4][0-9]|\b[01]?[0-9][0-9]?)(\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)){3}|(([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])))(?P<netmask>/\d+)?").unwrap();
            static ABS_DATE: Regex = Regex::new(r"^(?P<year>\d{4}(-(?P<month>\d{2})(-(?P<day>\d{2}))?)?)((T| )(?P<hour>\d{2}(:(?P<minute>\d{2}(:(?P<second>\d{2}))?))?))?(?P<offset_hour>[+-]\d{2}(:(?P<offset_minute>\d{2}))?|(?P<zulu>Z))?").unwrap();
            static REL_DATE: Regex = Regex::new(r"((?P<years>\d+ years?)\s+)?((?P<months>\d+ months?)\s+)?((?P<weeks>\d+ weeks?)\s+)?((?P<days>\d+ days?)\s+)?((?P<hours>\d+ hours?)\s+)?((?P<minutes>\d+ minutes?)\s+)?((?P<seconds>\d+ seconds?)\s+)?(ago|from now)").unwrap();
        }
        match self {
            DataValueType::Float => {
                FLOAT.with(|float| float.find_at(inp, 0).map(|x| x.end() as u8))
            }
            DataValueType::Integer => {
                INTEGER.with(|int| int.find_at(inp, 0).map(|x| x.end() as u8))
            }
            DataValueType::Boolean if inp.to_ascii_lowercase().starts_with("true") => {
                Some("true".len() as u8)
            }
            DataValueType::Boolean if inp.to_ascii_lowercase().starts_with("yes") => {
                Some("yes".len() as u8)
            }
            DataValueType::Boolean if inp.to_ascii_lowercase().starts_with("false") => {
                Some("false".len() as u8)
            }
            DataValueType::Boolean if inp.to_ascii_lowercase().starts_with("no") => {
                Some("no".len() as u8)
            }
            DataValueType::Boolean => None,
            DataValueType::IP => None,   // TODO: implement IP matching
            DataValueType::CIDR => None, // TODO: implement CIDR matching
            DataValueType::RelativeDate => {
                REL_DATE.with(|rel_date| rel_date.find_at(inp, 0).map(|x| x.end() as u8))
            }
            DataValueType::AbsoluteDate => {
                ABS_DATE.with(|abs_date| abs_date.find_at(inp, 0).map(|x| x.end() as u8))
            }
            DataValueType::String => None, // TODO: implement String matching
        }
    }

    fn maximum_bound(self) -> Option<u8> {
        match self {
            DataValueType::Float => None,
            DataValueType::Integer => None,
            DataValueType::Boolean => Some(3),
            // An IPv6 can only be this long at max, so this is what we need at worst
            DataValueType::IP => Some("0000:0000:0000:0000:0000:0000:0000:0000".len() as u8),
            // A CIDR is only this long
            DataValueType::CIDR => Some("0000:0000:0000:0000:0000:0000:0000:0000/128".len() as u8),
            DataValueType::RelativeDate => None,
            DataValueType::AbsoluteDate => None,
            DataValueType::String => None,
        }
    }
}
