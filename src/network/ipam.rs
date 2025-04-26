use std::str::FromStr;

pub struct SubNet {
    pub ip: [u8; 4], // 子网地址
    pub mask: u8,    // 子网掩码
}

impl FromStr for SubNet {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid subnet format",
            ));
        }

        let ip_parts: Vec<&str> = parts[0].split('.').collect();
        if ip_parts.len() != 4 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid IP address format",
            ));
        }

        let mut ip = [0; 4];
        for i in 0..4 {
            ip[i] = ip_parts[i].parse::<u8>().map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid IP address")
            })?;
        }

        let mask = parts[1]
            .parse::<u8>()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid mask"))?;

        Ok(SubNet { ip, mask })
    }
}
