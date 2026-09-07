pub trait ToAstring {
    fn to_astring(self) -> String;
}

impl ToAstring for i64 {
    fn to_astring(self) -> String {
        self.format_into(&mut core::fmt::NumBuffer::new())
            .to_owned()
    }
}

impl ToAstring for f64 {
    fn to_astring(self) -> String {
        #[cfg(feature = "zmij")]
        return zmij::Buffer::new().format(self).to_owned();

        #[cfg(not(feature = "zmij"))]
        self.to_string()
    }
}
