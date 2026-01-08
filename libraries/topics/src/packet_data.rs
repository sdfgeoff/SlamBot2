#[macro_export]
macro_rules! packet_data_enum {
    ($($variant:ident),* $(,)?) => {
        #[derive(Serialize, Deserialize, Debug)]
        #[non_exhaustive]
        pub enum PacketData {
            $(
                $variant($variant),
            )*
        }


        impl PacketDataTrait for PacketData {
            fn topic(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant(_) => stringify!($variant),
                    )*
                }
            }
        }
    };
}
