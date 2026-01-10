#![no_std]

pub trait PacketTrait {
    /**
     * Packets can have an optional address field. If the packet has an address field it will be delivered only to that address, regardless of the topic 
     * of this packet and the subscriptions of the receivers. If the address field is None, the packet will be delivered to all subscribers of the topic.
     */
    fn get_to(&self) -> Option<u16>;


    /**
     * Get the topic string for this packet. This is used for routing packets to subscribers. The packet will be delievered to all subscribers of this topic.
     */
    fn get_topic(&self) -> &str;


    /**
     * Set the "from" address field of this packet. This is used by the router to indicate return addresses.
     */
    fn set_from(&mut self, from: u16);
}
