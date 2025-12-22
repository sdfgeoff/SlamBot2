#![no_std]

pub trait PacketTrait {
    fn get_to(&self) -> Option<u16>;
    fn get_topic(&self) -> &str;
    fn set_from(&mut self, from: u16);
}
