use std::io::{Result as IoResult, Write};

use podio::{LittleEndian, WritePodExt};

use flatanf::Program;

impl Program {
    /// Writes the program out to the given Write.
    pub fn serialize<W: Write>(&self, w: &mut W) -> IoResult<()> {
        w.write_all(b"ofta")?;
        unimplemented!();
    }
}
