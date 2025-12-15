use std::str::SplitWhitespace;
use tokio::io;
use tokio::io::Result;

pub struct Parser<'a> {
    pub split: &'a mut SplitWhitespace<'a>,
}
impl Parser<'_> {
    fn base_parse(&mut self, param_name: &str) -> Result<&str> {
        let result = self.split.next().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("-ERROR: {param_name} parameter is required."),
            )
        })?;
        Ok(result)
    }
    pub fn parse_key(&mut self) -> Result<String> {
        Ok(self.base_parse("KEY")?.to_string())
    }
    pub fn parse_value(&mut self) -> Result<String> {
        Ok(self.base_parse("VALUE")?.to_string())
    }
    pub fn parse_seconds(&mut self) -> Result<u64> {
        let param_name = "SECONDS";
        let seconds = self.base_parse(param_name)?.parse::<u64>().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "-ERROR: SECONDS parameter needs to be of type: u64",
            )
        })?;
        Ok(seconds)
    }
    pub fn parse_keys(&mut self) -> Result<Vec<String>> {
        let keys: Vec<String> = self.split.map(|s| s.to_string()).collect();
        if keys.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "-ERROR: MGET requires at least one KEY parameter.",
            ));
        }
        Ok(keys)
    }
    pub fn parse_key_value_pairs(&mut self) -> Result<(Vec<String>, Vec<String>)> {
        let args: Vec<String> = self.split.map(|s| s.to_string()).collect();
        if args.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "-ERROR: MSET requires at least one KEY VALUE pair parameter.",
            ));
        }
        if args.len() % 2 != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "-ERROR: Uneven number of keys and values.",
            ));
        }
        let mut keys: Vec<String> = [].to_vec();
        let mut values: Vec<String> = [].to_vec();
        let mut i = 0;
        for arg in args {
            if i % 2 == 1 {
                values.push(arg.to_string());
            } else {
                keys.push(arg.to_string());
            }
            i += 1;
        }
        Ok((keys, values))
    }
}
